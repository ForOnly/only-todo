<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { MessageItem } from "@/composables/useMessage";

defineProps<{
  item: MessageItem;
}>();

defineEmits<{
  close: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div
    class="msg"
    :class="item.type"
    :role="item.type === 'error' || item.type === 'warning' ? 'alert' : 'status'"
  >
    <span class="bar" aria-hidden="true" />
    <span class="text">{{ item.text }}</span>
    <button
      v-if="item.closable"
      type="button"
      class="close"
      :title="t('common.messageClose')"
      :aria-label="t('common.messageClose')"
      @click="$emit('close')"
    >
      ×
    </button>
  </div>
</template>

<style scoped>
.msg {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  width: min(400px, 100%);
  padding: 10px 12px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  box-shadow: var(--shadow-message);
  font-size: 13px;
  line-height: 1.45;
  color: var(--color-text);
  word-break: break-word;
  overflow: hidden;
}

.bar {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--color-accent);
}

.text {
  flex: 1;
  min-width: 0;
  padding-inline-start: 2px;
}

.close {
  flex-shrink: 0;
  box-sizing: border-box;
  width: 24px;
  height: 24px;
  margin: -2px -4px -2px 0;
  padding: 0;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-muted);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
}

.close:hover {
  background: var(--color-bg-accent);
  color: var(--color-text);
}

.close:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.msg.success {
  border-color: color-mix(in srgb, var(--color-success) 28%, var(--color-border));
  background: color-mix(in srgb, var(--color-success) 7%, var(--color-surface));
}

.msg.success .bar {
  background: var(--color-success);
}

.msg.error {
  border-color: color-mix(in srgb, var(--color-danger) 28%, var(--color-border));
  background: color-mix(in srgb, var(--color-danger) 7%, var(--color-surface));
}

.msg.error .bar {
  background: var(--color-danger);
}

.msg.warning {
  border-color: color-mix(in srgb, var(--color-warning) 28%, var(--color-border));
  background: color-mix(in srgb, var(--color-warning) 7%, var(--color-surface));
}

.msg.warning .bar {
  background: var(--color-warning);
}

.msg.info {
  border-color: color-mix(in srgb, var(--color-accent) 28%, var(--color-border));
  background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
}

.msg.info .bar {
  background: var(--color-accent);
}
</style>
