import type { Composer } from "vue-i18n";

import type { TodoDto, TodoStatus } from "@/api/types";
import { i18n } from "@/i18n";

function currentLocale(): string {
  return i18n.global.locale.value;
}

function parseMs(iso: string | null | undefined): number | null {
  if (!iso) return null;
  const ms = Date.parse(iso);
  return Number.isNaN(ms) ? null : ms;
}

/** 本地今日 0 点（即将到期等本地日历语义） */
export function startOfLocalDayMs(nowMs: number = Date.now()): number {
  const d = new Date(nowMs);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

/** UTC 今日 0 点，与后端 list_focus_board 的 date_naive()+00:00:00 UTC 一致 */
export function startOfUtcDayMs(nowMs: number = Date.now()): number {
  const d = new Date(nowMs);
  return Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate());
}

export function isOverdueByDay(
  dueDate: string | null | undefined,
  status: TodoStatus,
  nowMs: number = Date.now(),
): boolean {
  if (!dueDate) return false;
  if (status === "Done" || status === "Archived") return false;
  const dueMs = parseMs(dueDate);
  if (dueMs === null) return false;
  return dueMs < startOfUtcDayMs(nowMs);
}

function hasTimeComponent(iso: string): boolean {
  const d = new Date(iso);
  return d.getHours() !== 0 || d.getMinutes() !== 0;
}

/** 绝对时间：有具体时刻则带时分，否则仅日期 */
export function formatAbsoluteDateTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  if (hasTimeComponent(iso)) {
    return d.toLocaleString(currentLocale(), {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  return d.toLocaleDateString(currentLocale());
}

function dayDiffFromToday(isoMs: number, nowMs: number): number {
  const startToday = startOfLocalDayMs(nowMs);
  const startTarget = startOfLocalDayMs(isoMs);
  return Math.round((startTarget - startToday) / 86_400_000);
}

function utcDayDiffFromToday(isoMs: number, nowMs: number): number {
  const startToday = startOfUtcDayMs(nowMs);
  const startTarget = startOfUtcDayMs(isoMs);
  return Math.round((startTarget - startToday) / 86_400_000);
}

/** 截止相对语义：今天/明天/逾期 N 天 */
export function formatRelativeDue(
  dueDate: string | null | undefined,
  status: TodoStatus,
  t: Composer["t"],
  nowMs: number = Date.now(),
): string | null {
  if (!dueDate) return null;
  const dueMs = parseMs(dueDate);
  if (dueMs === null) return null;

  if (isOverdueByDay(dueDate, status, nowMs)) {
    const days = Math.max(1, Math.abs(utcDayDiffFromToday(dueMs, nowMs)));
    return t("timeMeta.overdueDays", { n: days });
  }

  const diff = dayDiffFromToday(dueMs, nowMs);
  if (diff === 0) {
    if (hasTimeComponent(dueDate)) {
      const time = new Date(dueMs).toLocaleTimeString(currentLocale(), {
        hour: "2-digit",
        minute: "2-digit",
      });
      return t("timeMeta.dueTodayAt", { time });
    }
    return t("timeMeta.dueToday");
  }
  if (diff === 1) return t("timeMeta.dueTomorrow");
  if (diff > 1 && diff <= 7) return t("timeMeta.dueInDays", { n: diff });
  return formatAbsoluteDateTime(dueDate);
}

/** 逾期多久（仅逾期任务） */
export function formatRelativeOverdue(
  dueDate: string | null | undefined,
  status: TodoStatus,
  t: Composer["t"],
  nowMs: number = Date.now(),
): string | null {
  if (!isOverdueByDay(dueDate, status, nowMs)) return null;
  const dueMs = parseMs(dueDate);
  if (dueMs === null) return null;
  const days = Math.max(1, Math.abs(utcDayDiffFromToday(dueMs, nowMs)));
  return t("timeMeta.overdueDays", { n: days });
}

/** 逾期天数（UTC 日历差，至少 1）：用于「n天前逾期」 */
export function getOverdueDaysOnly(
  dueDate: string | null | undefined,
  status: TodoStatus,
  nowMs: number = Date.now(),
): number | null {
  if (!isOverdueByDay(dueDate, status, nowMs)) return null;
  const dueMs = parseMs(dueDate);
  if (dueMs === null) return null;
  return Math.max(1, Math.abs(utcDayDiffFromToday(dueMs, nowMs)));
}

/** 任务年龄 / 多久未更新 */
export function formatRelativeAge(
  iso: string | null | undefined,
  t: Composer["t"],
  nowMs: number = Date.now(),
): string | null {
  const then = parseMs(iso);
  if (then === null) return null;
  const diffSec = Math.max(0, Math.floor((nowMs - then) / 1000));
  if (diffSec < 60) return t("timeMeta.justNow");
  const mins = Math.floor(diffSec / 60);
  if (mins < 60) return t("timeMeta.minutesAgo", { n: mins });
  const hours = Math.floor(mins / 60);
  if (hours < 48) return t("timeMeta.hoursAgo", { n: hours });
  const days = Math.floor(hours / 24);
  return t("timeMeta.daysAgo", { n: days });
}

/** 列表行时间 token：仅在有截止日期时展示；无 dueDate 返回 null */
export function formatListTimeToken(
  todo: Pick<TodoDto, "dueDate" | "status">,
  t: Composer["t"],
  nowMs: number = Date.now(),
): string | null {
  return formatRelativeDue(todo.dueDate, todo.status, t, nowMs);
}

