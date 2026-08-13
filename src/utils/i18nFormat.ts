import type { Composer } from "vue-i18n";

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
