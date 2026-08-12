import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "@/App.vue";
import FloatBodyApp from "@/FloatBodyApp.vue";
import FloatChromeApp from "@/FloatChromeApp.vue";
import "@/styles/forms.css";

async function bootstrap() {
  const label = getCurrentWindow().label;
  const root =
    label === "float-chrome" ? FloatChromeApp : label === "float-body" ? FloatBodyApp : App;
  createApp(root).mount("#app");
}

void bootstrap();
