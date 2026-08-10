use tauri::State;

use crate::domain::{
    status::TodoStatus,
    todo::{
        CreateTodoDto, ListTodoQuery, PaginatedResponse, TodoDto, UpdateTodoDto,
    },
};
use crate::errors::AppError;
use crate::services::todo_service::TodoService;
use crate::state::AppState;

#[tauri::command]
pub fn create_todo(state: State<AppState>, dto: CreateTodoDto) -> Result<TodoDto, AppError> {
    TodoService::create(&state.db, dto)
}

#[tauri::command]
pub fn update_todo(state: State<AppState>, dto: UpdateTodoDto) -> Result<TodoDto, AppError> {
    TodoService::update(&state.db, dto)
}

#[tauri::command]
pub fn delete_todo(state: State<AppState>, id: String) -> Result<(), AppError> {
    TodoService::delete(&state.db, &id)
}

#[tauri::command]
pub fn restore_todo(state: State<AppState>, id: String) -> Result<TodoDto, AppError> {
    TodoService::restore(&state.db, &id)
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
pub fn transition_todo(
    state: State<AppState>,
    id: String,
    status: TodoStatus,
) -> Result<TodoDto, AppError> {
    TodoService::transition(&state.db, &id, status)
}
