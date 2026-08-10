import type { AppError } from "@/api/types";

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

/** 提取供 UI 展示的人类可读错误信息 */
export function formatErrorMessage(error: unknown): string {
  const parsed = parseAppError(error);
  if (parsed) {
    return parsed.message;
  }
  return String(error);
}
