<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";

const props = defineProps<{
  open: boolean;
  title: string;
  /** 悬浮窗等窄容器：缩小最小宽度 */
  compact?: boolean;
  /** 默认 true；float 窗口内设 false，避免 Teleport 打断 hover */
  teleport?: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && props.open) {
    emit("close");
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body" :disabled="teleport === false">
    <div v-if="open" class="modal-backdrop" @click.self="emit('close')">
      <div
        class="modal"
        :class="{ compact }"
        role="dialog"
        aria-modal="true"
        :aria-label="title"
      >
        <header class="modal-header">
          <h2>{{ title }}</h2>
          <button
            class="close-btn"
            type="button"
            :aria-label="$t('common.close')"
            @click="emit('close')"
          >
            ×
          </button>
        </header>
        <div class="modal-body">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.modal {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  min-width: 400px;
  max-width: 90vw;
  max-height: min(90vh, calc(100vh - 48px));
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-md);
  overflow: hidden;
  color: var(--color-text);
}

.modal.compact {
  min-width: 260px;
  max-width: calc(100vw - 24px);
  width: calc(100% - 24px);
  max-height: calc(100vh - 24px);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.modal-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.close-btn {
  border: none;
  background: none;
  font-size: 24px;
  line-height: 1;
  color: var(--color-muted);
  cursor: pointer;
  padding: 0 4px;
  transition: color var(--transition-fast);
}

.close-btn:hover {
  color: var(--color-text);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
</style>
