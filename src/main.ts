import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "@/App.vue";
import FloatApp from "@/FloatApp.vue";
import "@/styles/forms.css";

async function bootstrap() {
  const label = getCurrentWindow().label;
  const root = label === "float" ? FloatApp : App;
  createApp(root).mount("#app");
}

void bootstrap();
