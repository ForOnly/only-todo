<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  activeCount: number;
  overdueCount: number;
  dueTodayCount: number;
}>();

const emit = defineEmits<{
  expand: [];
  "drag-start": [];
}>();

const urgencyClass = computed(() => {
  if (props.overdueCount > 0) return "urgency-overdue";
  if (props.dueTodayCount > 0) return "urgency-today";
  return "";
});

const DRAG_THRESHOLD = 5;
let pointerId: number | null = null;
let startX = 0;
let startY = 0;
let moved = false;

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
    emit("expand");
  }
}
</script>

<template>
  <div
    class="float-ball-host"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
  >
    <div class="float-ball" :class="urgencyClass">
      <span class="logo">T</span>
      <span v-if="activeCount > 0" class="badge">{{ activeCount > 99 ? "99+" : activeCount }}</span>
    </div>
  </div>
</template>

<style scoped>
/* 64×64 窗口内居中 56 圆球，为 badge 留白避免 HWND 裁切 */
.float-ball-host {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  touch-action: none;
  cursor: pointer;
  user-select: none;
}

.float-ball {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: linear-gradient(145deg, #3b82f6, #2563eb);
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  font-family: Inter, system-ui, sans-serif;
}

.float-ball.urgency-today {
  background: linear-gradient(145deg, #f59e0b, #d97706);
  box-shadow: 0 4px 16px rgba(217, 119, 6, 0.45);
}

.float-ball.urgency-overdue {
  background: linear-gradient(145deg, #ef4444, #dc2626);
  box-shadow: 0 4px 16px rgba(220, 38, 38, 0.45);
}

.logo {
  color: #fff;
  font-weight: 700;
  font-size: 22px;
}

.badge {
  position: absolute;
  top: -2px;
  right: -2px;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  border-radius: 9px;
  background: #111827;
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  line-height: 18px;
  text-align: center;
}
</style>
