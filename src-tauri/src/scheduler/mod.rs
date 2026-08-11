use std::time::Duration;

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
            ReminderRepository::advance(db, &item.reminder)?;
            continue;
        }

        // 关闭通知时仍推进，避免重新打开后积压洪水
        if !notification_enabled {
            ReminderRepository::advance(db, &item.reminder)?;
            continue;
        }

        // 启动时仅补发 24 小时内错过的提醒；更早的只跳到未来
        if startup_catchup {
            let cutoff = now - ChronoDuration::hours(24);
            if item.reminder.next_trigger_at < cutoff {
                ReminderRepository::advance(db, &item.reminder)?;
                continue;
            }
        }

        let payload = NotificationPayload {
            todo_id: item.reminder.todo_id.clone(),
            title: item.todo_title.clone(),
            trigger_at: item.reminder.next_trigger_at,
        };
        if let Err(error) = NotificationService::send(app, payload) {
            tracing::error!("failed to send reminder notification: {error}");
            continue;
        }
        ReminderService::advance(db, &item.reminder.id)?;
    }

    Ok(())
}
