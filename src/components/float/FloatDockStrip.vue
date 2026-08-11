<script setup lang="ts">
import type { DockEdge } from "@/api/types";

defineProps<{
  edge: DockEdge;
}>();

const emit = defineEmits<{
  peek: [clientY: number];
  unpeek: [];
  "drag-start": [];
}>();

const DRAG_THRESHOLD = 5;
const PEEK_DELAY_MS = 150;

let pointerId: number | null = null;
let startX = 0;
let startY = 0;
let moved = false;
let peekTimer: ReturnType<typeof setTimeout> | null = null;
let lastClientY = 0;

function clearPeekTimer() {
  if (peekTimer) {
    clearTimeout(peekTimer);
    peekTimer = null;
  }
}

function onMouseEnter(event: MouseEvent) {
  lastClientY = event.clientY;
  clearPeekTimer();
  peekTimer = setTimeout(() => {
    peekTimer = null;
    if (!moved) {
      emit("peek", lastClientY);
    }
  }, PEEK_DELAY_MS);
}

function onMouseLeave() {
  clearPeekTimer();
  emit("unpeek");
}

function onMouseMove(event: MouseEvent) {
  lastClientY = event.clientY;
}

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  pointerId = event.pointerId;
  startX = event.clientX;
  startY = event.clientY;
  moved = false;
  lastClientY = event.clientY;
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onPointerMove(event: PointerEvent) {
  if (pointerId !== event.pointerId || moved) return;
  const dx = Math.abs(event.clientX - startX);
  const dy = Math.abs(event.clientY - startY);
  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    moved = true;
    clearPeekTimer();
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {
      // ignore
    }
    emit("drag-start");
  }
}

function onPointerUp(event: PointerEvent) {
  if (pointerId !== event.pointerId) return;
  pointerId = null;
  moved = false;
  try {
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  } catch {
    // ignore
  }
}
</script>

<template>
  <div
    class="dock-strip"
    :class="`edge-${edge}`"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
    @mousemove="onMouseMove"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
  >
    <div class="dock-strip-bar" />
  </div>
</template>

<style scoped>
/* HWND 约 48px；蓝条 8px 贴屏幕外缘，内侧透明热区 */
.dock-strip {
  width: 100%;
  height: 100%;
  background: transparent;
  display: flex;
  cursor: pointer;
  user-select: none;
  touch-action: none;
}

.edge-left {
  justify-content: flex-start;
}

.edge-right {
  justify-content: flex-end;
}

.edge-top {
  flex-direction: column;
  justify-content: flex-start;
}

.edge-bottom {
  flex-direction: column;
  justify-content: flex-end;
}

.dock-strip-bar {
  background: #2563eb;
  flex-shrink: 0;
}

.edge-left .dock-strip-bar,
.edge-right .dock-strip-bar {
  width: 8px;
  height: 100%;
}

.edge-top .dock-strip-bar,
.edge-bottom .dock-strip-bar {
  width: 100%;
  height: 8px;
}
</style>
