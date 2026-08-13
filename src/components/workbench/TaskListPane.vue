<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/common/AppButton.vue";
import type { SortBy, SortOrder, TodoDto, WorkbenchView } from "@/api/types";
import { formatDueDate, isOverdue } from "@/utils/date";
import { sortKeysForView } from "@/utils/listSort";

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
  toggleSortOrder: [];
  emptyAction: [action: "create" | "all"];
}>();

const { t } = useI18n();

const sortOptions = computed(() => sortKeysForView(props.view));

const viewTitle = computed(() => {
  if (props.view === "tag" && props.activeTag) {
    return t("views.tagTitle", { tag: props.activeTag });
  }
  return t(`views.${props.view}`);
});

const emptyCopy = computed(() => {
  switch (props.view) {
    case "today":
      return {
        title: t("list.empty.todayTitle"),
        hint: t("list.empty.todayHint"),
        cta: "create" as const,
        ctaLabel: t("list.empty.todayCta"),
      };
    case "overdue":
      return {
        title: t("list.empty.overdueTitle"),
        hint: t("list.empty.overdueHint"),
        cta: "all" as const,
      };
    case "doing":
      return {
        title: t("list.empty.doingTitle"),
        hint: t("list.empty.doingHint"),
        cta: "all" as const,
      };
    case "trash":
      return { title: t("list.empty.trashTitle"), hint: t("list.empty.trashHint") };
    case "archived":
      return { title: t("list.empty.archivedTitle"), hint: t("list.empty.archivedHint") };
    case "done":
      return { title: t("list.empty.doneTitle"), hint: t("list.empty.doneHint") };
    default:
      return {
        title: t("list.empty.defaultTitle"),
        hint: t("list.empty.defaultHint"),
        cta: "create" as const,
        ctaLabel: t("list.empty.defaultCta"),
      };
  }
});

const showLoadingHint = computed(() => props.loading && props.todos.length === 0);
const isRefreshing = computed(() => props.loading && props.todos.length > 0);

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
      <div class="sort-controls">
        <label class="sort">
          <span>{{ $t("list.sort") }}</span>
          <select
            :value="sortBy"
            @change="emit('changeSort', ($event.target as HTMLSelectElement).value as SortBy)"
          >
            <option v-for="key in sortOptions" :key="key" :value="key">
              {{ $t(`sort.${key}`) }}
            </option>
          </select>
        </label>
        <button
          type="button"
          class="sort-order-btn"
          :title="$t('list.toggleSortOrder')"
          :aria-label="sortOrder === 'asc' ? $t('list.sortAsc') : $t('list.sortDesc')"
          @click="emit('toggleSortOrder')"
        >
          {{ sortOrder === "asc" ? "↑" : "↓" }}
        </button>
      </div>
    </header>

    <div v-if="showLoadingHint" class="hint">{{ $t("common.loading") }}</div>

    <ul v-else class="todo-list" :class="{ refreshing: isRefreshing }">
      <li v-if="todos.length === 0" class="empty">
        <p class="empty-title">{{ emptyCopy.title }}</p>
        <p class="empty-hint">{{ emptyCopy.hint }}</p>
        <div v-if="emptyCopy.cta" class="empty-actions">
          <AppButton
            v-if="emptyCopy.cta === 'create'"
            variant="primary"
            @click="emit('emptyAction', 'create')"
          >
            {{ emptyCopy.ctaLabel }}
          </AppButton>
          <AppButton v-else variant="ghost" @click="emit('emptyAction', 'all')">
            {{ $t("list.viewAll") }}
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
          :aria-label="todo.status === 'Done' ? $t('list.markUndone') : $t('list.markDone')"
          @click="onCheck(todo, $event)"
        />
        <div class="body">
          <div class="title-row">
            <span class="title">{{ todo.title }}</span>
            <span v-if="todo.status === 'Doing'" class="doing-mark">{{ $t("views.doing") }}</span>
          </div>
          <div class="meta">
            <span class="priority">{{ $t(`priority.${todo.priority}`) }}</span>
            <span>{{ $t(`status.${todo.status}`) }}</span>
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
      <AppButton :disabled="page <= 1" @click="emit('prevPage')">{{ $t("list.prevPage") }}</AppButton>
      <span>{{ page }} / {{ Math.max(1, Math.ceil(total / pageSize)) }}</span>
      <AppButton :disabled="page * pageSize >= total" @click="emit('nextPage')">
        {{ $t("list.nextPage") }}
      </AppButton>
    </footer>
  </section>
</template>

<style scoped>
.list-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
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
  color: var(--color-text);
}

.count {
  font-size: 12px;
  color: var(--color-muted);
}

.sort-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.sort {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-muted);
}

.sort select {
  padding: 4px 8px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font: inherit;
  color: var(--color-text);
  background: var(--color-surface);
}

.sort-order-btn {
  padding: 4px 8px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  font: inherit;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.sort-order-btn:hover {
  background: var(--color-surface-muted);
}

.todo-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
  transition: opacity var(--transition-fast);
}

.todo-list.refreshing {
  opacity: 0.72;
}

.todo-item {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 16px 12px 14px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  transition: background 0.15s ease;
}

.todo-item:hover {
  background: var(--color-surface-muted);
}

.todo-item.active {
  background: var(--color-accent-soft);
}

.todo-item.overdue .title {
  color: var(--color-overdue);
}

.todo-item.done .title {
  text-decoration: line-through;
  color: var(--color-muted);
}

.priority-bar {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--color-priority-low);
}

.priority-bar[data-priority="Urgent"] {
  background: var(--color-priority-urgent);
}
.priority-bar[data-priority="High"] {
  background: var(--color-priority-high);
}
.priority-bar[data-priority="Medium"] {
  background: var(--color-priority-medium);
}
.priority-bar[data-priority="Low"] {
  background: var(--color-priority-low);
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
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.doing-mark {
  flex-shrink: 0;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--color-accent-soft);
  color: var(--color-accent);
}

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-muted);
}

.due-overdue {
  color: var(--color-overdue);
  font-weight: 600;
}

.chip {
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--color-surface-muted);
  color: var(--color-text-secondary);
}

.empty {
  padding: 48px 24px;
  text-align: center;
}

.empty-title {
  margin: 0 0 6px;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.empty-hint {
  margin: 0 0 16px;
  font-size: 13px;
  color: var(--color-muted);
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
  border-top: 1px solid var(--color-border);
  font-size: 13px;
  color: var(--color-muted);
}

.hint {
  padding: 24px;
  color: var(--color-muted);
  text-align: center;
}
</style>
