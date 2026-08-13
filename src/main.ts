import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "@/App.vue";
import FloatBodyApp from "@/FloatBodyApp.vue";
import FloatChromeApp from "@/FloatChromeApp.vue";
import { getSettings } from "@/api/settings";
import { createAppI18n } from "@/i18n";
import { applyAppearance } from "@/utils/appearance";
import { installDesktopGuards } from "@/utils/desktopGuards";
import "@/styles/tokens.css";
import "@/styles/forms.css";

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

  try {
    applyAppearance(await getSettings());
  } catch {
    // settings 未就绪时保留 tokens 默认 light + zh-CN
  }

  app.mount("#app");
}

void bootstrap();
