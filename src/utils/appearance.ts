/**
 * 三窗共用：只应用语言。主题由宿主 set_theme，CSS 跟 prefers-color-scheme。
 */
import type { SettingsDto } from "@/api/types";
import { setI18nLocale } from "@/i18n";

export function applyAppearance(settings: Pick<SettingsDto, "uiLocale">): void {
  setI18nLocale(settings.uiLocale);
}
