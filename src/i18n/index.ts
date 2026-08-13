import { createI18n } from "vue-i18n";

import type { UiLocale } from "@/api/generated/UiLocale";
import enUS from "./en-US";
import zhCN from "./zh-CN";

export type MessageSchema = typeof zhCN;

export const i18n = createI18n({
  legacy: false,
  locale: "zh-CN",
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
    "en-US": enUS as unknown as MessageSchema,
  },
});

export function setI18nLocale(locale: UiLocale): void {
  i18n.global.locale.value = locale === "en-US" ? "en-US" : "zh-CN";
}

export function createAppI18n() {
  return i18n;
}
