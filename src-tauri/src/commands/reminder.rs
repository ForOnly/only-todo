use tauri::{AppHandle, State};

use crate::broadcast;
use crate::domain::reminder::{CreateReminderDto, ReminderDto, UpdateReminderDto};
use crate::errors::AppError;
use crate::services::reminder_service::ReminderService;
use crate::state::AppState;

#[tauri::command]
pub fn create_reminder(
    app: AppHandle,
    state: State<AppState>,
    dto: CreateReminderDto,
) -> Result<ReminderDto, AppError> {
    let reminder = ReminderService::create(&state.db, dto)?;
    broadcast::emit_todos_changed(&app);
    Ok(reminder)
}

#[tauri::command]
pub fn update_reminder(
    app: AppHandle,
    state: State<AppState>,
    dto: UpdateReminderDto,
) -> Result<ReminderDto, AppError> {
    let reminder = ReminderService::update(&state.db, dto)?;
    broadcast::emit_todos_changed(&app);
    Ok(reminder)
}

#[tauri::command]
pub fn delete_reminder(app: AppHandle, state: State<AppState>, id: String) -> Result<(), AppError> {
    ReminderService::delete(&state.db, &id)?;
    broadcast::emit_todos_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn list_reminders(
    state: State<AppState>,
    todo_id: String,
) -> Result<Vec<ReminderDto>, AppError> {
    ReminderService::list(&state.db, &todo_id)
}

#[tauri::command]
pub fn snooze_reminder(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    minutes: i64,
) -> Result<ReminderDto, AppError> {
    let reminder = ReminderService::snooze(&state.db, &id, minutes)?;
    broadcast::emit_todos_changed(&app);
    Ok(reminder)
}
