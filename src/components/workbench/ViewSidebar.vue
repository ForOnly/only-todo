<script setup lang="ts">
import type { WorkbenchView } from "@/api/types";
import { LIBRARY_FOOTER_VIEWS, LIBRARY_MAIN_VIEWS } from "@/constants/workbenchViews";

const props = defineProps<{
  view: WorkbenchView;
  activeTag: string | null;
  allTags: string[];
}>();

const emit = defineEmits<{
  selectView: [view: WorkbenchView, tag?: string | null];
}>();

function isActive(id: WorkbenchView): boolean {
  return props.view === id;
}

function isTagActive(tag: string): boolean {
  return props.view === "tag" && props.activeTag === tag;
}
</script>

<template>
  <aside class="view-sidebar" :aria-label="$t('views.navLabel')">
    <div class="sidebar-scroll">
      <nav class="nav">
        <button
          v-for="id in LIBRARY_MAIN_VIEWS"
          :key="id"
          type="button"
          class="nav-item"
          :class="{ active: isActive(id) }"
          @click="emit('selectView', id)"
        >
          <span>{{ $t(`views.${id}`) }}</span>
        </button>
      </nav>

      <div v-if="allTags.length" class="tags-block">
        <h3 class="tags-title">{{ $t("views.tags") }}</h3>
        <button
          v-for="tag in allTags"
          :key="tag"
          type="button"
          class="nav-item tag"
          :class="{ active: isTagActive(tag) }"
          :title="tag"
          @click="emit('selectView', 'tag', tag)"
        >
          {{ tag }}
        </button>
      </div>
    </div>

    <nav class="nav nav-footer">
      <button
        v-for="id in LIBRARY_FOOTER_VIEWS"
        :key="id"
        type="button"
        class="nav-item"
        :class="{ active: isActive(id) }"
        @click="emit('selectView', id)"
      >
        <span>{{ $t(`views.${id}`) }}</span>
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.view-sidebar {
  width: 184px;
  flex-shrink: 0;
  align-self: stretch;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-border);
  background: var(--color-surface-muted);
  padding: 12px 8px 0;
}

.sidebar-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-footer {
  flex-shrink: 0;
  margin-top: auto;
  padding: 12px 0 12px;
  border-top: 1px solid var(--color-border);
  background: var(--color-surface-muted);
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.nav-item:hover {
  background: var(--color-bg-accent);
}

.nav-item:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 1px;
}

.nav-item.active {
  background: var(--color-accent-soft);
  color: var(--color-text);
  font-weight: 600;
}

.tags-block {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.tags-title {
  margin: 0 0 6px 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-muted);
}

.nav-item.tag {
  color: var(--color-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}
</style>
