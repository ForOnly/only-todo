import type { Composer } from "vue-i18n";

import type { EventDto, TodoStatus } from "@/api/types";

/** 相对时间（活动条等） */
export function formatRelativeTime(
  iso: string,
  t: Composer["t"],
  nowMs: number = Date.now(),
): string {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return iso;
  const diffSec = Math.max(0, Math.floor((nowMs - then) / 1000));
  if (diffSec < 60) return t("activity.relativeJustNow");
  const mins = Math.floor(diffSec / 60);
  if (mins < 60) return t("activity.relativeMinutes", { n: mins });
  const hours = Math.floor(mins / 60);
  if (hours < 48) return t("activity.relativeHours", { n: hours });
  const days = Math.floor(hours / 24);
  return t("activity.relativeDays", { n: days });
}

/** eventType 形如 todo.created → activity.types.todo.created */
export function translateEventType(eventType: string, t: Composer["t"]): string {
  const key = `activity.types.${eventType}`;
  const translated = t(key);
  return translated === key ? eventType : translated;
}

type ActivityPayload = {
  title?: unknown;
  to?: unknown;
  minutes?: unknown;
};

function parseActivityPayload(raw: string): ActivityPayload {
  try {
    const parsed: unknown = JSON.parse(raw);
    if (parsed && typeof parsed === "object") {
      return parsed as ActivityPayload;
    }
  } catch {
    // 历史或损坏 payload
  }
  return {};
}

function payloadText(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

const TODO_STATUSES: readonly TodoStatus[] = ["Todo", "Doing", "Done", "Archived"];

function isTodoStatus(value: string): value is TodoStatus {
  return (TODO_STATUSES as readonly string[]).includes(value);
}

/** 活动条展示：最新在上（仓库无游标时仍为时间正序） */
export function eventsNewestFirst(events: EventDto[]): EventDto[] {
  return [...events].sort((a, b) => {
    const byTime = b.createdAt.localeCompare(a.createdAt);
    if (byTime !== 0) return byTime;
    return b.id.localeCompare(a.id);
  });
}

export type ActivityLine = {
  id: string;
  action: string;
  title: string | null;
  time: string;
};

/** 活动条一行：动作 · 标题 · 相对时间 */
export function formatActivityLine(
  event: EventDto,
  t: Composer["t"],
  nowMs: number = Date.now(),
): ActivityLine {
  const payload = parseActivityPayload(event.payload);
  const title = payloadText(payload.title);
  const time = formatRelativeTime(event.createdAt, t, nowMs);

  if (event.eventType === "todo.transitioned") {
    const to = payloadText(payload.to);
    const action =
      to && isTodoStatus(to) ? t(`status.${to}`) : translateEventType(event.eventType, t);
    return { id: event.id, action, title, time };
  }
  if (event.eventType === "reminder.triggered") {
    return { id: event.id, action: t("activity.actionReminder"), title, time };
  }
  if (event.eventType === "reminder.snoozed") {
    const minutes = typeof payload.minutes === "number" ? payload.minutes : null;
    const action =
      minutes != null
        ? t("activity.actionSnoozed", { n: minutes })
        : translateEventType(event.eventType, t);
    return { id: event.id, action, title, time };
  }
  return {
    id: event.id,
    action: translateEventType(event.eventType, t),
    title,
    time,
  };
}

export function formatActivitySummary(
  event: EventDto,
  t: Composer["t"],
  nowMs: number = Date.now(),
): string {
  const line = formatActivityLine(event, t, nowMs);
  if (line.title) {
    return t("activity.summaryWithTitle", {
      action: line.action,
      title: line.title,
      time: line.time,
    });
  }
  return t("activity.summaryNoTitle", { action: line.action, time: line.time });
}
