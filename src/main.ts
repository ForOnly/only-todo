import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "@/App.vue";
import FloatBodyApp from "@/FloatBodyApp.vue";
import FloatChromeApp from "@/FloatChromeApp.vue";
import { getSettings } from "@/api/settings";
import { createAppI18n } from "@/i18n";
import { applyAppearance } from "@/utils/appearance";
import "@/styles/tokens.css";
import "@/styles/forms.css";

async function bootstrap() {
  const label = getCurrentWindow().label;
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
