use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use chrono::{Duration as ChronoDuration, Utc};
use tauri::{AppHandle, Manager};
use tokio::time;

use crate::infrastructure::database::Database;
use crate::infrastructure::notification::{NotificationPayload, NotificationService};
use crate::repository::reminder_repository::ReminderRepository;
use crate::services::reminder_service::ReminderService;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

/// Scheduler 轮询间隔：在响应速度与空闲 CPU 之间折中。
const TICK_INTERVAL: Duration = Duration::from_secs(30);
/// 通知失败后首次退避；之后指数增长，上限 30 分钟。
const NOTIFY_BACKOFF_BASE: Duration = Duration::from_secs(60);
const NOTIFY_BACKOFF_MAX: Duration = Duration::from_secs(30 * 60);
/// 连续失败达到此次数后推进提醒，避免永久卡死刷屏。
const NOTIFY_MAX_FAILURES: u32 = 5;

struct NotifyBackoffEntry {
    failures: u32,
    retry_after: Instant,
}

/// 进程内通知失败退避（不落库；重启后重新尝试）。
static NOTIFY_BACKOFF: LazyLock<Mutex<HashMap<String, NotifyBackoffEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run_scheduler(app).await;
    });
}

async fn run_scheduler(app: AppHandle) {
    let mut interval = time::interval(TICK_INTERVAL);
    let mut startup_catchup_done = false;

    loop {
        interval.tick().await;

        let state = match app.try_state::<AppState>() {
            Some(state) => state,
            None => continue,
        };

        // 启动 tick 只跑 catch-up，避免同 tick 再跑普通扫描抵消 24h 策略
        if !startup_catchup_done {
            if let Err(error) = process_due_reminders(&app, &state.db, true).await {
                tracing::error!("startup reminder catchup failed: {error}");
            }
            startup_catchup_done = true;
            continue;
        }

        if let Err(error) = process_due_reminders(&app, &state.db, false).await {
            tracing::error!("scheduler tick failed: {error}");
        }
    }
}

async fn process_due_reminders(
    app: &AppHandle,
    db: &Database,
    startup_catchup: bool,
) -> Result<(), crate::errors::AppError> {
    let notification_enabled = SettingsService::is_notification_enabled(db)?;
    let now = Utc::now();
    let due_items = ReminderRepository::find_due(db, &now)?;

    for item in due_items {
        // 软删任务：只推进时间，不发通知
        if item.todo_deleted {
            clear_notify_backoff(&item.reminder.id);
            ReminderRepository::advance(db, &item.reminder)?;
            continue;
        }

        // 关闭通知时仍推进，避免重新打开后积压洪水
        if !notification_enabled {
            clear_notify_backoff(&item.reminder.id);
            ReminderRepository::advance(db, &item.reminder)?;
            continue;
        }

        // 启动时仅补发 24 小时内错过的提醒；更早的只跳到未来
        if startup_catchup {
            let cutoff = now - ChronoDuration::hours(24);
            if item.reminder.next_trigger_at < cutoff {
                clear_notify_backoff(&item.reminder.id);
                ReminderRepository::advance(db, &item.reminder)?;
                continue;
            }
        }

        if should_skip_notify(&item.reminder.id) {
            continue;
        }

        let payload = NotificationPayload {
            todo_id: item.reminder.todo_id.clone(),
            title: item.todo_title.clone(),
            trigger_at: item.reminder.next_trigger_at,
        };
        if let Err(error) = NotificationService::send(app, payload) {
            tracing::error!("failed to send reminder notification: {error}");
            if record_notify_failure(&item.reminder.id) {
                tracing::warn!(
                    reminder_id = %item.reminder.id,
                    "notification failed repeatedly; advancing reminder to stop retry spam"
                );
                clear_notify_backoff(&item.reminder.id);
                ReminderService::advance(db, &item.reminder.id)?;
            }
            continue;
        }
        clear_notify_backoff(&item.reminder.id);
        ReminderService::advance(db, &item.reminder.id)?;
    }

    Ok(())
}

fn should_skip_notify(reminder_id: &str) -> bool {
    let Ok(map) = NOTIFY_BACKOFF.lock() else {
        return false;
    };
    map.get(reminder_id)
        .is_some_and(|entry| Instant::now() < entry.retry_after)
}

/// 记录失败；返回 true 表示应放弃并 advance。
fn record_notify_failure(reminder_id: &str) -> bool {
    let Ok(mut map) = NOTIFY_BACKOFF.lock() else {
        return false;
    };
    let entry = map
        .entry(reminder_id.to_string())
        .or_insert(NotifyBackoffEntry {
            failures: 0,
            retry_after: Instant::now(),
        });
    entry.failures = entry.failures.saturating_add(1);
    let shift = entry.failures.saturating_sub(1).min(8);
    let backoff = NOTIFY_BACKOFF_BASE
        .saturating_mul(1u32 << shift)
        .min(NOTIFY_BACKOFF_MAX);
    entry.retry_after = Instant::now() + backoff;
    tracing::warn!(
        reminder_id = %reminder_id,
        failures = entry.failures,
        backoff_secs = backoff.as_secs(),
        "reminder notification backoff"
    );
    entry.failures >= NOTIFY_MAX_FAILURES
}

fn clear_notify_backoff(reminder_id: &str) {
    if let Ok(mut map) = NOTIFY_BACKOFF.lock() {
        map.remove(reminder_id);
    }
}
