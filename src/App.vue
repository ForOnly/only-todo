<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";

import type {
  CreateTodoDto,
  FloatDefaultMode,
  RepeatType,
  SortBy,
  TodoDto,
  WorkbenchView,
} from "@/api/types";
import { listAllTags, listTodos, transitionTodo } from "@/api/todos";
import { getSettings, updateSettings } from "@/api/settings";
import { showFloatingWindow } from "@/api/window";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import CreateTodoModal from "@/components/todo/CreateTodoModal.vue";
import ViewSidebar from "@/components/workbench/ViewSidebar.vue";
import TaskListPane from "@/components/workbench/TaskListPane.vue";
import TaskInspector from "@/components/workbench/TaskInspector.vue";
import { TAURI_EVENTS } from "@/constants/events";
import { useReminders, useTodoDetail } from "@/composables/useTodoDetail";
import { useTodos } from "@/composables/useTodos";
import { dueDateRangeForFilter } from "@/utils/date";
import { formatErrorMessage } from "@/utils/error";
import { parseListDefaultSort } from "@/utils/listSort";

const {
  todos,
  total,
  page,
  pageSize,
  loading,
  error: listError,
  keyword,
  view,
  activeTag,
  sortBy,
  sortOrder,
  selectedId,
  fetchTodos,
  createTodo,
  selectTodo,
  setView,
  setSort,
  search,
  goToPage,
} = useTodos();

const {
  reminders,
  loading: remindersLoading,
  error: remindersError,
  addReminder,
  updateReminder,
  removeReminder,
  snoozeReminder,
} = useReminders(selectedId);

const {
  detail,
  editTitle,
  editDescription,
  editPriority,
  editDueDate,
  editTags,
  saving,
  error: detailError,
  scheduleAutosave,
  flushAutosave,
  reload,
  transitionTo,
  remove,
  restore,
} = useTodoDetail(selectedId, async () => {
  await fetchTodos();
  await refreshOverdueCount();
  await loadTags();
});

const settingsOpen = ref(false);
const createOpen = ref(false);
const notificationEnabled = ref(true);
const listDefaultSortRaw = ref('{"sort_by":"priority","sort_order":"desc"}');
const floatAlwaysOnTop = ref(true);
const floatVisibleCount = ref(5);
const floatAutoShow = ref(true);
const floatDefaultMode = ref<FloatDefaultMode>("ball");
const floatHoverPreview = ref(false);
const autostartEnabled = ref(false);
const allTags = ref<string[]>([]);
const overdueCount = ref(0);
const createModalRef = ref<InstanceType<typeof CreateTodoModal> | null>(null);
const reminderError = ref<string | null>(null);
const eventUnlisteners: (() => void)[] = [];

onMounted(async () => {
  try {
    const settings = await getSettings();
    notificationEnabled.value = settings.notificationEnabled;
    listDefaultSortRaw.value = settings.listDefaultSort;
    floatAlwaysOnTop.value = settings.floatAlwaysOnTop;
    floatVisibleCount.value = settings.floatVisibleCount;
    floatAutoShow.value = settings.floatAutoShow;
    floatDefaultMode.value = settings.floatDefaultMode;
    floatHoverPreview.value = settings.floatHoverPreview;
    autostartEnabled.value = settings.autostartEnabled;
    const sort = parseListDefaultSort(settings.listDefaultSort);
    sortBy.value = sort.sortBy;
    sortOrder.value = sort.sortOrder;
  } catch {
    // 首次启动时 settings 表可能尚未写入种子数据，忽略即可
  }

  await fetchTodos();
  await loadTags();
  await refreshOverdueCount();

  eventUnlisteners.push(
    await listen<string>(TAURI_EVENTS.NAVIGATE_TO_TODO, (event) => {
      void navigateToTodo(event.payload);
    }),
  );

  eventUnlisteners.push(
    await listen(TAURI_EVENTS.CREATE_TODO, () => {
      createOpen.value = true;
    }),
  );

  eventUnlisteners.push(
    await listen(TAURI_EVENTS.OPEN_SETTINGS, () => {
      settingsOpen.value = true;
    }),
  );

  window.addEventListener("keydown", onGlobalKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
  for (const unlisten of eventUnlisteners) {
    unlisten();
  }
});

async function loadTags() {
  try {
    allTags.value = await listAllTags();
  } catch {
    allTags.value = [];
  }
}

async function refreshOverdueCount() {
  try {
    const dueRange = dueDateRangeForFilter("overdue");
    const result = await listTodos({
      status: ["Todo", "Doing"],
      dueDateBefore: dueRange.before,
      page: 1,
      pageSize: 1,
    });
    overdueCount.value = result.total;
  } catch {
    overdueCount.value = 0;
  }
}

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

function onGlobalKeydown(event: KeyboardEvent) {
  if (settingsOpen.value || createOpen.value) {
    if (event.key === "Escape") {
      settingsOpen.value = false;
      createOpen.value = false;
    }
    return;
  }

  if (event.key === "Escape") {
    if (selectedId.value) {
      event.preventDefault();
      void closeDetail();
    }
    return;
  }

  if (event.key === "/" && !isTypingTarget(event.target)) {
    event.preventDefault();
    document.getElementById("workbench-search")?.focus();
    return;
  }

  if (isTypingTarget(event.target)) return;

  if (event.key === "n") {
    event.preventDefault();
    createOpen.value = true;
    return;
  }

  if (event.key === "j" || event.key === "k") {
    event.preventDefault();
    void moveSelection(event.key === "j" ? 1 : -1);
    return;
  }

  if (event.key === "x" && selectedId.value && !detail.value?.deletedAt) {
    event.preventDefault();
    void handleRemove();
  }
}

async function moveSelection(delta: number) {
  if (!todos.value.length) return;
  const ids = todos.value.map((t) => t.id);
  const current = selectedId.value ? ids.indexOf(selectedId.value) : -1;
  let next = current + delta;
  if (current < 0) next = delta > 0 ? 0 : ids.length - 1;
  if (next < 0) next = 0;
  if (next >= ids.length) next = ids.length - 1;
  await navigateToTodo(ids[next] ?? null);
}

async function navigateToTodo(id: string | null) {
  await flushAutosave();
  await selectTodo(id);
}

function openCreateModal() {
  createOpen.value = true;
}

async function closeDetail() {
  await flushAutosave();
  selectedId.value = null;
}

async function handleSelect(id: string) {
  if (selectedId.value === id) {
    await closeDetail();
    return;
  }
  await navigateToTodo(id);
}

async function handleToggleComplete(todo: TodoDto) {
  try {
    const next = todo.status === "Done" ? "Todo" : "Done";
    await transitionTodo(todo.id, next);
    if (selectedId.value === todo.id) {
      await reload();
    }
    await fetchTodos();
    await refreshOverdueCount();
  } catch (err) {
    listError.value = formatErrorMessage(err);
  }
}

async function handleRemove() {
  const ok = await remove();
  if (ok) {
    await refreshOverdueCount();
    await loadTags();
  }
}

async function handleRestore() {
  const restoredId = selectedId.value;
  const ok = await restore();
  if (ok && restoredId) {
    view.value = "all";
    activeTag.value = null;
    page.value = 1;
    await fetchTodos();
    await selectTodo(restoredId);
    await refreshOverdueCount();
    await loadTags();
  }
}

async function handleCreateSubmit(dto: CreateTodoDto) {
  try {
    await createTodo(dto);
    createOpen.value = false;
    createModalRef.value?.resetSubmitting();
    await loadTags();
    await refreshOverdueCount();
  } catch (err) {
    createModalRef.value?.setError(formatErrorMessage(err));
  }
}

async function handleAddReminder(datetime: string, repeatType?: RepeatType) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await addReminder(selectedId.value, datetime, repeatType);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleUpdateReminder(id: string, datetime: string) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await updateReminder(id, selectedId.value, datetime);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleRemoveReminder(id: string) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await removeReminder(id, selectedId.value);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleSnoozeReminder(id: string, minutes: number) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await snoozeReminder(id, selectedId.value, minutes);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleViewSelect(next: WorkbenchView, tag?: string | null) {
  await flushAutosave();
  setView(next, tag);
  selectedId.value = null;
}

function handleChangeSort(next: SortBy) {
  setSort(next);
}

function handleEmptyAction(action: "create" | "all") {
  if (action === "create") {
    openCreateModal();
  } else {
    setView("all");
  }
}

async function handleOpenCompanion() {
  try {
    await showFloatingWindow();
  } catch (err) {
    listError.value = formatErrorMessage(err);
  }
}

async function handleSettingsUpdate(payload: {
  notificationEnabled?: boolean;
  listDefaultSort?: string;
  floatAlwaysOnTop?: boolean;
  floatVisibleCount?: number;
  floatAutoShow?: boolean;
  floatDefaultMode?: FloatDefaultMode;
  floatHoverPreview?: boolean;
  autostartEnabled?: boolean;
}) {
  const settings = await updateSettings(payload);
  notificationEnabled.value = settings.notificationEnabled;
  listDefaultSortRaw.value = settings.listDefaultSort;
  floatAlwaysOnTop.value = settings.floatAlwaysOnTop;
  floatVisibleCount.value = settings.floatVisibleCount;
  floatAutoShow.value = settings.floatAutoShow;
  floatDefaultMode.value = settings.floatDefaultMode;
  floatHoverPreview.value = settings.floatHoverPreview;
  autostartEnabled.value = settings.autostartEnabled;
  if (payload.listDefaultSort) {
    const sort = parseListDefaultSort(settings.listDefaultSort);
    setSort(sort.sortBy, sort.sortOrder);
  }
}
</script>

<template>
  <div class="app">
    <AppHeader
      v-model:keyword="keyword"
      @search="search"
      @create="openCreateModal"
      @settings="settingsOpen = true"
      @open-companion="handleOpenCompanion"
    />

    <div class="body">
      <ViewSidebar
        :view="view"
        :active-tag="activeTag"
        :all-tags="allTags"
        :overdue-count="overdueCount"
        @select-view="handleViewSelect"
      />

      <TaskListPane
        :todos="todos"
        :selected-id="selectedId"
        :loading="loading"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :view="view"
        :active-tag="activeTag"
        :sort-by="sortBy"
        :sort-order="sortOrder"
        @select="handleSelect"
        @toggle-complete="handleToggleComplete"
        @prev-page="goToPage(page - 1)"
        @next-page="goToPage(page + 1)"
        @change-sort="handleChangeSort"
        @empty-action="handleEmptyAction"
      />

      <TaskInspector
        :detail="detail"
        v-model:edit-title="editTitle"
        v-model:edit-description="editDescription"
        v-model:edit-priority="editPriority"
        v-model:edit-due-date="editDueDate"
        v-model:edit-tags="editTags"
        :saving="saving"
        :error="detailError"
        :reminders="reminders"
        :reminders-loading="remindersLoading"
        :reminders-error="remindersError ?? reminderError"
        :suggested-tags="allTags"
        @field-change="scheduleAutosave"
        @close="closeDetail"
        @transition="transitionTo"
        @remove="handleRemove"
        @restore="handleRestore"
        @add-reminder="handleAddReminder"
        @update-reminder="handleUpdateReminder"
        @remove-reminder="handleRemoveReminder"
        @snooze-reminder="handleSnoozeReminder"
      />
    </div>

    <p v-if="listError" class="global-error">{{ listError }}</p>

    <CreateTodoModal
      ref="createModalRef"
      :open="createOpen"
      @close="createOpen = false"
      @submit="handleCreateSubmit"
    />

    <SettingsModal
      :open="settingsOpen"
      :notification-enabled="notificationEnabled"
      :list-default-sort="listDefaultSortRaw"
      :float-always-on-top="floatAlwaysOnTop"
      :float-visible-count="floatVisibleCount"
      :float-auto-show="floatAutoShow"
      :float-default-mode="floatDefaultMode"
      :float-hover-preview="floatHoverPreview"
      :autostart-enabled="autostartEnabled"
      @close="settingsOpen = false"
      @update="handleSettingsUpdate"
    />
  </div>
</template>

<style>
* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  height: 100%;
}

body {
  font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", system-ui, sans-serif;
  color: #0f172a;
  background: #eef2f7;
}
</style>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background:
    radial-gradient(1200px 400px at 10% -10%, rgba(15, 23, 42, 0.06), transparent 60%),
    #eef2f7;
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
  margin: 0;
  background: #fff;
  border-top: 1px solid #e2e8f0;
}

.global-error {
  margin: 0;
  padding: 8px 16px;
  background: #fef2f2;
  color: #dc2626;
}
</style>
