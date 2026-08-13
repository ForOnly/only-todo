//! 托盘菜单与 tooltip 文案（随 ui.locale）
use tauri::menu::MenuItem;
use tauri::AppHandle;
use tauri::Manager;
use tauri::Wry;

use crate::domain::UiLocale;
use crate::float::session::companion_is_shown;
use crate::repository::settings_repository::SettingsRepository;
use crate::state::AppState;

pub struct TrayLabels {
    pub show_float: &'static str,
    pub hide_float: &'static str,
    pub new_todo: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn tray_labels(locale: UiLocale) -> TrayLabels {
    match locale {
        UiLocale::EnUS => TrayLabels {
            show_float: "Show assistant",
            hide_float: "Hide assistant",
            new_todo: "New task",
            settings: "Settings",
            quit: "Quit",
        },
        UiLocale::ZhCN => TrayLabels {
            show_float: "显示助理",
            hide_float: "隐藏助理",
            new_todo: "新建任务",
            settings: "设置",
            quit: "退出",
        },
    }
}

pub fn overdue_word(locale: UiLocale) -> &'static str {
    match locale {
        UiLocale::EnUS => "overdue",
        UiLocale::ZhCN => "逾期",
    }
}

pub fn read_ui_locale(app: &AppHandle) -> UiLocale {
    let Some(state) = app.try_state::<AppState>() else {
        return UiLocale::ZhCN;
    };
    SettingsRepository::get(&state.db)
        .map(|s| s.ui_locale)
        .unwrap_or(UiLocale::ZhCN)
}

/// 持有托盘菜单项；助理显隐为单一动态项 float_action
pub struct TrayMenuItems {
    pub float_action: MenuItem<Wry>,
    pub new_todo: MenuItem<Wry>,
    pub settings: MenuItem<Wry>,
    pub quit: MenuItem<Wry>,
}

impl TrayMenuItems {
    pub fn apply_locale(&self, locale: UiLocale) -> Result<(), tauri::Error> {
        let labels = tray_labels(locale);
        self.new_todo.set_text(labels.new_todo)?;
        self.settings.set_text(labels.settings)?;
        self.quit.set_text(labels.quit)?;
        Ok(())
    }

    /// 按助理当前可见性切换「显示助理」/「隐藏助理」文案（互斥一项）
    pub fn sync_float_action(&self, app: &AppHandle) -> Result<(), tauri::Error> {
        let labels = tray_labels(read_ui_locale(app));
        let text = if companion_is_shown(app) {
            labels.hide_float
        } else {
            labels.show_float
        };
        self.float_action.set_text(text)
    }
}

pub fn apply_tray_locale(app: &AppHandle, locale: UiLocale) {
    if let Some(items) = app.try_state::<TrayMenuItems>() {
        if let Err(error) = items.apply_locale(locale) {
            tracing::warn!(error = %error, "failed to update tray menu locale");
        }
        if let Err(error) = items.sync_float_action(app) {
            tracing::warn!(error = %error, "failed to sync tray float action");
        }
    }
    // 只改 tooltip，避免与上方 sync 重复
    crate::float::update_tray_tooltip_text(app);
}

pub fn sync_tray_float_action(app: &AppHandle) {
    if let Some(items) = app.try_state::<TrayMenuItems>() {
        if let Err(error) = items.sync_float_action(app) {
            tracing::warn!(error = %error, "failed to sync tray float action");
        }
    }
}
