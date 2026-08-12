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

const DRAG_THRESHOLD = 10;

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

const tooltip = computed(() => {
  const parts = [`${props.activeCount} 待办`];
  if (props.overdueCount > 0) parts.push(`${props.overdueCount} 逾期`);
  return `${parts.join(" · ")} · 单击打开，拖到边缘可贴边`;
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

function onPointerCancel(event: PointerEvent) {
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
    :class="[`edge-${edge}`, urgencyClass]"
    :title="tooltip"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerCancel"
    @mouseenter="emit('enter')"
    @mouseleave="emit('leave')"
  >
    <span class="dock-strip-bar">
      <span v-if="badge" class="count">{{ badge }}</span>
    </span>
  </div>
</template>

<style scoped>
.dock-strip {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  cursor: grab;
  user-select: none;
  touch-action: none;
  background: transparent;
}

.edge-left {
  justify-content: flex-start;
}

.edge-right {
  justify-content: flex-end;
}

.dock-strip-bar {
  width: 16px;
  height: 48px;
  flex: 0 0 16px;
  background: #2563eb;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.edge-left .dock-strip-bar {
  border-radius: 0 8px 8px 0;
}

.edge-right .dock-strip-bar {
  border-radius: 8px 0 0 8px;
}

.urgency-active .dock-strip-bar {
  background: #2563eb;
}

.urgency-overdue .dock-strip-bar {
  background: #dc2626;
}

.count {
  position: absolute;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  border-radius: 9px;
  background: #111827;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  line-height: 18px;
  text-align: center;
  font-family: system-ui, sans-serif;
}

.edge-left .count {
  left: 10px;
}

.edge-right .count {
  right: 10px;
}
</style>
