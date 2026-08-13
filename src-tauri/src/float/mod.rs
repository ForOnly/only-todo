pub mod geometry;
pub mod surfaces;
pub mod session;
pub mod persistence;
pub mod window_ops;
pub mod host;

pub use host::FloatHost;
pub use session::snapshot;
use tauri::Manager;

use crate::services::todo_service::TodoService;
use crate::state::AppState;

pub const CHROME_LABEL: &str = "float-chrome";
pub const BODY_LABEL: &str = "float-body";
pub const SESSION_EVENT: &str = "float-session-changed";

/// chrome 透明区点击穿透：轮询光标是否落在圆/条视觉区内
pub fn start_chrome_hit_test(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(32));
        let _ = window_ops::update_chrome_click_through(&app);
    });
}

pub fn update_tray_tooltip(app: &tauri::AppHandle) {
    update_tray_tooltip_text(app);
    crate::tray_i18n::sync_tray_float_action(app);
}

/// 仅更新 tooltip 文案（locale 路径已单独 sync 菜单时用）
pub fn update_tray_tooltip_text(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let count = TodoService::count_active(&state.db).unwrap_or(0);
    let overdue = TodoService::count_due_urgency(&state.db)
        .map(|(overdue, _)| overdue)
        .unwrap_or(0);
    let locale = crate::tray_i18n::read_ui_locale(app);
    let overdue_word = crate::tray_i18n::overdue_word(locale);
    let tooltip = if overdue > 0 {
        format!("Only Todo ({count}) · {overdue} {overdue_word}")
    } else {
        format!("Only Todo ({count})")
    };
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}
