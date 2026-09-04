import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "@/App.vue";
import FloatBodyApp from "@/FloatBodyApp.vue";
import FloatChromeApp from "@/FloatChromeApp.vue";
import { getSettings } from "@/api/settings";
import type { SettingsDto } from "@/api/types";
import { createAppI18n } from "@/i18n";
import { applyAppearance } from "@/utils/appearance";
import { installDesktopGuards } from "@/utils/desktopGuards";
import "@/styles/tokens.css";
import "@/styles/forms.css";

const BACKEND_READY_TIMEOUT_MS = 10_000;
const BACKEND_READY_POLL_MS = 50;

/** 轮询 get_settings，直到 AppState 已 manage；超时返回 null 走默认语言降级 */
async function waitForSettings(): Promise<SettingsDto | null> {
  const deadline = Date.now() + BACKEND_READY_TIMEOUT_MS;
  while (Date.now() < deadline) {
    try {
      return await getSettings();
    } catch {
      await new Promise((resolve) => setTimeout(resolve, BACKEND_READY_POLL_MS));
    }
  }
  return null;
}

async function bootstrap() {
  installDesktopGuards();
  const label = getCurrentWindow().label;
  // 透明窗标记：tokens 对 chrome/body 强制透明；main 保持默认铺底防 FOUC
  document.documentElement.dataset.surface =
    label === "float-chrome" ? "chrome" : label === "float-body" ? "body" : "main";
  const root =
    label === "float-chrome" ? FloatChromeApp : label === "float-body" ? FloatBodyApp : App;
  const app = createApp(root);
  app.use(createAppI18n());

  const settings = await waitForSettings();
  if (settings) {
    applyAppearance(settings);
  }
  // 超时则语言保持默认 zh-CN；主题由宿主 set_theme，不在此涂 DOM

  app.mount("#app");
}

void bootstrap();
