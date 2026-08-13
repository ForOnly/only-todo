/**
 * 前端类型入口：契约类型由 Rust（ts-rs）生成；本文件补充 UI 专用类型与常量。
 * 重新生成：`cd src-tauri && cargo run --bin export-ts`
 */
export type {
  BodyView,
  ChromeKind,
  CompanionDragEndResult,
  CompanionPlacement,
  CompanionSession,
  CompanionSurface,
  CompanionVisibility,
  DockEdge,
  EventDto,
  HomeShape,
  PaginatedTodos,
  PanelMode,
  Priority,
  ReminderDto,
  RepeatType,
  SettingsDto,
  StatusActionDto,
  TodoDto,
  TodoStatus,
  UiLocale,
  UiTheme,
  WorkbenchView,
} from "./generated";

import type {
  CreateReminderDto as GeneratedCreateReminderDto,
  CreateTodoDto as GeneratedCreateTodoDto,
  HomeShape,
  Priority,
  RepeatType,
  TodoStatus,
  UiLocale,
  UiTheme,
  WorkbenchView,
} from "./generated";

/** 查询/更新 DTO：前端按需传字段（serde 侧有 default） */
export type CreateTodoDto = Pick<GeneratedCreateTodoDto, "title"> &
  Partial<Omit<GeneratedCreateTodoDto, "title">>;
export type UpdateTodoDto = { id: string } & Partial<{
  title: string;
  description: string;
  priority: Priority;
  dueDate: string | null;
  tags: string[];
}>;
export type ListTodoQuery = Partial<{
  status: TodoStatus[];
  priority: Priority[];
  tags: string[];
  keyword: string;
  dueDateBefore: string;
  dueDateAfter: string;
  includeArchived: boolean;
  includeDeleted: boolean;
  sortBy: string;
  sortOrder: string;
  page: number;
  pageSize: number;
}>;
export type ListWorkbenchQuery = {
  view: WorkbenchView;
  tag?: string;
  keyword?: string;
  sortBy?: string;
  sortOrder?: string;
  page?: number;
  pageSize?: number;
};
export type CreateReminderDto = GeneratedCreateReminderDto;
export type UpdateReminderDto = { id: string } & Partial<{
  remindAt: string;
  enabled: boolean;
}>;
export type UpdateSettingsDto = Partial<{
  notificationEnabled: boolean;
  listDefaultSort: string;
  floatAlwaysOnTop: boolean;
  floatVisibleCount: number;
  floatAutoShow: boolean;
  floatDefaultMode: HomeShape;
  floatHoverPreview: boolean;
  autostartEnabled: boolean;
  uiTheme: UiTheme;
  uiLocale: UiLocale;
}>;
export type ListEventsQuery = Partial<{
  after: string;
  limit: number;
}>;

export type FloatDefaultMode = HomeShape;

export type SortBy =
  | "priority"
  | "createdAt"
  | "updatedAt"
  | "completedAt"
  | "dueDate"
  | "title";
export type SortOrder = "asc" | "desc";
export type DueDateFilter = "all" | "today" | "overdue";

/** settings 表中 list.default_sort 的 JSON 结构 */
export interface ListDefaultSort {
  sortBy: SortBy;
  sortOrder: SortOrder;
}

/** CreateTodoModal 表单模型，字段均有明确默认值 */
export interface CreateTodoFormModel {
  title: string;
  description: string;
  priority: Priority;
  /** datetime-local 字符串；空表示无截止日期 */
  dueDate: string;
  /** 逗号或空格分隔前的原始输入，提交时拆成 tags */
  tagsText: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

/** Tauri Command 返回的结构化错误 */
export interface AppError {
  code: string;
  message: string;
}

export const PRIORITY_OPTIONS: Priority[] = ["Urgent", "High", "Medium", "Low"];
export const REPEAT_TYPE_OPTIONS: RepeatType[] = ["none", "daily", "weekly"];
export const SNOOZE_OPTIONS = [5, 15, 30, 60] as const;

export const DEFAULT_CREATE_TODO_FORM = (): CreateTodoFormModel => ({
  title: "",
  description: "",
  priority: "Medium",
  dueDate: "",
  tagsText: "",
});

export const TITLE_MAX_LENGTH = 200;
export const DESCRIPTION_MAX_LENGTH = 5000;
