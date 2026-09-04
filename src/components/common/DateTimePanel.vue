<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";

import { registerModalEsc } from "@/composables/useModalEscStack";

const props = withDefaults(
  defineProps<{
    open: boolean;
    teleport?: boolean;
    panelStyle?: Record<string, string>;
    triggerRef?: HTMLElement | null;
  }>(),
  {
    teleport: true,
    panelStyle: () => ({}),
    triggerRef: null,
  },
);

const emit = defineEmits<{
  close: [];
}>();

const visible = ref(false);
const animating = ref(false);
const panelRef = ref<HTMLElement | null>(null);
let closeTimer: ReturnType<typeof setTimeout> | null = null;
let unregEsc: (() => void) | null = null;

watch(
  () => props.open,
  (v) => {
    if (closeTimer) {
      clearTimeout(closeTimer);
      closeTimer = null;
    }
    unregEsc?.();
    unregEsc = null;
    if (v) {
      visible.value = true;
      nextTick(() => {
        animating.value = true;
      });
      unregEsc = registerModalEsc(() => emit("close"));
    } else {
      animating.value = false;
      closeTimer = setTimeout(() => {
        visible.value = false;
        closeTimer = null;
      }, 150);
    }
  },
);

onUnmounted(() => {
  if (closeTimer) clearTimeout(closeTimer);
  unregEsc?.();
});

function onDocPointer(event: PointerEvent) {
  if (!props.open) return;
  const t = event.target as Node;
  if (panelRef.value?.contains(t)) return;
  if (props.triggerRef?.contains(t)) return;
  emit("close");
}

function onKeydown(event: KeyboardEvent) {
  if (!props.open) return;
  if (event.key === "Escape") {
    event.preventDefault();
    event.stopImmediatePropagation();
    emit("close");
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointer, true);
  window.addEventListener("keydown", onKeydown, true);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointer, true);
  window.removeEventListener("keydown", onKeydown, true);
});

defineExpose({
  panelEl: panelRef,
});
</script>

<template>
  <Teleport to="body" :disabled="teleport === false">
    <div v-if="visible" class="panel-wrap" :class="{ show: animating }">
      <div class="backdrop" @click.self="emit('close')" />
      <div ref="panelRef" class="panel" :style="panelStyle">
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.panel-wrap {
  position: fixed;
  inset: 0;
  /* 高于 AppModal(1000)，与 AppSelect / ContextMenu 同级 */
  z-index: 1100;
  pointer-events: none;
}

.panel-wrap.show {
  pointer-events: auto;
}

.backdrop {
  position: absolute;
  inset: 0;
  background: transparent;
}

.panel {
  /* position 由父组件 inline fixed 控制 */
  width: 300px;
  padding: 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  box-shadow: var(--shadow-md);
  color: var(--color-text);
  opacity: 0;
  transform: translateY(4px);
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
  pointer-events: auto;
}

.panel-wrap.show .panel {
  opacity: 1;
  transform: translateY(0);
}
</style>
