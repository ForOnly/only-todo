use tauri::State;

use crate::errors::AppError;
use crate::events::{list_events, EventDto, ListEventsQuery};
use crate::state::AppState;

#[tauri::command]
pub fn list_events_cmd(
    state: State<AppState>,
    query: ListEventsQuery,
) -> Result<Vec<EventDto>, AppError> {
    list_events(&state.db, query)
}
