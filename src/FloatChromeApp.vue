<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";

import type { CompanionSession } from "@/api/types";
import { companionClickChrome, companionPointerCluster, getCompanionSession } from "@/api/window";
import FloatBall from "@/components/float/FloatBall.vue";
import FloatDockStrip from "@/components/float/FloatDockStrip.vue";
import { useCompanionDrag } from "@/composables/useCompanionDrag";
import { TAURI_EVENTS } from "@/constants/events";

const session = ref<CompanionSession | null>(null);
const { startDrag, onPointerUp } = useCompanionDrag("chrome");

let unlistenSession: (() => void) | null = null;

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

onMounted(async () => {
  document.addEventListener("pointerup", onPointerUp, true);
  document.addEventListener("pointercancel", onPointerUp, true);
  unlistenSession = await listen<CompanionSession>(TAURI_EVENTS.FLOAT_SESSION_CHANGED, (event) =>
    applySession(event.payload),
  );
  try {
    applySession(await getCompanionSession());
  } catch (err) {
    console.error("getCompanionSession failed", err);
  }
});

onUnmounted(() => {
  document.removeEventListener("pointerup", onPointerUp, true);
  document.removeEventListener("pointercancel", onPointerUp, true);
  unlistenSession?.();
});
</script>

<template>
  <div class="chrome-root">
    <FloatBall
      v-if="session?.chrome === 'ball'"
      :active-count="session.activeCount"
      :overdue-count="session.overdueCount"
      @click="onClick"
      @drag-start="startDrag"
    />
    <FloatDockStrip
      v-else-if="session?.chrome === 'strip'"
      :edge="session.dockEdge"
      :active-count="session.activeCount"
      :overdue-count="session.overdueCount"
      @click="onClick"
      @drag-start="startDrag"
      @enter="onEnter"
      @leave="onLeave"
    />
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
</style>
