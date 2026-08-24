<script setup lang="ts">
import { useI18n } from "vue-i18n";

defineProps<{
  clearable: boolean;
}>();

const emit = defineEmits<{
  today: [];
  clear: [];
  confirm: [];
}>();

useI18n();
</script>

<template>
  <div class="action-bar">
    <button type="button" class="act-btn" @click="emit('today')">
      {{ $t("datetime.today") }}
    </button>
    <div class="spacer" />
    <button v-if="clearable" type="button" class="act-btn" @click="emit('clear')">
      {{ $t("datetime.clear") }}
    </button>
    <button type="button" class="act-btn primary" @click="emit('confirm')">
      {{ $t("common.confirm") }}
    </button>
  </div>
</template>

<style scoped>
.action-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--color-border);
}

.spacer {
  flex: 1;
}

.act-btn {
  border: none;
  background: transparent;
  color: var(--color-muted);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  transition:
    color var(--transition-fast),
    background var(--transition-fast);
}

.act-btn:hover {
  color: var(--color-text);
  background: var(--color-bg-accent);
}

.act-btn.primary {
  color: var(--color-accent);
  font-weight: 600;
}

.act-btn.primary:hover {
  background: var(--color-accent-soft);
}
</style>
