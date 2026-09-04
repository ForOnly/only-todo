<script setup lang="ts">
import AppMessage from "@/components/common/AppMessage.vue";
import { useMessage } from "@/composables/useMessage";

const { messages, dismiss } = useMessage();
</script>

<template>
  <div class="host" aria-live="polite">
    <TransitionGroup name="msg" tag="div" class="stack">
      <AppMessage
        v-for="item in messages"
        :key="item.id"
        :item="item"
        @close="dismiss(item.id)"
      />
    </TransitionGroup>
  </div>
</template>

<style scoped>
.host {
  position: fixed;
  top: 8px;
  inset-inline: 0;
  z-index: var(--z-message);
  display: flex;
  flex-direction: column;
  align-items: center;
  pointer-events: none;
  padding-inline: 12px;
}

.stack {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  width: 100%;
  max-width: min(400px, 100%);
}

.stack :deep(.msg) {
  pointer-events: auto;
}

.msg-enter-active,
.msg-leave-active {
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.msg-move {
  transition: transform var(--transition-fast);
}

.msg-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.msg-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.msg-leave-active {
  position: absolute;
  width: min(400px, calc(100% - 24px));
}

@media (prefers-reduced-motion: reduce) {
  .msg-enter-active,
  .msg-leave-active,
  .msg-move {
    transition: opacity 0.1s ease;
  }

  .msg-enter-from,
  .msg-leave-to {
    transform: none;
  }
}
</style>
