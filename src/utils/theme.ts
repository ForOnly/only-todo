import type { UiTheme } from "@/api/generated/UiTheme";

/** 将偏好解析为实际 light/dark（system 跟随 OS） */
export function resolveTheme(preference: UiTheme): "light" | "dark" {
  if (preference === "light" || preference === "dark") return preference;
  if (typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

/** 写入 documentElement.data-theme */
export function applyDocumentTheme(preference: UiTheme): void {
  document.documentElement.dataset.theme = resolveTheme(preference);
}

/** 监听系统主题变化（仅 preference=system 时生效） */
export function watchSystemTheme(
  getPreference: () => UiTheme,
  onChange?: () => void,
): () => void {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => {
    if (getPreference() === "system") {
      applyDocumentTheme("system");
      onChange?.();
    }
  };
  mq.addEventListener("change", handler);
  return () => mq.removeEventListener("change", handler);
}
