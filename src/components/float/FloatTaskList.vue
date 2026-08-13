<script setup lang="ts">
import { computed } from "vue";

import type { TodoDto } from "@/api/types";
import { formatDueDate, isOverdue } from "@/utils/date";

const props = defineProps<{
  todos: TodoDto[];
  selectedId: string | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  openMain: [id: string];
  complete: [id: string];
}>();

const showLoadingHint = computed(() => props.loading && props.todos.length === 0);
const isRefreshing = computed(() => props.loading && props.todos.length > 0);
</script>

<template>
  <div class="task-list">
    <div v-if="showLoadingHint" class="hint">{{ $t("common.loading") }}</div>
    <ul v-else :class="{ refreshing: isRefreshing }">
      <li
        v-for="todo in todos"
        :key="todo.id"
        :class="{ selected: selectedId === todo.id, overdue: isOverdue(todo.dueDate, todo.status) }"
        @click="emit('select', todo.id)"
        @dblclick.stop="emit('openMain', todo.id)"
      >
        <button
          type="button"
          class="check"
          :title="$t('companion.markComplete')"
          :aria-label="$t('companion.markComplete')"
          @click.stop="emit('complete', todo.id)"
        />
        <span class="priority" :data-level="todo.priority" />
        <div class="content">
          <span class="name">{{ todo.title }}</span>
          <span class="meta">
            {{ $t(`status.${todo.status}`) }} · {{ $t(`priority.${todo.priority}`) }}
            <template v-if="todo.dueDate">
              ·
              <span :class="{ 'due-overdue': isOverdue(todo.dueDate, todo.status) }">
                {{ formatDueDate(todo.dueDate) }}
              </span>
            </template>
          </span>
        </div>
      </li>
      <li v-if="todos.length === 0" class="hint">{{ $t("companion.emptyList") }}</li>
    </ul>
  </div>
</template>

<style scoped>
.task-list {
  flex: 1;
  overflow-y: auto;
}

ul {
  list-style: none;
  margin: 0;
  padding: 0;
  transition: opacity var(--transition-fast);
}

ul.refreshing {
  opacity: 0.72;
}

li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  transition: background var(--transition-fast);
}

li:hover,
li.selected {
  background: var(--color-accent-soft);
}

.check {
  width: 16px;
  height: 16px;
  margin-top: 8px;
  flex-shrink: 0;
  border: 1.5px solid var(--color-muted);
  border-radius: 4px;
  background: var(--color-surface);
  cursor: pointer;
  padding: 0;
  transition:
    border-color var(--transition-fast),
    background var(--transition-fast);
}

.check:hover {
  border-color: var(--color-accent);
  background: var(--color-accent-soft);
}

.priority {
  width: 4px;
  height: 32px;
  border-radius: 2px;
  flex-shrink: 0;
  background: var(--color-priority-low);
}

.priority[data-level="Urgent"] {
  background: var(--color-priority-urgent);
}

.priority[data-level="High"] {
  background: var(--color-priority-high);
}

.priority[data-level="Medium"] {
  background: var(--color-priority-medium);
}

.content {
  min-width: 0;
}

.name {
  display: block;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text);
}

.meta {
  display: block;
  font-size: 11px;
  color: var(--color-muted);
  margin-top: 2px;
}

.due-overdue,
li.overdue .name {
  color: var(--color-overdue);
}

.hint {
  padding: 16px 12px;
  color: var(--color-muted);
  font-size: 13px;
}
</style>
