//! 进程级外观：偏好在设置，解析在宿主，WebView 只认 set_theme。
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Theme};

use crate::domain::{AppearanceDto, ColorScheme, UiTheme};
use crate::state::AppState;

pub struct AppearanceHost {
    inner: Mutex<Snapshot>,
}

#[derive(Clone, Copy)]
struct Snapshot {
    preference: UiTheme,
    os: ColorScheme,
    resolved: ColorScheme,
}

impl AppearanceHost {
    pub fn new(preference: UiTheme, os: ColorScheme) -> Self {
        Self {
            inner: Mutex::new(Snapshot {
                preference,
                os,
                resolved: preference.resolve(os),
            }),
        }
    }

    pub fn snapshot(&self) -> AppearanceDto {
        let snap = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        AppearanceDto {
            preference: snap.preference,
            resolved: snap.resolved,
        }
    }
}

pub fn query_os_theme(app: &AppHandle, reported: Option<Theme>) -> ColorScheme {
    #[cfg(windows)]
    {
        if let Some(scheme) = os_theme_from_registry() {
            return scheme;
        }
    }
    if let Some(theme) = reported {
        return color_scheme_from_tauri(theme);
    }
    // set_theme 之后 window.theme() 是钉死值，不能当 OS
    if let Some(host) = app.try_state::<AppearanceHost>() {
        return host.inner.lock().unwrap_or_else(|e| e.into_inner()).os;
    }
    app.get_webview_window("main")
        .and_then(|window| window.theme().ok())
        .map(color_scheme_from_tauri)
        .unwrap_or(ColorScheme::Light)
}

/// 按当前偏好重新解析并钉到所有窗。resolved 未变则不调 set_theme，避免 ThemeChanged 回环。
pub fn sync(app: &AppHandle, preference: UiTheme, reported_os: Option<Theme>) {
    let os = query_os_theme(app, reported_os);
    let resolved = preference.resolve(os);
    let Some(host) = app.try_state::<AppearanceHost>() else {
        apply_theme_to_all_windows(app, resolved);
        return;
    };
    let unchanged = {
        let mut snap = host.inner.lock().unwrap_or_else(|e| e.into_inner());
        let same = snap.preference == preference && snap.os == os && snap.resolved == resolved;
        snap.preference = preference;
        snap.os = os;
        snap.resolved = resolved;
        same
    };
    if unchanged {
        return;
    }
    apply_theme_to_all_windows(app, resolved);
}

/// 启动或设置变更：用库里的偏好同步。
pub fn sync_from_settings(app: &AppHandle) {
    let preference = app
        .try_state::<AppState>()
        .and_then(|state| crate::services::settings_service::SettingsService::get(&state.db).ok())
        .map(|settings| settings.ui_theme)
        .unwrap_or(UiTheme::System);
    sync(app, preference, None);
}

/// OS / 引擎 ThemeChanged：system 跟 OS 重算；钉死偏好则把漂了的窗再钉回去。
pub fn on_os_theme_signal(app: &AppHandle, reported: Theme) {
    let preference = app
        .try_state::<AppearanceHost>()
        .map(|host| {
            host.inner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .preference
        })
        .unwrap_or(UiTheme::System);
    sync(app, preference, Some(reported));
}

/// 新窗 show 之前钉 theme。漏窗 = 未出现在 `webview_windows()` 或创建后未调用本函数。
pub fn pin_window_theme(window: &tauri::WebviewWindow, scheme: ColorScheme) {
    let want = theme_from_color_scheme(scheme);
    match window.theme() {
        Ok(current) if current == want => {}
        _ => {
            if let Err(error) = window.set_theme(Some(want)) {
                tracing::warn!(
                    error = %error,
                    label = window.label(),
                    "failed to set window theme"
                );
            }
        }
    }
}

pub fn apply_theme_to_all_windows(app: &AppHandle, scheme: ColorScheme) {
    for window in app.webview_windows().into_values() {
        pin_window_theme(&window, scheme);
    }
}

fn theme_from_color_scheme(scheme: ColorScheme) -> Theme {
    match scheme {
        ColorScheme::Light => Theme::Light,
        ColorScheme::Dark => Theme::Dark,
    }
}

fn color_scheme_from_tauri(theme: Theme) -> ColorScheme {
    match theme {
        Theme::Light => ColorScheme::Light,
        Theme::Dark => ColorScheme::Dark,
        _ => ColorScheme::Light,
    }
}

#[cfg(windows)]
fn os_theme_from_registry() -> Option<ColorScheme> {
    use windows::core::w;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

    let mut data: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut data as *mut u32).cast()),
            Some(&mut size),
        )
    };
    if status.is_ok() {
        Some(if data == 0 {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_ignore_os() {
        assert_eq!(
            UiTheme::Light.resolve(ColorScheme::Dark),
            ColorScheme::Light
        );
        assert_eq!(UiTheme::Dark.resolve(ColorScheme::Light), ColorScheme::Dark);
    }

    #[test]
    fn system_follows_os() {
        assert_eq!(
            UiTheme::System.resolve(ColorScheme::Dark),
            ColorScheme::Dark
        );
        assert_eq!(
            UiTheme::System.resolve(ColorScheme::Light),
            ColorScheme::Light
        );
    }
}
