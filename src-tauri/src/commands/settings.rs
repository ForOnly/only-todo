use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;

use crate::domain::{SettingsDto, UpdateSettingsDto};
use crate::errors::AppError;
use crate::float::host::FloatHost;
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
    if let Some(enabled) = dto.autostart_enabled {
        let autostart = app.autolaunch();
        if enabled {
            autostart
                .enable()
                .map_err(|error| AppError::InternalError {
                    message: error.to_string(),
                })?;
        } else {
            autostart
                .disable()
                .map_err(|error| AppError::InternalError {
                    message: error.to_string(),
                })?;
        }
    }

    let previous_home = SettingsRepository::get_home_shape(&state.db)?;
    let settings = SettingsService::update(&state.db, dto)?;
    let home_changed = settings.float_default_mode != previous_home;
    let _ = FloatHost::on_settings_updated(&app, home_changed);
    let _ = app.emit("settings-updated", &settings);
    Ok(settings)
}
