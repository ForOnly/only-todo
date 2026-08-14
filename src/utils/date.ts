import type { DueDateFilter, TodoStatus } from "@/api/types";
import { i18n } from "@/i18n";

function currentLocale(): string {
  return i18n.global.locale.value;
}

/** 将 ISO 时间格式化为列表/详情展示用字符串 */
export function formatDate(value: string): string {
  return new Date(value).toLocaleString(currentLocale());
}

/** 将 ISO 时间格式化为通知正文用字符串（月日 + 时分） */
export function formatDateTime(value: string): string {
  const date = new Date(value);
  return date.toLocaleString(currentLocale(), {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** 将 ISO 时间格式化为日期展示（不含时间） */
export function formatDueDate(value: string | null): string {
  if (!value) return "—";
  return new Date(value).toLocaleDateString(currentLocale());
}

export function isOverdue(dueDate: string | null, status: TodoStatus): boolean {
  if (!dueDate) return false;
  if (status === "Done" || status === "Archived") return false;
  return new Date(dueDate).getTime() < Date.now();
}

/** datetime-local 输入值 ↔ ISO */
export function toLocalDatetimeInput(iso: string | null | undefined): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function fromLocalDatetimeInput(local: string): string | null {
  if (!local.trim()) return null;
  const d = new Date(local);
  if (Number.isNaN(d.getTime())) return null;
  return d.toISOString();
}

/** 今日截止用的 datetime-local（本地日末），保证落在「今日」视图区间内 */
export function localTodayDueInput(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T23:59`;
}

/** 截止日期筛选 → 查询范围（半开区间 [after, before)，与后端 count 对齐） */
export function dueDateRangeForFilter(
  filter: DueDateFilter,
): { after?: string; before?: string } {
  if (filter === "all") return {};
  const now = new Date();
  const start = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const end = new Date(start);
  end.setDate(end.getDate() + 1);
  if (filter === "today") {
    return { after: start.toISOString(), before: end.toISOString() };
  }
  // 逾期：截止于今日零点之前
  return { before: start.toISOString() };
}
