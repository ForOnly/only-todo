<script setup lang="ts">
import { computed } from "vue";

import type { TodoDto } from "@/api/types";

const props = defineProps<{
  detail: TodoDto;
}>();

defineEmits<{
  close: [];
  openMain: [];
}>();

const descriptionText = computed(() => props.detail.description?.trim() ?? "");
</script>

<template>
  <section class="peek" @click.stop @dblclick.stop="$emit('openMain')">
    <div class="peek-head">
      <h3 class="peek-title">{{ detail.title }}</h3>
      <button
        type="button"
        class="peek-close"
        :title="$t('companion.detailClose')"
        :aria-label="$t('companion.detailClose')"
        @click.stop="$emit('close')"
      >
        ×
      </button>
    </div>
    <p v-if="descriptionText" class="peek-desc">{{ descriptionText }}</p>
    <p v-else class="peek-desc empty">{{ $t("companion.noDescription") }}</p>
  </section>
</template>

<style scoped>
.peek {
  margin: 0 8px 8px 28px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  background: var(--color-surface-muted);
  border: 1px solid var(--color-border);
  cursor: pointer;
}

.peek-head {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.peek-title {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.35;
  color: var(--color-text);
  word-break: break-word;
}

.peek-close {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  margin: -2px -4px 0 0;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-muted);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  transition:
    color var(--transition-fast),
    background var(--transition-fast);
}

.peek-close:hover {
  color: var(--color-text);
  background: var(--color-bg-accent);
}

.peek-desc {
  margin: 8px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
}

.peek-desc.empty {
  color: var(--color-muted);
  font-style: italic;
}
</style>
