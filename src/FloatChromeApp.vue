<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";

import type { CompanionSession, SettingsDto } from "@/api/types";
import { getSettings } from "@/api/settings";
import {
  companionClickChrome,
  companionMinimize,
  companionPointerCluster,
  getCompanionSession,
} from "@/api/window";
import FloatBall from "@/components/float/FloatBall.vue";
import FloatDockStrip from "@/components/float/FloatDockStrip.vue";
import AppShellOverlays from "@/components/common/AppShellOverlays.vue";
import { useCompanionDrag } from "@/composables/useCompanionDrag";
import { TAURI_EVENTS } from "@/constants/events";
import { applyAppearance } from "@/utils/appearance";

const session = ref<CompanionSession | null>(null);
const { startDrag, onPointerUp } = useCompanionDrag("chrome");

let unlistenSession: (() => void) | null = null;
let unlistenSettings: (() => void) | null = null;

function applySession(next: CompanionSession) {
  session.value = next;
}

async function onClick() {
  try {
    applySession(await companionClickChrome());
  } catch (err) {
    console.error("companionClickChrome failed", err);
  }
}

async function onEnter() {
  if (!session.value?.hoverPreview) return;
  try {
    applySession(await companionPointerCluster("chrome", { inside: true }));
  } catch (err) {
    console.error("pointer cluster enter failed", err);
  }
}

async function onLeave() {
  if (!session.value?.hoverPreview) return;
  try {
    applySession(await companionPointerCluster("chrome", { inside: false }));
  } catch (err) {
    console.error("pointer cluster leave failed", err);
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  const current = session.value;
  if (!current || current.visibility !== "shown") return;
  if (current.panelMode === "preview" || current.panelMode === "pinned") {
    void companionMinimize().then(applySession).catch((err) => {
      console.error("chrome minimize failed", err);
    });
  }
}

onMounted(async () => {
  document.addEventListener("pointerup", onPointerUp, true);
  document.addEventListener("pointercancel", onPointerUp, true);
  window.addEventListener("keydown", onKeydown);
  unlistenSession = await listen<CompanionSession>(TAURI_EVENTS.FLOAT_SESSION_CHANGED, (event) =>
    applySession(event.payload),
  );
  unlistenSettings = await listen<SettingsDto>(TAURI_EVENTS.SETTINGS_UPDATED, (event) => {
    applyAppearance(event.payload);
  });
  try {
    const settings = await getSettings();
    applyAppearance(settings);
  } catch {
    // ignore
  }
  try {
    applySession(await getCompanionSession());
  } catch (err) {
    console.error("getCompanionSession failed", err);
  }
});

onUnmounted(() => {
  document.removeEventListener("pointerup", onPointerUp, true);
  document.removeEventListener("pointercancel", onPointerUp, true);
  window.removeEventListener("keydown", onKeydown);
  unlistenSession?.();
  unlistenSettings?.();
});
</script>

<template>
  <div class="chrome-root">
    <Transition name="float-chrome" mode="out-in">
      <FloatBall
        v-if="session?.chrome === 'ball'"
        key="ball"
        :active-count="session.activeCount"
        :overdue-count="session.overdueCount"
        @click="onClick"
        @drag-start="startDrag"
      />
      <FloatDockStrip
        v-else-if="session?.chrome === 'strip'"
        key="strip"
        :edge="session.dockEdge"
        :active-count="session.activeCount"
        :overdue-count="session.overdueCount"
        @click="onClick"
        @drag-start="startDrag"
        @enter="onEnter"
        @leave="onLeave"
      />
    </Transition>

    <AppShellOverlays />
  </div>
</template>

<style>
html,
body,
#app {
  margin: 0;
  height: 100%;
  background: transparent;
  overflow: hidden;
}
</style>

<style scoped>
.chrome-root {
  width: 100%;
  height: 100vh;
  background: transparent;
}

.float-chrome-enter-active,
.float-chrome-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s cubic-bezier(0.22, 1, 0.36, 1);
}

.float-chrome-enter-from,
.float-chrome-leave-to {
  opacity: 0;
  transform: scale(0.92);
}

@media (prefers-reduced-motion: reduce) {
  .float-chrome-enter-active,
  .float-chrome-leave-active {
    transition: none;
  }
}
</style>
