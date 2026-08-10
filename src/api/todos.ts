import { invoke } from "@tauri-apps/api/core";

import type {
  CreateTodoDto,
  ListTodoQuery,
  PaginatedResponse,
  TodoDto,
  TodoStatus,
  UpdateTodoDto,
} from "@/api/types";

/** Tauri Command: create_todo */
export async function createTodo(dto: CreateTodoDto): Promise<TodoDto> {
  return invoke("create_todo", { dto });
}

/** Tauri Command: update_todo */
export async function updateTodo(dto: UpdateTodoDto): Promise<TodoDto> {
  return invoke("update_todo", { dto });
}

/** Tauri Command: delete_todo（软删除） */
export async function deleteTodo(id: string): Promise<void> {
  return invoke("delete_todo", { id });
}

/** Tauri Command: restore_todo */
export async function restoreTodo(id: string): Promise<TodoDto> {
  return invoke("restore_todo", { id });
}

/** Tauri Command: get_todo */
export async function getTodo(id: string): Promise<TodoDto> {
  return invoke("get_todo", { id });
}

/** Tauri Command: list_todos */
export async function listTodos(
  query: ListTodoQuery = {},
): Promise<PaginatedResponse<TodoDto>> {
  return invoke("list_todos", { query });
}

/** Tauri Command: transition_todo */
export async function transitionTodo(
  id: string,
  status: TodoStatus,
): Promise<TodoDto> {
  return invoke("transition_todo", { id, status });
}

/** Tauri Command: show_main_window */
export async function showMainWindow(todoId?: string): Promise<void> {
  return invoke("show_main_window", { todoId: todoId ?? null });
}
