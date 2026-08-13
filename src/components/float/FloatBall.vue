<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  activeCount: number;
  overdueCount: number;
}>();

const emit = defineEmits<{
  click: [];
  "drag-start": [];
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
    props.overdueCount > 0 ? t("companion.ballOverdue", { n: props.overdueCount }) : "";
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
  <button
    type="button"
    class="float-ball-host"
    :title="tooltip"
    :aria-label="tooltip"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerCancel"
  >
    <div class="float-ball" :class="urgencyClass">
      <Transition name="ball-badge">
        <span v-if="badge" class="badge">{{ badge }}</span>
      </Transition>
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
  cursor: grab;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  touch-action: none;
  outline: none;
}

.float-ball-host:focus,
.float-ball-host:focus-visible {
  outline: none;
}

.float-ball-host:active {
  cursor: grabbing;
}

/* 仪表盘主体：表面圆 + inset 环；无外扩 shadow */
.float-ball {
  --ball-fill: color-mix(in srgb, var(--color-surface) 92%, transparent);
  --ring-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
  --badge-fill: var(--color-accent);
  --badge-fg: var(--color-on-accent);

  position: relative;
  width: 52px;
  height: 52px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-ui);
  background:
    radial-gradient(
      circle at 32% 28%,
      color-mix(in srgb, var(--color-surface) 40%, transparent),
      transparent 52%
    ),
    var(--ball-fill);
  box-shadow:
    inset 0 0 0 2px var(--ring-color),
    inset 0 1px 0 color-mix(in srgb, var(--color-surface) 55%, transparent);
  transition:
    transform var(--float-motion, 220ms cubic-bezier(0.22, 1, 0.36, 1)),
    box-shadow var(--float-motion, 220ms cubic-bezier(0.22, 1, 0.36, 1)),
    background-color 0.22s ease;
  will-change: transform;
}

/* 空闲：淡环 + 中心点 */
.float-ball.urgency-idle {
  --ball-fill: color-mix(in srgb, var(--color-surface) 88%, transparent);
  --ring-color: color-mix(in srgb, var(--color-accent) 38%, transparent);
}

.float-ball.urgency-idle::after {
  content: "";
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--color-accent) 55%, transparent);
}

/* 有待办：实色环 + 角标 */
.float-ball.urgency-active {
  --ball-fill: color-mix(in srgb, var(--color-accent-soft) 55%, var(--color-surface));
  --ring-color: var(--color-accent);
  --badge-fill: var(--color-accent);
  --badge-fg: var(--color-on-accent);
}

/* 逾期：urgent 环呼吸 + 角标 */
.float-ball.urgency-overdue {
  --ball-fill: color-mix(in srgb, var(--color-surface) 90%, transparent);
  --ring-color: var(--color-priority-urgent);
  --badge-fill: var(--color-priority-urgent);
  --badge-fg: var(--color-on-accent);
}

.float-ball.urgency-overdue::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 50%;
  box-shadow: inset 0 0 0 2px color-mix(in srgb, var(--color-priority-urgent) 70%, transparent);
  pointer-events: none;
  animation: float-urgency-pulse 2.4s ease-in-out infinite;
}

.float-ball-host:focus-visible .float-ball {
  box-shadow:
    inset 0 0 0 2px var(--ring-color),
    inset 0 0 0 4px color-mix(in srgb, var(--ring-color) 35%, transparent);
}

.float-ball-host:hover .float-ball {
  transform: scale(1.05);
}

.float-ball-host:active .float-ball {
  transform: scale(0.94);
}

.badge {
  position: absolute;
  z-index: 1;
  top: -2px;
  right: -2px;
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
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-surface) 35%, transparent);
  pointer-events: none;
}

.ball-badge-enter-active,
.ball-badge-leave-active {
  transition:
    opacity 0.16s ease,
    transform 0.16s cubic-bezier(0.22, 1, 0.36, 1);
}

.ball-badge-enter-from,
.ball-badge-leave-to {
  opacity: 0;
  transform: scale(0.75);
}

@keyframes float-urgency-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.35;
  }
}

@media (prefers-reduced-motion: reduce) {
  .float-ball {
    transition-duration: 0.01ms;
  }

  .float-ball.urgency-overdue::before {
    animation: none;
    opacity: 0.85;
  }

  .float-ball-host:hover .float-ball,
  .float-ball-host:active .float-ball {
    transform: none;
  }

  .ball-badge-enter-active,
  .ball-badge-leave-active {
    transition: none;
  }
}
</style>
