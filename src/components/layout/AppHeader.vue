<script setup lang="ts">
import AppButton from "@/components/common/AppButton.vue";
import type { MainMode } from "@/api/types";

defineProps<{
  keyword: string;
  mode: MainMode;
}>();

const emit = defineEmits<{
  "update:keyword": [value: string];
  "update:mode": [value: MainMode];
  search: [];
  clearSearch: [];
  settings: [];
}>();
</script>

<template>
  <header class="header">
    <div class="modes" role="tablist" :aria-label="$t('modes.aria')">
      <button
        type="button"
        role="tab"
        class="mode"
        :class="{ active: mode === 'focus' }"
        :aria-selected="mode === 'focus'"
        @click="emit('update:mode', 'focus')"
      >
        {{ $t("modes.focus") }}
      </button>
      <button
        type="button"
        role="tab"
        class="mode"
        :class="{ active: mode === 'library' }"
        :aria-selected="mode === 'library'"
        @click="emit('update:mode', 'library')"
      >
        {{ $t("modes.library") }}
      </button>
    </div>
    <div class="actions">
      <div class="search-wrap">
        <input
          id="workbench-search"
          class="search"
          type="text"
          :placeholder="$t('header.searchPlaceholder')"
          :value="keyword"
          @input="emit('update:keyword', ($event.target as HTMLInputElement).value)"
          @keydown.enter="emit('search')"
        />
        <button
          v-if="keyword"
          type="button"
          class="clear"
          :aria-label="$t('header.clearSearch')"
          @click="emit('clearSearch')"
        >
          ×
        </button>
      </div>
      <AppButton variant="ghost" @click="emit('settings')">{{ $t("header.settings") }}</AppButton>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.modes {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
  padding: 2px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-muted);
}

.mode {
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.mode.active {
  background: var(--color-surface);
  color: var(--color-text);
  box-shadow: var(--shadow-sm);
}

.mode:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 1px;
}

.actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex: 1;
  min-width: 0;
  justify-content: flex-end;
}

.search-wrap {
  position: relative;
  flex: 1;
  max-width: 360px;
  min-width: 120px;
}

.search {
  width: 100%;
  padding: 8px 28px 8px 10px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font: inherit;
  color: var(--color-text);
  background: var(--color-surface);
  box-sizing: border-box;
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.search:focus {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: var(--focus-ring);
}

.clear {
  position: absolute;
  right: 4px;
  top: 50%;
  transform: translateY(-50%);
  border: none;
  background: transparent;
  color: var(--color-muted);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  padding: 2px 6px;
}

.clear:hover {
  color: var(--color-text);
}
</style>
