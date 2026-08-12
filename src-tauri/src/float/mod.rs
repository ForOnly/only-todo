pub mod geometry;
pub mod host;
pub mod surfaces;

pub use host::{snapshot, FloatHost};

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
        let _ = host::update_chrome_click_through(&app);
    });
}

pub fn update_tray_tooltip(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let count = TodoService::count_active(&state.db).unwrap_or(0);
    let overdue = TodoService::count_due_urgency(&state.db)
        .map(|(overdue, _)| overdue)
        .unwrap_or(0);
    let tooltip = if overdue > 0 {
        format!("Only Todo ({count}) · {overdue} 逾期")
    } else {
        format!("Only Todo ({count})")
    };
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}
