<script setup lang="ts">
import { nextTick, onUnmounted, ref, watch } from "vue";

import { registerModalEsc } from "@/composables/useModalEscStack";

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const visible = ref(false);
const animating = ref(false);
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
      }, 220);
    }
  },
);

onUnmounted(() => {
  if (closeTimer) clearTimeout(closeTimer);
  unregEsc?.();
});
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="drawer-wrap" :class="{ show: animating }">
      <div class="drawer-backdrop" @click.self="emit('close')" />
      <div class="drawer-panel">
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.drawer-wrap {
  position: fixed;
  inset: 0;
  z-index: 200;
}

.drawer-backdrop {
  position: absolute;
  inset: 0;
  background: var(--color-overlay);
  opacity: 0;
  transition: opacity 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.drawer-wrap.show .drawer-backdrop {
  opacity: 1;
}

.drawer-panel {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 360px;
  max-width: 90vw;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
  min-height: 0;
  transform: translateX(100%);
  transition: transform 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.drawer-wrap.show .drawer-panel {
  transform: translateX(0);
}
</style>