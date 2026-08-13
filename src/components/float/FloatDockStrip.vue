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
  return "urgency-idle";
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
      <Transition name="dock-badge">
        <span v-if="badge" class="count">{{ badge }}</span>
      </Transition>
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
  outline: none;
}

.dock-strip:focus,
.dock-strip:focus-visible {
  outline: none;
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

/* 表面手柄 + 朝屏幕内侧的 accent 脊；无外扩 shadow */
.dock-strip-bar {
  --strip-fill: color-mix(in srgb, var(--color-surface) 90%, transparent);
  --spine-color: color-mix(in srgb, var(--color-accent) 40%, transparent);
  --badge-fill: var(--color-accent);
  --badge-fg: var(--color-on-accent);

  width: 16px;
  height: 48px;
  flex: 0 0 16px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--strip-fill);
  transition:
    transform var(--float-motion, 220ms cubic-bezier(0.22, 1, 0.36, 1)),
    background-color 0.22s ease,
    box-shadow var(--float-motion, 220ms cubic-bezier(0.22, 1, 0.36, 1));
  will-change: transform;
}

.edge-left .dock-strip-bar {
  border-radius: 0 10px 10px 0;
  box-shadow:
    inset -2px 0 0 var(--spine-color),
    inset 0 1px 0 color-mix(in srgb, var(--color-surface) 45%, transparent);
}

.edge-right .dock-strip-bar {
  border-radius: 10px 0 0 10px;
  box-shadow:
    inset 2px 0 0 var(--spine-color),
    inset 0 1px 0 color-mix(in srgb, var(--color-surface) 45%, transparent);
}

/* 空闲：淡脊 + 竖点可拖暗示 */
.urgency-idle .dock-strip-bar {
  --strip-fill: color-mix(in srgb, var(--color-surface) 88%, transparent);
  --spine-color: color-mix(in srgb, var(--color-accent) 38%, transparent);
}

.urgency-idle .dock-strip-bar::after {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 2px;
  height: 12px;
  transform: translate(-50%, -50%);
  border-radius: 1px;
  background: repeating-linear-gradient(
    to bottom,
    color-mix(in srgb, var(--color-accent) 40%, transparent) 0 2px,
    transparent 2px 5px
  );
  opacity: 0.7;
  pointer-events: none;
}

/* 有待办：实色脊 + 角标 */
.urgency-active .dock-strip-bar {
  --strip-fill: color-mix(in srgb, var(--color-accent-soft) 50%, var(--color-surface));
  --spine-color: var(--color-accent);
  --badge-fill: var(--color-accent);
  --badge-fg: var(--color-on-accent);
}

/* 逾期：urgent 脊呼吸 + 角标 */
.urgency-overdue .dock-strip-bar {
  --strip-fill: color-mix(in srgb, var(--color-surface) 90%, transparent);
  --spine-color: var(--color-priority-urgent);
  --badge-fill: var(--color-priority-urgent);
  --badge-fg: var(--color-on-accent);
}

.edge-left.urgency-overdue .dock-strip-bar::before,
.edge-right.urgency-overdue .dock-strip-bar::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  animation: float-strip-pulse 2.4s ease-in-out infinite;
}

.edge-left.urgency-overdue .dock-strip-bar::before {
  box-shadow: inset -2px 0 0 color-mix(in srgb, var(--color-priority-urgent) 75%, transparent);
}

.edge-right.urgency-overdue .dock-strip-bar::before {
  box-shadow: inset 2px 0 0 color-mix(in srgb, var(--color-priority-urgent) 75%, transparent);
}

.dock-strip:focus-visible .dock-strip-bar {
  --spine-color: color-mix(in srgb, var(--spine-color) 80%, var(--color-text));
}

/* 向屏幕内侧轻 nudge */
.edge-left.dock-strip:hover .dock-strip-bar {
  transform: translateX(2px);
}

.edge-right.dock-strip:hover .dock-strip-bar {
  transform: translateX(-2px);
}

.edge-left.dock-strip:active .dock-strip-bar {
  transform: translateX(1px) scale(0.96);
}

.edge-right.dock-strip:active .dock-strip-bar {
  transform: translateX(-1px) scale(0.96);
}

/* 与圆球同款实色 pill 角标 */
.count {
  position: absolute;
  z-index: 1;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: var(--badge-fill);
  color: var(--badge-fg);
  font-size: 10px;
  font-weight: 700;
  line-height: 18px;
  text-align: center;
  font-family: var(--font-ui);
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.03em;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-surface) 35%, transparent);
  pointer-events: none;
}

.edge-left .count {
  left: 10px;
}

.edge-right .count {
  right: 10px;
}

.dock-badge-enter-active,
.dock-badge-leave-active {
  transition:
    opacity 0.16s ease,
    transform 0.16s cubic-bezier(0.22, 1, 0.36, 1);
}

.dock-badge-enter-from,
.dock-badge-leave-to {
  opacity: 0;
  transform: scale(0.75);
}

@keyframes float-strip-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.35;
  }
}

@media (prefers-reduced-motion: reduce) {
  .dock-strip-bar {
    transition-duration: 0.01ms;
  }

  .edge-left.urgency-overdue .dock-strip-bar::before,
  .edge-right.urgency-overdue .dock-strip-bar::before {
    animation: none;
    opacity: 0.85;
  }

  .edge-left.dock-strip:hover .dock-strip-bar,
  .edge-right.dock-strip:hover .dock-strip-bar,
  .edge-left.dock-strip:active .dock-strip-bar,
  .edge-right.dock-strip:active .dock-strip-bar {
    transform: none;
  }

  .dock-badge-enter-active,
  .dock-badge-leave-active {
    transition: none;
  }
}
</style>
