<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppTagChip from "@/components/common/AppTagChip.vue";
import type { TodoDto } from "@/api/types";
import { formatAbsoluteDateTime, formatRelativeAge, formatRelativeDue } from "@/utils/timeMeta";

const props = defineProps<{
  todo: TodoDto;
}>();

const emit = defineEmits<{
  edit: [];
}>();

const { t } = useI18n();

const hasDescription = computed(() => props.todo.description?.trim() !== "");

const dueLabel = computed(() => {
  if (!props.todo.dueDate) return null;
  return formatRelativeDue(props.todo.dueDate, props.todo.status, t);
});

const createdLabel = computed(() => {
  return formatRelativeAge(props.todo.createdAt, t) ?? formatAbsoluteDateTime(props.todo.createdAt);
});

const statusLabel = computed(() => t(`status.${props.todo.status}`));
</script>

<template>
  <section class="peek" @click.stop @dblclick.stop="emit('edit')">
    <div class="peek-head">
      <span class="status-pill" :data-status="todo.status">{{ statusLabel }}</span>
      <span class="priority-tag" :data-priority="todo.priority">{{
        $t(`priority.${todo.priority}`)
      }}</span>
      <div v-if="todo.tags.length" class="head-tags">
        <AppTagChip v-for="tag in todo.tags" :key="tag" :label="tag" />
      </div>
      <button type="button" class="edit-btn" @click.stop="emit('edit')">
        {{ $t("peek.edit") }}
      </button>
    </div>

    <div class="peek-meta">
      <span
        v-if="dueLabel"
        class="meta-item due"
        :class="{
          overdue:
            todo.status !== 'Done' && todo.dueDate && new Date(todo.dueDate).getTime() < Date.now(),
        }"
      >
        {{ $t("inspector.dueDate") }}: {{ dueLabel }}
      </span>
      <span class="meta-item sep">·</span>
      <span class="meta-item">{{ $t("inspector.createdAt") }}: {{ createdLabel }}</span>
    </div>

    <p class="peek-desc" :class="{ empty: !hasDescription }">
      {{ hasDescription ? todo.description : $t("peek.noDescription") }}
    </p>
  </section>
</template>

<style scoped>
.peek {
  margin: 4px 16px 8px 27px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  background: var(--color-surface-muted);
  border: 1px solid var(--color-border);
}

.peek-head {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 6px;
}

.head-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.status-pill {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: var(--color-surface);
  border: 1px solid var(--color-border-strong);
  color: var(--color-text-secondary);
}

.status-pill[data-status="Doing"] {
  background: var(--color-accent-soft);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.status-pill[data-status="Done"] {
  background: var(--color-surface-muted);
  border-color: var(--color-success);
  color: var(--color-success);
}

.priority-tag {
  font-size: 11px;
  color: var(--color-muted);
}

.priority-tag[data-priority="Urgent"] {
  color: var(--color-priority-urgent);
}

.priority-tag[data-priority="High"] {
  color: var(--color-priority-high);
}

.priority-tag[data-priority="Medium"] {
  color: var(--color-priority-medium);
}

.peek-meta {
  font-size: 12px;
  color: var(--color-muted);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.peek-meta .meta-item.sep {
  color: var(--color-border-strong);
}

.peek-meta .due.overdue {
  color: var(--color-overdue);
  font-weight: 600;
}

.peek-desc {
  margin: 0 0 8px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
}

.peek-desc.empty {
  color: var(--color-muted);
  font-style: italic;
  font-size: 12px;
}

.edit-btn {
  flex-shrink: 0;
  padding: 3px 10px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-accent);
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  margin-left: auto;
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast);
}

.edit-btn:hover {
  background: var(--color-accent-soft);
  border-color: var(--color-accent);
}
</style>
