<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import type { EventDto } from "@/api/types";
import {
  eventsNewestFirst,
  formatActivityLine,
  formatActivitySummary,
} from "@/utils/i18nFormat";

const props = defineProps<{
  events: EventDto[];
}>();

const { t } = useI18n();
const collapsed = ref(true);

const ordered = computed(() => eventsNewestFirst(props.events));
const latest = computed(() => ordered.value[0] ?? null);
const summary = computed(() =>
  latest.value ? formatActivitySummary(latest.value, t) : "",
);
const lines = computed(() => ordered.value.map((event) => formatActivityLine(event, t)));
</script>

<template>
  <aside v-if="events.length" class="activity" :aria-label="$t('activity.aria')">
    <header class="activity-header">
      <p v-if="collapsed" class="activity-summary" :title="summary">{{ summary }}</p>
      <h2 v-else class="activity-title">{{ $t("activity.title") }}</h2>
      <button
        type="button"
        class="activity-toggle"
        :aria-expanded="!collapsed"
        @click="collapsed = !collapsed"
      >
        {{ collapsed ? $t("activity.expand") : $t("activity.collapse") }}
      </button>
    </header>
    <ul v-show="!collapsed" class="activity-list">
      <li
        v-for="line in lines"
        :key="line.id"
        class="activity-item"
        :class="{ 'has-title': line.title }"
      >
        <span class="activity-action">{{ line.action }}</span>
        <span v-if="line.title" class="activity-task" :title="line.title">{{ line.title }}</span>
        <span class="activity-time">{{ line.time }}</span>
      </li>
    </ul>
  </aside>
</template>

<style scoped>
.activity {
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
  padding: 6px 16px;
  flex-shrink: 0;
}

.activity-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 20px;
}

.activity-summary,
.activity-title {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.activity-title {
  font-weight: 600;
  color: var(--color-muted);
}

.activity-toggle {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 4px;
  flex-shrink: 0;
  transition: color var(--transition-fast);
}

.activity-toggle:hover {
  color: var(--color-accent-hover);
}

.activity-list {
  margin: 6px 0 2px;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 196px;
  overflow: auto;
}

.activity-item {
  display: flex;
  gap: 8px;
  align-items: baseline;
  font-size: 13px;
  line-height: 1.5;
  color: var(--color-text);
}

.activity-action {
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  flex-shrink: 0;
}

.activity-task {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-secondary);
}

.activity-time {
  color: var(--color-muted);
  font-size: 12px;
  white-space: nowrap;
  flex-shrink: 0;
}

.activity-item.has-title .activity-time {
  margin-left: auto;
}
</style>
