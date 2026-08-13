import type { AppError } from "@/api/types";
import { i18n } from "@/i18n";

/** 将 Tauri invoke 错误解析为结构化 AppError（若可能） */
export function parseAppError(error: unknown): AppError | null {
  if (typeof error === "string") {
    try {
      const parsed = JSON.parse(error) as unknown;
      return parseAppError(parsed);
    } catch {
      return { code: "UNKNOWN", message: error };
    }
  }

  if (error && typeof error === "object") {
    const record = error as Record<string, unknown>;
    if (typeof record.code === "string" && typeof record.message === "string") {
      return { code: record.code, message: record.message };
    }
    if (typeof record.message === "string") {
      return { code: "UNKNOWN", message: record.message };
    }
  }

  return null;
}

const KNOWN_CODES = new Set([
  "NOT_FOUND",
  "VALIDATION_ERROR",
  "INVALID_TRANSITION",
  "DB_ERROR",
  "INTERNAL_ERROR",
]);

/** 提取供 UI 展示的人类可读错误信息（常见 code 走 i18n） */
export function formatErrorMessage(error: unknown): string {
  const parsed = parseAppError(error);
  if (parsed) {
    if (KNOWN_CODES.has(parsed.code)) {
      return i18n.global.t(`errors.${parsed.code}` as "errors.NOT_FOUND");
    }
    return parsed.message;
  }
  return String(error);
}
