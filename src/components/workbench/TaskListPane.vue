<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/common/AppButton.vue";
import AppSelect from "@/components/common/AppSelect.vue";
import AppTagChip from "@/components/common/AppTagChip.vue";
import type { FocusBoardDto, SortBy, SortOrder, TodoDto, WorkbenchView } from "@/api/types";
import { formatRelativeAge, formatRelativeDue, getOverdueDaysOnly } from "@/utils/timeMeta";
import { sortKeysForView } from "@/utils/listSort";

const props = defineProps<{
  todos: TodoDto[];
  focusBoard: FocusBoardDto;
  selectedId: string | null;
  loading: boolean;
  total: number;
  page: number;
  pageSize: number;
  view: WorkbenchView;
  activeTag: string | null;
  sortBy: SortBy;
  sortOrder: SortOrder;
  grouped: boolean;
  searching: boolean;
  showInlineAdd: boolean;
  inlinePlaceholder: string;
}>();

const emit = defineEmits<{
  select: [id: string];
  toggleComplete: [todo: TodoDto];
  prevPage: [];
  nextPage: [];
  changeSort: [sortBy: SortBy];
  toggleSortOrder: [];
  quickAdd: [title: string];
}>();

const { t } = useI18n();
const quickTitle = ref("");

interface ListGroup {
  id: string;
  title: string | null;
  items: TodoDto[];
}

const sortOptions = computed(() => sortKeysForView(props.view));

const sortSelectOptions = computed(() =>
  sortOptions.value.map((key) => ({ value: key, label: t(`sort.${key}`) })),
);

const sortByModel = computed({
  get: () => props.sortBy,
  set: (v: string) => emit("changeSort", v as SortBy),
});

const viewTitle = computed(() => {
  if (props.searching) return t("list.searchTitle");
  if (props.view === "tag" && props.activeTag) {
    return t("views.tagTitle", { tag: props.activeTag });
  }
  return t(`views.${props.view}`);
});

const emptyCopy = computed(() => {
  if (props.searching) {
    return { title: t("list.empty.searchTitle"), hint: t("list.empty.searchHint") };
  }
  switch (props.view) {
    case "today":
      return { title: t("list.empty.focusTitle"), hint: t("list.empty.focusHint") };
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
      };
  }
});

const groups = computed<ListGroup[]>(() => {
  if (props.todos.length === 0) return [];

  if (props.grouped && !props.searching) {
    return [
      { id: "overdue", title: t("focus.overdue"), items: props.focusBoard.overdue },
      { id: "doing", title: t("focus.doing"), items: props.focusBoard.doing },
      { id: "dueToday", title: t("focus.dueToday"), items: props.focusBoard.dueToday },
    ].filter((group) => group.items.length > 0);
  }

  return [{ id: "flat", title: null, items: props.todos }];
});

const showLoadingHint = computed(() => props.loading && props.todos.length === 0);
const isRefreshing = computed(() => props.loading && props.todos.length > 0);
const showPagination = computed(() => props.total > props.pageSize);

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

function visibleTags(todo: TodoDto): string[] {
  return todo.tags.slice(0, 2);
}

function extraTagCount(todo: TodoDto): number {
  return Math.max(0, todo.tags.length - 2);
}

function compactRelative(iso: string | null | undefined): string | null {
  const relative = formatRelativeAge(iso, t);
  return relative ? relative.replace(/\s/g, "") : null;
}

function createdTimeToken(todo: TodoDto): string {
  const createdRelative = compactRelative(todo.createdAt);
  return createdRelative ? t("timeMeta.createdAgoSuffix", { time: createdRelative }) : "—";
}

function dueTimeToken(todo: TodoDto): string | null {
  if (!todo.dueDate) return null;
  const overdueDays = getOverdueDaysOnly(todo.dueDate, todo.status);
  if (overdueDays) {
    const daysAgo = t("timeMeta.daysAgo", { n: overdueDays }).replace(/\s/g, "");
    return t("timeMeta.overdueAgoSuffix", { time: daysAgo });
  }
  const dueRelative = formatRelativeDue(todo.dueDate, todo.status, t);
  if (!dueRelative) return null;
  return t("timeMeta.dueAgoSuffix", { time: dueRelative.replace(/\s/g, "") });
}

/** 有截止则「截止/创建」，否则仅创建 */
function listMetaToken(todo: TodoDto): string {
  const created = createdTimeToken(todo);
  const due = dueTimeToken(todo);
  if (due) return `${due}/${created}`;
  return created;
}

function submitQuickAdd() {
  const title = quickTitle.value.trim();
  if (!title) return;
  emit("quickAdd", title);
  quickTitle.value = "";
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
          <AppSelect v-model="sortByModel" :options="sortSelectOptions" />
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
      <li v-if="showInlineAdd" class="quick-add">
        <input
          id="workbench-quick-add"
          v-model="quickTitle"
          type="text"
          maxlength="200"
          :placeholder="inlinePlaceholder"
          @keydown.enter.prevent="submitQuickAdd"
        />
      </li>

      <li v-if="groups.length === 0" class="empty">
        <p class="empty-title">{{ emptyCopy.title }}</p>
        <p class="empty-hint">{{ emptyCopy.hint }}</p>
      </li>

      <template v-for="group in groups" :key="group.id">
        <li v-if="group.title" class="group-head" :data-group="group.id">{{ group.title }}</li>
        <li
          v-for="todo in group.items"
          :key="todo.id"
          class="todo-item"
          tabindex="0"
          :class="{
            active: todo.id === selectedId,
            done: todo.status === 'Done',
            'overdue-row': group.id === 'overdue',
          }"
          @click="onRowClick(todo)"
          @keydown.enter.prevent="onRowClick(todo)"
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
            <div class="title-meta-row">
              <div class="title-row">
                <span class="title">{{ todo.title }}</span>
                <span v-if="todo.status === 'Doing'" class="doing-mark">{{ $t("views.doing") }}</span>
              </div>
              <div class="meta-right">
                <span
                  class="meta-single"
                  :class="{ 'due-overdue': getOverdueDaysOnly(todo.dueDate, todo.status) }"
                >
                  {{ listMetaToken(todo) }}
                </span>
              </div>
            </div>
            <div v-if="todo.tags.length" class="tag-row">
              <AppTagChip
                v-for="tag in visibleTags(todo)"
                :key="tag"
                :label="tag"
              />
              <AppTagChip
                v-if="extraTagCount(todo)"
                :label="$t('list.moreTags', { n: extraTagCount(todo) })"
                more
              />
            </div>
          </div>
        </li>
      </template>
    </ul>

    <footer v-if="showPagination" class="pagination">
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

.sort :deep(.trigger) {
  padding: 4px 8px;
  font-size: 12px;
  min-width: 110px;
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

.group-head {
  padding: 12px 16px 6px;
  font-size: 11px;
  font-weight: 650;
  letter-spacing: 0.04em;
  color: var(--color-muted);
}

.group-head[data-group="overdue"] {
  color: var(--color-overdue);
}

.group-head[data-group="doing"] {
  color: var(--color-accent);
}

.todo-item.overdue-row {
  background: rgba(185, 28, 28, 0.04);
}

.todo-item.overdue-row:hover {
  background: rgba(185, 28, 28, 0.08);
}

.quick-add {
  padding: 8px 16px 8px 14px;
  border-bottom: 1px solid var(--color-border);
}

.quick-add input {
  width: 100%;
  border: none;
  background: transparent;
  font: inherit;
  font-size: 14px;
  color: var(--color-text);
  padding: 6px 0;
  box-sizing: border-box;
}

.quick-add input:focus {
  outline: none;
}

.quick-add input::placeholder {
  color: var(--color-muted);
}

.todo-item {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 16px 10px 14px;
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

.todo-item:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: -2px;
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
  appearance: none;
  width: 16px;
  height: 16px;
  margin-top: 3px;
  flex-shrink: 0;
  cursor: pointer;
  border: 1.5px solid var(--color-border-strong);
  border-radius: 4px;
  background: var(--color-surface);
}

.check:checked {
  background: var(--color-accent);
  border-color: var(--color-accent);
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'><path fill='none' stroke='white' stroke-width='2' d='M3 8.5 6.5 12 13 4'/></svg>");
  background-size: 12px;
  background-position: center;
  background-repeat: no-repeat;
}

.check:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.check:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 1px;
}

.body {
  min-width: 0;
  flex: 1;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
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

.title-meta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.meta-right {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  text-align: right;
  color: var(--color-muted);
  font-size: 12px;
  flex-shrink: 0;
}

.meta-single {
  white-space: nowrap;
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
  white-space: nowrap;
}

.tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  width: 100%;
  justify-content: flex-end;
  margin-top: 4px;
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
  margin: 0;
  font-size: 13px;
  color: var(--color-muted);
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
