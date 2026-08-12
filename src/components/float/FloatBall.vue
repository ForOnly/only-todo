<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  activeCount: number;
  overdueCount: number;
}>();

const emit = defineEmits<{
  click: [];
  "drag-start": [];
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
    class="float-ball-host"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
  >
    <div class="float-ball" :class="urgencyClass">
      <span v-if="badge" class="badge">{{ badge }}</span>
    </div>
  </button>
</template>

<style scoped>
.float-ball-host {
  width: 100%;
  height: 100%;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  touch-action: none;
}

.float-ball {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: #2563eb;
  box-shadow: 0 4px 16px rgba(37, 99, 235, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 14px;
  font-weight: 600;
}

.float-ball.urgency-active {
  background: #2563eb;
}

.float-ball.urgency-overdue {
  background: #dc2626;
  box-shadow: 0 4px 16px rgba(220, 38, 38, 0.45);
}

.badge {
  line-height: 1;
}
</style>
