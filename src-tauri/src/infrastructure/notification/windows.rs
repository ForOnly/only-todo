//! Windows toast 注册 — dev 模式下必须设置，否则通知可能静默失败。
//!
//! 未设置 AppUserModelID 时，Windows 会将 toast 路由到 PowerShell 或在 debug 构建中直接丢弃。

#[cfg(windows)]
pub fn ensure_toast_registration() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    const APP_ID: &str = "com.only-todo.app";
    let wide: Vec<u16> = OsStr::new(APP_ID)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        if let Err(error) = SetCurrentProcessExplicitAppUserModelID(PCWSTR(wide.as_ptr())) {
            tracing::warn!("设置 AppUserModelID 失败: {error}");
        }
    }
}

#[cfg(not(windows))]
pub fn ensure_toast_registration() {}
