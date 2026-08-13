//! 跨窗数据变更广播（主窗 / 伴侣 / 托盘）

use tauri::{AppHandle, Emitter};

/// 任务或提醒写成功后调用：通知各 WebView 刷新，并更新托盘 tooltip。
pub fn emit_todos_changed(app: &AppHandle) {
    let _ = app.emit("todos-changed", ());
    crate::float::update_tray_tooltip(app);
}
