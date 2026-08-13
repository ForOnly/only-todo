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
  return "";
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
  cursor: grab;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  touch-action: none;
}

.float-ball-host:active {
  cursor: grabbing;
}

.float-ball {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-on-accent);
  font-size: 15px;
  font-weight: 700;
  font-family: var(--font-ui);
  background:
    radial-gradient(circle at 32% 28%, color-mix(in srgb, #fff 28%, transparent), transparent 46%),
    var(--color-accent);
  box-shadow:
    var(--shadow-md),
    inset 0 1px 0 color-mix(in srgb, #fff 22%, transparent);
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease,
    background-color 0.2s ease;
}

.float-ball-host:hover .float-ball {
  transform: scale(1.04);
}

.float-ball.urgency-active {
  background:
    radial-gradient(circle at 32% 28%, color-mix(in srgb, #fff 28%, transparent), transparent 46%),
    var(--color-accent);
}

.float-ball.urgency-overdue {
  background:
    radial-gradient(circle at 32% 28%, color-mix(in srgb, #fff 22%, transparent), transparent 46%),
    var(--color-priority-urgent);
}

.badge {
  line-height: 1;
  letter-spacing: -0.02em;
}
</style>
