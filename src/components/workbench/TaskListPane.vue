<script setup lang="ts">
import { computed } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import {
  PRIORITY_LABELS,
  STATUS_LABELS,
  type SortBy,
  type SortOrder,
  type TodoDto,
  type WorkbenchView,
} from "@/api/types";
import { formatDueDate, isOverdue } from "@/utils/date";
import { sortOptionsForView } from "@/utils/listSort";

const props = defineProps<{
  todos: TodoDto[];
  selectedId: string | null;
  loading: boolean;
  total: number;
  page: number;
  pageSize: number;
  view: WorkbenchView;
  activeTag: string | null;
  sortBy: SortBy;
  sortOrder: SortOrder;
}>();

const emit = defineEmits<{
  select: [id: string];
  toggleComplete: [todo: TodoDto];
  prevPage: [];
  nextPage: [];
  changeSort: [sortBy: SortBy];
  emptyAction: [action: "create" | "all"];
}>();

const sortOptions = computed(() => sortOptionsForView(props.view));

const viewTitle = computed(() => {
  if (props.view === "tag" && props.activeTag) {
    return `标签 · ${props.activeTag}`;
  }
  const titles: Record<WorkbenchView, string> = {
    today: "今日",
    overdue: "逾期",
    doing: "进行中",
    all: "全部",
    done: "已完成",
    archived: "归档",
    trash: "回收站",
    tag: "标签",
  };
  return titles[props.view];
});

function emptyCopy(): { title: string; hint: string; cta?: "create" | "all" } {
  switch (props.view) {
    case "today":
      return {
        title: "今天还没有任务",
        hint: "添加一条今日任务，或从全部中挑选。今日列表最多展示 200 条合并结果。",
        cta: "create",
      };
    case "overdue":
      return { title: "没有逾期任务", hint: "保持这个状态就很好。", cta: "all" };
    case "doing":
      return { title: "没有进行中的任务", hint: "在详情里点「开始」即可进入此视图。", cta: "all" };
    case "trash":
      return { title: "回收站为空", hint: "删除的任务会出现在这里。" };
    case "archived":
      return { title: "暂无归档", hint: "归档的任务会集中在此。" };
    case "done":
      return { title: "还没有完成记录", hint: "完成任务后会出现在这里。" };
    default:
      return { title: "暂无任务", hint: "按 n 或点新建开始。", cta: "create" };
  }
}

function canToggle(todo: TodoDto): boolean {
  return !todo.deletedAt && (todo.status === "Todo" || todo.status === "Doing" || todo.status === "Done");
}

function onRowClick(todo: TodoDto) {
  emit("select", todo.id);
}

function onCheck(todo: TodoDto, event: Event) {
  event.stopPropagation();
  if (!canToggle(todo)) return;
  emit("toggleComplete", todo);
}
</script>

<template>
  <section class="list-pane">
    <header class="list-header">
      <div class="title-block">
        <h2>{{ viewTitle }}</h2>
        <span class="count">{{ total }}</span>
      </div>
      <label class="sort">
        <span>排序</span>
        <select
          :value="sortBy"
          @change="emit('changeSort', ($event.target as HTMLSelectElement).value as SortBy)"
        >
          <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </label>
    </header>

    <div v-if="loading" class="hint">加载中...</div>

    <ul v-else class="todo-list">
      <li v-if="todos.length === 0" class="empty">
        <p class="empty-title">{{ emptyCopy().title }}</p>
        <p class="empty-hint">{{ emptyCopy().hint }}</p>
        <div v-if="emptyCopy().cta" class="empty-actions">
          <AppButton
            v-if="emptyCopy().cta === 'create'"
            variant="primary"
            @click="emit('emptyAction', 'create')"
          >
            添加今日任务
          </AppButton>
          <AppButton v-else variant="ghost" @click="emit('emptyAction', 'all')">
            查看全部
          </AppButton>
        </div>
      </li>

      <li
        v-for="todo in todos"
        :key="todo.id"
        class="todo-item"
        :class="{
          active: todo.id === selectedId,
          done: todo.status === 'Done',
          overdue: isOverdue(todo.dueDate, todo.status),
        }"
        @click="onRowClick(todo)"
      >
        <span class="priority-bar" :data-priority="todo.priority" />
        <input
          class="check"
          type="checkbox"
          :checked="todo.status === 'Done'"
          :disabled="!canToggle(todo)"
          :aria-label="todo.status === 'Done' ? '标记未完成' : '标记完成'"
          @click="onCheck(todo, $event)"
        />
        <div class="body">
          <div class="title-row">
            <span class="title">{{ todo.title }}</span>
            <span v-if="todo.status === 'Doing'" class="doing-mark">进行中</span>
          </div>
          <div class="meta">
            <span class="priority">{{ PRIORITY_LABELS[todo.priority] }}</span>
            <span>{{ STATUS_LABELS[todo.status] }}</span>
            <span
              v-if="todo.dueDate"
              :class="{ 'due-overdue': isOverdue(todo.dueDate, todo.status) }"
            >
              {{ formatDueDate(todo.dueDate) }}
            </span>
            <span v-for="tag in todo.tags" :key="tag" class="chip">{{ tag }}</span>
          </div>
        </div>
      </li>
    </ul>

    <footer v-if="view !== 'today' && total > pageSize" class="pagination">
      <AppButton :disabled="page <= 1" @click="emit('prevPage')">上一页</AppButton>
      <span>{{ page }} / {{ Math.max(1, Math.ceil(total / pageSize)) }}</span>
      <AppButton :disabled="page * pageSize >= total" @click="emit('nextPage')">下一页</AppButton>
    </footer>
  </section>
</template>

<style scoped>
.list-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: #fff;
  border-right: 1px solid #e2e8f0;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid #e2e8f0;
}

.title-block {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.title-block h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 650;
  color: #0f172a;
}

.count {
  font-size: 12px;
  color: #94a3b8;
}

.sort {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #64748b;
}

.sort select {
  padding: 4px 8px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font: inherit;
  background: #fff;
}

.todo-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
}

.todo-item {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 16px 12px 14px;
  border-bottom: 1px solid #f1f5f9;
  cursor: pointer;
}

.todo-item:hover {
  background: #f8fafc;
}

.todo-item.active {
  background: #f1f5f9;
}

.todo-item.overdue .title {
  color: #b91c1c;
}

.todo-item.done .title {
  text-decoration: line-through;
  color: #94a3b8;
}

.priority-bar {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: #cbd5e1;
}

.priority-bar[data-priority="Urgent"] {
  background: #dc2626;
}
.priority-bar[data-priority="High"] {
  background: #ea580c;
}
.priority-bar[data-priority="Medium"] {
  background: #2563eb;
}
.priority-bar[data-priority="Low"] {
  background: #94a3b8;
}

.check {
  margin-top: 3px;
  flex-shrink: 0;
  cursor: pointer;
}

.body {
  min-width: 0;
  flex: 1;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title {
  font-weight: 600;
  color: #0f172a;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.doing-mark {
  flex-shrink: 0;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: #dbeafe;
  color: #1d4ed8;
}

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: #64748b;
}

.due-overdue {
  color: #dc2626;
  font-weight: 600;
}

.chip {
  padding: 1px 6px;
  border-radius: 4px;
  background: #f1f5f9;
  color: #475569;
}

.empty {
  padding: 48px 24px;
  text-align: center;
}

.empty-title {
  margin: 0 0 6px;
  font-size: 15px;
  font-weight: 600;
  color: #334155;
}

.empty-hint {
  margin: 0 0 16px;
  font-size: 13px;
  color: #94a3b8;
}

.empty-actions {
  display: flex;
  justify-content: center;
  gap: 8px;
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-top: 1px solid #e2e8f0;
  font-size: 13px;
  color: #64748b;
}

.hint {
  padding: 24px;
  color: #64748b;
  text-align: center;
}
</style>
