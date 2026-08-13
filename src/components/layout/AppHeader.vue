<script setup lang="ts">
import AppButton from "@/components/common/AppButton.vue";

defineProps<{
  keyword: string;
}>();

const emit = defineEmits<{
  "update:keyword": [value: string];
  search: [];
  create: [];
  settings: [];
}>();
</script>

<template>
  <header class="header">
    <div class="brand">{{ $t("common.brand") }}</div>
    <div class="actions">
      <input
        id="workbench-search"
        class="search"
        type="text"
        :placeholder="$t('header.searchPlaceholder')"
        :value="keyword"
        @input="emit('update:keyword', ($event.target as HTMLInputElement).value)"
        @keydown.enter="emit('search')"
      />
      <AppButton variant="primary" @click="emit('create')">{{ $t("header.newTodo") }}</AppButton>
      <AppButton variant="ghost" @click="emit('settings')">{{ $t("header.settings") }}</AppButton>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background: linear-gradient(180deg, var(--color-surface) 0%, var(--color-surface-muted) 100%);
}

.brand {
  font-family: var(--font-ui);
  font-weight: 750;
  font-size: 18px;
  letter-spacing: -0.02em;
  color: var(--color-text);
  flex-shrink: 0;
}

.actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex: 1;
  min-width: 0;
  justify-content: flex-end;
}

.search {
  flex: 1;
  max-width: 320px;
  min-width: 120px;
  padding: 8px 10px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font: inherit;
  color: var(--color-text);
  background: var(--color-surface);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.search:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: var(--focus-ring);
}
</style>
