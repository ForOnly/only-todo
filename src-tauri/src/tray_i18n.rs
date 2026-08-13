//! 托盘菜单与 tooltip 文案（随 ui.locale）
use tauri::menu::MenuItem;
use tauri::AppHandle;
use tauri::Manager;
use tauri::Wry;

use crate::domain::UiLocale;
use crate::repository::settings_repository::SettingsRepository;
use crate::state::AppState;

pub struct TrayLabels {
    pub show_float: &'static str,
    pub show_main: &'static str,
    pub new_todo: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn tray_labels(locale: UiLocale) -> TrayLabels {
    match locale {
        UiLocale::EnUS => TrayLabels {
            show_float: "Show companion",
            show_main: "Open main window",
            new_todo: "New task",
            settings: "Settings",
            quit: "Quit",
        },
        UiLocale::ZhCN => TrayLabels {
            show_float: "显示伴侣",
            show_main: "打开主窗口",
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

/// 持有托盘菜单项，便于 locale 变更时改文案
pub struct TrayMenuItems {
    pub show_float: MenuItem<Wry>,
    pub show_main: MenuItem<Wry>,
    pub new_todo: MenuItem<Wry>,
    pub settings: MenuItem<Wry>,
    pub quit: MenuItem<Wry>,
}

impl TrayMenuItems {
    pub fn apply_locale(&self, locale: UiLocale) -> Result<(), tauri::Error> {
        let labels = tray_labels(locale);
        self.show_float.set_text(labels.show_float)?;
        self.show_main.set_text(labels.show_main)?;
        self.new_todo.set_text(labels.new_todo)?;
        self.settings.set_text(labels.settings)?;
        self.quit.set_text(labels.quit)?;
        Ok(())
    }
}

pub fn apply_tray_locale(app: &AppHandle, locale: UiLocale) {
    if let Some(items) = app.try_state::<TrayMenuItems>() {
        if let Err(error) = items.apply_locale(locale) {
            tracing::warn!(error = %error, "failed to update tray menu locale");
        }
    }
    crate::float::update_tray_tooltip(app);
}
