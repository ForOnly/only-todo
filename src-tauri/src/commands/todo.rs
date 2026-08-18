use tauri::{AppHandle, State};

use crate::broadcast;
use crate::domain::{
    status::{StatusActionDto, TodoStatus},
    todo::{
        CreateTodoDto, FocusBoardDto, ListFocusBoardQuery, ListTodoQuery, ListWorkbenchQuery,
        PaginatedResponse, TodoDto, UpdateTodoDto,
    },
};
use crate::errors::AppError;
use crate::services::todo_service::TodoService;
use crate::state::AppState;

#[tauri::command]
pub fn create_todo(
    app: AppHandle,
    state: State<AppState>,
    dto: CreateTodoDto,
) -> Result<TodoDto, AppError> {
    let todo = TodoService::create(&state.db, dto)?;
    broadcast::emit_todos_changed(&app);
    Ok(todo)
}

#[tauri::command]
pub fn update_todo(
    app: AppHandle,
    state: State<AppState>,
    dto: UpdateTodoDto,
) -> Result<TodoDto, AppError> {
    let todo = TodoService::update(&state.db, dto)?;
    broadcast::emit_todos_changed(&app);
    Ok(todo)
}

#[tauri::command]
pub fn delete_todo(app: AppHandle, state: State<AppState>, id: String) -> Result<(), AppError> {
    TodoService::delete(&state.db, &id)?;
    broadcast::emit_todos_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn restore_todo(
    app: AppHandle,
    state: State<AppState>,
    id: String,
) -> Result<TodoDto, AppError> {
    let todo = TodoService::restore(&state.db, &id)?;
    broadcast::emit_todos_changed(&app);
    Ok(todo)
}

#[tauri::command]
pub fn get_todo(state: State<AppState>, id: String) -> Result<TodoDto, AppError> {
    TodoService::get(&state.db, &id)
}

#[tauri::command]
pub fn list_todos(
    state: State<AppState>,
    query: ListTodoQuery,
) -> Result<PaginatedResponse<TodoDto>, AppError> {
    TodoService::list(&state.db, query)
}

#[tauri::command]
pub fn list_workbench_todos(
    state: State<AppState>,
    query: ListWorkbenchQuery,
) -> Result<PaginatedResponse<TodoDto>, AppError> {
    TodoService::list_workbench(&state.db, query)
}

#[tauri::command]
pub fn list_focus_board(
    state: State<AppState>,
    query: ListFocusBoardQuery,
) -> Result<FocusBoardDto, AppError> {
    TodoService::list_focus_board(&state.db, query)
}

#[tauri::command]
pub fn list_all_tags(state: State<AppState>) -> Result<Vec<String>, AppError> {
    TodoService::list_all_tags(&state.db)
}

#[tauri::command]
pub fn get_allowed_transitions(status: TodoStatus) -> Vec<StatusActionDto> {
    TodoService::allowed_transitions(status)
}

#[tauri::command]
pub fn transition_todo(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    status: TodoStatus,
) -> Result<TodoDto, AppError> {
    let todo = TodoService::transition(&state.db, &id, status)?;
    broadcast::emit_todos_changed(&app);
    Ok(todo)
}
