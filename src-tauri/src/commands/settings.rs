use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::commands::window::{
    apply_float_mode_from_settings, FLOAT_LABEL,
};
use crate::domain::{FloatDisplayMode, SettingsDto, UpdateSettingsDto};
use crate::errors::AppError;
use crate::repository::settings_repository::SettingsRepository;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<SettingsDto, AppError> {
    SettingsService::get(&state.db)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    dto: UpdateSettingsDto,
) -> Result<SettingsDto, AppError> {
    let default_mode_changed = dto.float_default_mode.is_some();

    if let Some(enabled) = dto.autostart_enabled {
        let autostart = app.autolaunch();
        if enabled {
            autostart.enable().map_err(|error| AppError::InternalError {
                message: error.to_string(),
            })?;
        } else {
            autostart.disable().map_err(|error| AppError::InternalError {
                message: error.to_string(),
            })?;
        }
    }

    let settings = SettingsService::update(&state.db, dto)?;

    // 未挂边时立即切到新主形态；已挂边只更新 default_mode，不改 HWND / unpeek
    if default_mode_changed {
        let current = SettingsRepository::get_display_mode(&state.db)?;
        if current != FloatDisplayMode::Docked {
            let home = SettingsRepository::get_home_mode(&state.db)?;
            SettingsRepository::set_display_mode(&state.db, home)?;
            if let Some(window) = app.get_webview_window(FLOAT_LABEL) {
                let _ = apply_float_mode_from_settings(&app, &window, &state.db);
            }
        }
    }

    if let Some(window) = app.get_webview_window(FLOAT_LABEL) {
        window.set_always_on_top(settings.float_always_on_top).ok();
    }

    let _ = app.emit("settings-updated", &settings);

    Ok(settings)
}
