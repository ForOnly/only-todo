<script setup lang="ts">
import { computed } from "vue";

import type { DockEdge } from "@/api/types";

const props = defineProps<{
  edge: DockEdge;
  activeCount: number;
  overdueCount: number;
}>();

const emit = defineEmits<{
  click: [];
  "drag-start": [];
  enter: [];
  leave: [];
}>();

const DRAG_THRESHOLD = 5;

let pointerId: number | null = null;
let startX = 0;
let startY = 0;
let moved = false;

const urgencyClass = computed(() => {
  if (props.overdueCount > 0) return "urgency-overdue";
  if (props.activeCount > 0) return "urgency-active";
  return "";
});

const badge = computed(() => {
  if (props.activeCount <= 0) return "";
  return props.activeCount > 99 ? "99+" : String(props.activeCount);
});

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  pointerId = event.pointerId;
  startX = event.clientX;
  startY = event.clientY;
  moved = false;
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onPointerMove(event: PointerEvent) {
  if (pointerId !== event.pointerId || moved) return;
  const dx = Math.abs(event.clientX - startX);
  const dy = Math.abs(event.clientY - startY);
  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    moved = true;
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
  const wasClick = !moved;
  pointerId = null;
  moved = false;
  try {
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  } catch {
    // ignore
  }
  if (wasClick) {
    emit("click");
  }
}
</script>

<template>
  <button
    type="button"
    class="dock-strip"
    :class="[`edge-${edge}`, urgencyClass]"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @mouseenter="emit('enter')"
    @mouseleave="emit('leave')"
  >
    <span class="dock-strip-bar">
      <span v-if="badge" class="count">{{ badge }}</span>
    </span>
  </button>
</template>

<style scoped>
.dock-strip {
  width: 100%;
  height: 100%;
  padding: 0;
  border: none;
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

.dock-strip-bar {
  width: 16px;
  height: 100%;
  flex: 0 0 16px;
  background: #2563eb;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 10px;
  font-weight: 600;
}

.edge-left .dock-strip-bar {
  border-radius: 0 4px 4px 0;
}

.edge-right .dock-strip-bar {
  border-radius: 4px 0 0 4px;
}

.urgency-active .dock-strip-bar {
  background: #2563eb;
}

.urgency-overdue .dock-strip-bar {
  background: #dc2626;
}

.count {
  writing-mode: vertical-rl;
  text-orientation: mixed;
  letter-spacing: 0.04em;
  line-height: 1;
}
</style>
