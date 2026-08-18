import { invoke } from "@tauri-apps/api/core";

import type {
  CreateTodoDto,
  FocusBoardDto,
  ListFocusBoardQuery,
  ListTodoQuery,
  ListWorkbenchQuery,
  PaginatedTodos,
  StatusActionDto,
  TodoDto,
  TodoStatus,
  UpdateTodoDto,
  WorkbenchView,
} from "@/api/types";

/** 兼容旧 PaginatedResponse 形状 */
export type PaginatedResponse<T> = {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
};

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
export async function listTodos(query: ListTodoQuery = {}): Promise<PaginatedResponse<TodoDto>> {
  return invoke("list_todos", { query });
}

/** Tauri Command: list_workbench_todos */
export async function listWorkbenchTodos(
  query: ListWorkbenchQuery,
): Promise<PaginatedResponse<TodoDto>> {
  const result = await invoke<PaginatedTodos>("list_workbench_todos", { query });
  return {
    items: result.items,
    total: result.total,
    page: result.page,
    pageSize: result.pageSize,
  };
}

/** Tauri Command: list_focus_board */
export async function listFocusBoard(
  query: Partial<ListFocusBoardQuery> = {},
): Promise<FocusBoardDto> {
  return invoke("list_focus_board", {
    query: {
      sortBy: query.sortBy ?? "priority",
      sortOrder: query.sortOrder ?? "desc",
    },
  });
}

/** Tauri Command: transition_todo */
export async function transitionTodo(id: string, status: TodoStatus): Promise<TodoDto> {
  return invoke("transition_todo", { id, status });
}

/** Tauri Command: get_allowed_transitions */
export async function getAllowedTransitions(status: TodoStatus): Promise<StatusActionDto[]> {
  return invoke("get_allowed_transitions", { status });
}

/** Tauri Command: show_main_window */
export async function showMainWindow(todoId?: string): Promise<void> {
  return invoke("show_main_window", { todoId: todoId ?? null });
}

/** Tauri Command: list_all_tags */
export async function listAllTags(): Promise<string[]> {
  return invoke("list_all_tags");
}

export type { WorkbenchView };
