use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::domain::{AppearanceDto, SettingsDto, UpdateSettingsDto};
use crate::errors::AppError;
use crate::float::host::FloatHost;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<SettingsDto, AppError> {
    SettingsService::get(&state.db)
}

#[tauri::command]
pub fn get_appearance(app: AppHandle) -> AppearanceDto {
    app.try_state::<crate::appearance::AppearanceHost>()
        .map(|host| host.snapshot())
        .unwrap_or_else(|| {
            let preference = app
                .try_state::<AppState>()
                .and_then(|state| SettingsService::get(&state.db).ok())
                .map(|settings| settings.ui_theme)
                .unwrap_or(crate::domain::UiTheme::System);
            let os = crate::appearance::query_os_theme(&app, None);
            AppearanceDto {
                preference,
                resolved: preference.resolve(os),
            }
        })
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    dto: UpdateSettingsDto,
) -> Result<SettingsDto, AppError> {
    let previous = SettingsService::get(&state.db)?;
    let autostart_changed = dto
        .autostart_enabled
        .is_some_and(|enabled| enabled != previous.autostart_enabled);
    let locale_changed = dto
        .ui_locale
        .is_some_and(|locale| locale != previous.ui_locale);

    // 先落库，避免自启 OS API 失败拖垮通知/助理等其它设置
    let settings = SettingsService::update(&state.db, dto)?;

    if autostart_changed {
        sync_autostart_os(&app, settings.autostart_enabled);
    }

    if locale_changed {
        crate::tray_i18n::apply_tray_locale(&app, settings.ui_locale);
    }

    let home_changed = settings.float_default_mode != previous.float_default_mode;
    let _ = FloatHost::on_settings_updated(&app, home_changed);
    crate::appearance::sync_from_settings(&app);
    let _ = app.emit("settings-updated", &settings);
    Ok(settings)
}

/// 同步开机自启到 OS；失败只记日志，不回滚已保存偏好（开发态路径常不可注册）。
fn sync_autostart_os(app: &AppHandle, enabled: bool) {
    let autostart = app.autolaunch();
    let result = if enabled {
        autostart.enable()
    } else {
        autostart.disable()
    };

    if let Err(error) = result {
        let message = error.to_string();
        // Windows 无启动项时 disable 常报「找不到文件」——视为已关闭
        if !enabled && is_missing_autostart_entry(&message) {
            tracing::debug!("autostart disable: no existing entry ({message})");
            return;
        }
        tracing::warn!(
            enabled,
            error = %message,
            "failed to sync OS autostart; preference already saved"
        );
    }
}

fn is_missing_autostart_entry(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("os error 2") || lower.contains("not found") || message.contains("找不到")
}
