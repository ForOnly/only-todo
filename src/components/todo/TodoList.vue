<script setup lang="ts">
import AppButton from "@/components/common/AppButton.vue";
import {
  PRIORITY_LABELS,
  STATUS_LABELS,
  type TodoDto,
} from "@/api/types";
import { formatDate, formatDueDate } from "@/utils/date";

defineProps<{
  todos: TodoDto[];
  selectedId: string | null;
  loading: boolean;
  total: number;
  page: number;
  pageSize: number;
}>();

const emit = defineEmits<{
  select: [id: string];
  prevPage: [];
  nextPage: [];
}>();
</script>

<template>
  <section class="list-panel">
    <div v-if="loading" class="hint">加载中...</div>
    <ul v-else class="todo-list">
      <li
        v-for="todo in todos"
        :key="todo.id"
        class="todo-item"
        :class="{ active: todo.id === selectedId, done: todo.status === 'Done' }"
        @click="emit('select', todo.id)"
      >
        <div class="title-row">
          <span class="title">{{ todo.title }}</span>
          <span class="badge priority">{{ PRIORITY_LABELS[todo.priority] }}</span>
        </div>
        <div class="meta">
          <span>{{ STATUS_LABELS[todo.status] }}</span>
          <span v-if="todo.dueDate">截止 {{ formatDueDate(todo.dueDate) }}</span>
          <span v-if="todo.tags.length">{{ todo.tags.join(", ") }}</span>
          <span>{{ formatDate(todo.updatedAt) }}</span>
        </div>
      </li>
      <li v-if="todos.length === 0" class="hint">暂无任务</li>
    </ul>
    <footer class="pagination">
      <AppButton :disabled="page <= 1" @click="emit('prevPage')">
        上一页
      </AppButton>
      <span>{{ page }} / {{ Math.max(1, Math.ceil(total / pageSize)) }}</span>
      <AppButton :disabled="page * pageSize >= total" @click="emit('nextPage')">
        下一页
      </AppButton>
    </footer>
  </section>
</template>

<style scoped>
.list-panel {
  display: flex;
  flex-direction: column;
  background: #fff;
  min-width: 0;
  flex: 1;
  height: 100%;
}

.todo-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
}

.todo-item {
  padding: 12px 16px;
  border-bottom: 1px solid #f3f4f6;
  cursor: pointer;
}

.todo-item:hover,
.todo-item.active {
  background: #eff6ff;
}

.todo-item.done .title {
  text-decoration: line-through;
  color: #9ca3af;
}

.title-row {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.title {
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.badge {
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 4px;
  background: #f3f4f6;
  flex-shrink: 0;
}

.meta {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
  font-size: 12px;
  color: #6b7280;
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-top: 1px solid #e5e7eb;
}

.hint {
  padding: 24px;
  color: #6b7280;
  text-align: center;
}
</style>
