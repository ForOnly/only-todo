use tauri::State;

use crate::domain::{SettingsDto, UpdateSettingsDto};
use crate::errors::AppError;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<SettingsDto, AppError> {
    SettingsService::get(&state.db)
}

#[tauri::command]
pub fn update_settings(
    state: State<AppState>,
    dto: UpdateSettingsDto,
) -> Result<SettingsDto, AppError> {
    SettingsService::update(&state.db, dto)
}
