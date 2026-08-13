<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

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

const { t } = useI18n();

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
  const overdue =
    props.overdueCount > 0
      ? t("companion.ballOverdue", { n: props.overdueCount })
      : "";
  return t("companion.ballTitle", { active: props.activeCount, overdue });
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
    role="button"
    tabindex="0"
    :aria-label="tooltip"
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

.dock-strip:active {
  cursor: grabbing;
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
  background: var(--color-accent);
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-sm);
  transition:
    filter 0.18s ease,
    transform 0.18s ease,
    background-color 0.2s ease;
}

.dock-strip:hover .dock-strip-bar {
  filter: brightness(1.08);
}

.edge-left .dock-strip-bar {
  border-radius: 0 10px 10px 0;
}

.edge-right .dock-strip-bar {
  border-radius: 10px 0 0 10px;
}

.urgency-active .dock-strip-bar {
  background: var(--color-accent);
}

.urgency-overdue .dock-strip-bar {
  background: var(--color-priority-urgent);
}

.count {
  position: absolute;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  font-size: 10px;
  font-weight: 700;
  line-height: 16px;
  text-align: center;
  font-family: var(--font-ui);
  box-shadow: var(--shadow-sm);
}

.edge-left .count {
  left: 10px;
}

.edge-right .count {
  right: 10px;
}
</style>
