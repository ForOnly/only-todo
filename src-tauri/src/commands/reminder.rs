use tauri::State;

use crate::domain::reminder::{
    CreateReminderDto, ReminderDto, UpdateReminderDto,
};
use crate::errors::AppError;
use crate::services::reminder_service::ReminderService;
use crate::state::AppState;

#[tauri::command]
pub fn create_reminder(
    state: State<AppState>,
    dto: CreateReminderDto,
) -> Result<ReminderDto, AppError> {
    ReminderService::create(&state.db, dto)
}

#[tauri::command]
pub fn update_reminder(
    state: State<AppState>,
    dto: UpdateReminderDto,
) -> Result<ReminderDto, AppError> {
    ReminderService::update(&state.db, dto)
}

#[tauri::command]
pub fn delete_reminder(state: State<AppState>, id: String) -> Result<(), AppError> {
    ReminderService::delete(&state.db, &id)
}

#[tauri::command]
pub fn list_reminders(
    state: State<AppState>,
    todo_id: String,
) -> Result<Vec<ReminderDto>, AppError> {
    ReminderService::list(&state.db, &todo_id)
}
