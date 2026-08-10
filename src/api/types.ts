export type TodoStatus = "Todo" | "Doing" | "Done" | "Archived";
export type Priority = "Low" | "Medium" | "High" | "Urgent";
export type RepeatType = "none" | "daily" | "weekly";
export type SortBy =
  | "priority"
  | "createdAt"
  | "updatedAt"
  | "completedAt"
  | "title";
export type SortOrder = "asc" | "desc";

/** settings 表中 list.default_sort 的 JSON 结构 */
export interface ListDefaultSort {
  sortBy: SortBy;
  sortOrder: SortOrder;
}

export interface TodoDto {
  id: string;
  title: string;
  description: string;
  status: TodoStatus;
  priority: Priority;
  createdAt: string;
  updatedAt: string;
  completedAt: string | null;
  deletedAt: string | null;
}

export interface CreateTodoDto {
  title: string;
  description?: string;
  priority?: Priority;
}

/** CreateTodoModal 表单模型，字段均有明确默认值 */
export interface CreateTodoFormModel {
  title: string;
  description: string;
  priority: Priority;
}

export interface UpdateTodoDto {
  id: string;
  title?: string;
  description?: string;
  priority?: Priority;
}

export interface ListTodoQuery {
  status?: TodoStatus[];
  priority?: Priority[];
  keyword?: string;
  sortBy?: SortBy;
  sortOrder?: SortOrder;
  page?: number;
  pageSize?: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

export interface ReminderDto {
  id: string;
  todoId: string;
  remindAt: string;
  repeatType: RepeatType;
  repeatConfig: string;
  snoozeCount: number;
  nextTriggerAt: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateReminderDto {
  todoId: string;
  remindAt: string;
  repeatType?: RepeatType;
}

export interface UpdateReminderDto {
  id: string;
  remindAt?: string;
  enabled?: boolean;
}

export interface SettingsDto {
  notificationEnabled: boolean;
  listDefaultSort: string;
}

export interface UpdateSettingsDto {
  notificationEnabled?: boolean;
  listDefaultSort?: string;
}

/** Tauri Command 返回的结构化错误 */
export interface AppError {
  code: string;
  message: string;
}

export const PRIORITY_OPTIONS: Priority[] = ["Urgent", "High", "Medium", "Low"];
export const STATUS_OPTIONS: TodoStatus[] = ["Todo", "Done"];

export const PRIORITY_LABELS: Record<Priority, string> = {
  Urgent: "紧急",
  High: "高",
  Medium: "中",
  Low: "低",
};

export const STATUS_LABELS: Record<TodoStatus, string> = {
  Todo: "待办",
  Doing: "进行中",
  Done: "已完成",
  Archived: "已归档",
};

export const DEFAULT_CREATE_TODO_FORM = (): CreateTodoFormModel => ({
  title: "",
  description: "",
  priority: "Medium",
});

export const TITLE_MAX_LENGTH = 200;
export const DESCRIPTION_MAX_LENGTH = 5000;
