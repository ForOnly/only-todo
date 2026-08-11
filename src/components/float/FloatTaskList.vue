<script setup lang="ts">
import { PRIORITY_LABELS, STATUS_LABELS, type TodoDto } from "@/api/types";
import { formatDueDate } from "@/utils/date";

defineProps<{
  todos: TodoDto[];
  selectedId: string | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  openMain: [id: string];
}>();
</script>

<template>
  <div class="task-list">
    <div v-if="loading" class="hint">加载中...</div>
    <ul v-else>
      <li
        v-for="todo in todos"
        :key="todo.id"
        :class="{ selected: selectedId === todo.id }"
        @click="emit('select', todo.id)"
        @dblclick.stop="emit('openMain', todo.id)"
      >
        <span class="priority" :data-level="todo.priority" />
        <div class="content">
          <span class="name">{{ todo.title }}</span>
          <span class="meta">
            {{ STATUS_LABELS[todo.status] }} · {{ PRIORITY_LABELS[todo.priority] }}
            <template v-if="todo.dueDate"> · {{ formatDueDate(todo.dueDate) }}</template>
          </span>
        </div>
      </li>
      <li v-if="todos.length === 0" class="hint">暂无待办</li>
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
}

li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
}

li:hover,
li.selected {
  background: #f3f4f6;
}

.priority {
  width: 4px;
  height: 32px;
  border-radius: 2px;
  flex-shrink: 0;
  background: #9ca3af;
}

.priority[data-level="Urgent"] {
  background: #dc2626;
}

.priority[data-level="High"] {
  background: #f97316;
}

.priority[data-level="Medium"] {
  background: #3b82f6;
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
}

.meta {
  display: block;
  font-size: 11px;
  color: #6b7280;
  margin-top: 2px;
}

.hint {
  padding: 16px 12px;
  color: #6b7280;
  font-size: 13px;
}
</style>
