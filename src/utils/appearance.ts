/**
 * 三窗共用：根据 settings 应用主题与语言。
 */
import type { SettingsDto } from "@/api/types";
import { setI18nLocale } from "@/i18n";
import { applyDocumentTheme, watchSystemTheme } from "@/utils/theme";
import type { UiTheme } from "@/api/generated/UiTheme";

let currentTheme: UiTheme = "system";
let stopSystemWatch: (() => void) | null = null;

export function applyAppearance(settings: Pick<SettingsDto, "uiTheme" | "uiLocale">): void {
  currentTheme = settings.uiTheme;
  applyDocumentTheme(settings.uiTheme);
  setI18nLocale(settings.uiLocale);

  if (!stopSystemWatch) {
    stopSystemWatch = watchSystemTheme(() => currentTheme);
  }
}
