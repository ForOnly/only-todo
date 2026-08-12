import type { DueDateFilter, TodoStatus } from "@/api/types";

/** 将 ISO 时间格式化为列表/详情展示用字符串 */
export function formatDate(value: string): string {
  return new Date(value).toLocaleString();
}

/** 将 ISO 时间格式化为通知正文用字符串（月日 + 时分） */
export function formatDateTime(value: string): string {
  const date = new Date(value);
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** 将 ISO 时间格式化为日期展示（不含时间） */
export function formatDueDate(value: string | null): string {
  if (!value) return "—";
  return new Date(value).toLocaleDateString();
}

export function isOverdue(dueDate: string | null, status: TodoStatus): boolean {
  if (!dueDate) return false;
  if (status === "Done" || status === "Archived") return false;
  return new Date(dueDate).getTime() < Date.now();
}

/** datetime-local 输入值 ↔ ISO */
export function toLocalDatetimeInput(value: string | null): string {
  if (!value) return "";
  const date = new Date(value);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function fromLocalDatetimeInput(value: string): string | null {
  if (!value.trim()) return null;
  return new Date(value).toISOString();
}

/** 截止日期筛选 → 查询范围（半开区间 [after, before)，与后端 count 对齐） */
export function dueDateRangeForFilter(filter: DueDateFilter): {
  after?: string;
  before?: string;
} {
  if (filter === "all") return {};

  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const endOfToday = new Date(startOfToday);
  endOfToday.setDate(endOfToday.getDate() + 1);

  if (filter === "today") {
    return {
      after: startOfToday.toISOString(),
      before: endOfToday.toISOString(),
    };
  }

  return {
    before: startOfToday.toISOString(),
  };
}
