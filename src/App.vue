<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen } from "@tauri-apps/api/event";

import type {
  CreateTodoDto,
  FloatDefaultMode,
  RepeatType,
  SortBy,
  SortOrder,
  TodoDto,
  UiLocale,
  UiTheme,
  WorkbenchView,
} from "@/api/types";
import { listAllTags, listTodos, getTodo, transitionTodo } from "@/api/todos";
import { listEvents } from "@/api/events";
import { getSettings, updateSettings } from "@/api/settings";
import { showFloatingWindow } from "@/api/window";
import type { EventDto } from "@/api/types";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import CreateTodoModal from "@/components/todo/CreateTodoModal.vue";
import ViewSidebar from "@/components/workbench/ViewSidebar.vue";
import TaskListPane from "@/components/workbench/TaskListPane.vue";
import TaskInspector from "@/components/workbench/TaskInspector.vue";
import { TAURI_EVENTS } from "@/constants/events";
import { useReminders, useTodoDetail } from "@/composables/useTodoDetail";
import { useTodos } from "@/composables/useTodos";
import { applyAppearance } from "@/utils/appearance";
import { dueDateRangeForFilter, localTodayDueInput } from "@/utils/date";
import { formatErrorMessage } from "@/utils/error";
import {
  formatRelativeTime,
  translateEntityType,
  translateEventType,
} from "@/utils/i18nFormat";
import { parseListDefaultSort } from "@/utils/listSort";

const { t } = useI18n();

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
  fetchReminders,
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
  saveStatus,
  error: detailError,
  isDirty,
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
  await loadRecentEvents();
});

const settingsOpen = ref(false);
const settingsError = ref<string | null>(null);
const settingsSaving = ref(false);
const createOpen = ref(false);
/** 打开新建弹窗时可选预填截止日期（今日空态） */
const createInitialDue = ref<string | null>(null);
const notificationEnabled = ref(true);
const listDefaultSortRaw = ref('{"sort_by":"priority","sort_order":"desc"}');
const floatAlwaysOnTop = ref(true);
const floatVisibleCount = ref(5);
const floatAutoShow = ref(true);
const floatDefaultMode = ref<FloatDefaultMode>("ball");
const floatHoverPreview = ref(false);
const autostartEnabled = ref(false);
const uiTheme = ref<UiTheme>("system");
const uiLocale = ref<UiLocale>("zh-CN");
const allTags = ref<string[]>([]);
const overdueCount = ref(0);
const createModalRef = ref<InstanceType<typeof CreateTodoModal> | null>(null);
const reminderError = ref<string | null>(null);
const recentEvents = ref<EventDto[]>([]);
const activityCollapsed = ref(false);
const eventUnlisteners: (() => void)[] = [];

async function loadAndApplySettings() {
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
    uiTheme.value = settings.uiTheme;
    uiLocale.value = settings.uiLocale;
    applyAppearance(settings);
    const sort = parseListDefaultSort(settings.listDefaultSort);
    sortBy.value = sort.sortBy;
    sortOrder.value = sort.sortOrder;
  } catch {
    // 首次启动时 settings 表可能尚未写入种子数据，忽略即可
  }
}

onMounted(async () => {
  await loadAndApplySettings();

  await fetchTodos();
  await loadTags();
  await refreshOverdueCount();
  await loadRecentEvents();

  eventUnlisteners.push(
    await listen<string>(TAURI_EVENTS.NAVIGATE_TO_TODO, (event) => {
      void navigateToTodo(event.payload);
    }),
  );

  eventUnlisteners.push(
    await listen(TAURI_EVENTS.CREATE_TODO, () => {
      openCreateModal();
    }),
  );

  eventUnlisteners.push(
    await listen(TAURI_EVENTS.OPEN_SETTINGS, () => {
      settingsError.value = null;
      settingsOpen.value = true;
    }),
  );

  eventUnlisteners.push(
    await listen(TAURI_EVENTS.SETTINGS_UPDATED, () => {
      void loadAndApplySettings();
    }),
  );

  // 跨窗写操作后刷新；无 dirty 不回写，避免覆盖伴侣修改与 update 风暴
  let todosChangedTimer: ReturnType<typeof setTimeout> | null = null;
  eventUnlisteners.push(
    await listen(TAURI_EVENTS.TODOS_CHANGED, () => {
      if (todosChangedTimer) clearTimeout(todosChangedTimer);
      todosChangedTimer = setTimeout(() => {
        todosChangedTimer = null;
        void (async () => {
          await fetchTodos();
          await refreshOverdueCount();
          await loadTags();
          await loadRecentEvents();
          if (!selectedId.value || saving.value) return;
          if (isDirty()) {
            await flushAutosave();
          } else {
            await reload();
            await fetchReminders(selectedId.value);
          }
        })();
      }, 200);
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

async function loadRecentEvents() {
  try {
    recentEvents.value = await listEvents({ limit: 8 });
  } catch {
    recentEvents.value = [];
  }
}

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
    openCreateModal();
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
  const ids = todos.value.map((item) => item.id);
  const current = selectedId.value ? ids.indexOf(selectedId.value) : -1;
  let next = current + delta;
  if (current < 0) next = delta > 0 ? 0 : ids.length - 1;
  if (next < 0) next = 0;
  if (next >= ids.length) next = ids.length - 1;
  await navigateToTodo(ids[next] ?? null);
}

async function navigateToTodo(id: string | null) {
  if (!(await flushAutosave())) return;
  if (!id) {
    await selectTodo(null);
    return;
  }

  try {
    const todo = await getTodo(id);
    if (todo.deletedAt) {
      if (view.value !== "trash") {
        view.value = "trash";
        activeTag.value = null;
        page.value = 1;
        await fetchTodos();
      } else if (!todos.value.some((item) => item.id === id)) {
        await fetchTodos();
      }
    } else if (!todos.value.some((item) => item.id === id)) {
      // 当前视图可能筛掉该任务；切到「全部」再定位（与恢复后导航一致）
      if (view.value !== "all" || activeTag.value) {
        view.value = "all";
        activeTag.value = null;
        page.value = 1;
      }
      await fetchTodos();
    }
  } catch {
    // get 失败仍尝试选中，由 Inspector 展示错误
  }

  await selectTodo(id);
}

function openCreateModal(opts?: { dueToday?: boolean }) {
  createInitialDue.value = opts?.dueToday ? localTodayDueInput() : null;
  createOpen.value = true;
}

async function closeDetail(): Promise<boolean> {
  if (!(await flushAutosave())) return false;
  selectedId.value = null;
  return true;
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
    const created = await createTodo(dto);
    createOpen.value = false;
    createInitialDue.value = null;
    createModalRef.value?.resetSubmitting();
    await loadTags();
    await refreshOverdueCount();
    // 今日视图创建但未带 due 时可能不在列表；稳妥切全部并选中
    if (view.value === "today" && !todos.value.some((item) => item.id === created.id)) {
      view.value = "all";
      activeTag.value = null;
      page.value = 1;
      await fetchTodos();
      await selectTodo(created.id);
    }
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
  if (!(await flushAutosave())) return;
  setView(next, tag);
  selectedId.value = null;
}

function handleChangeSort(next: SortBy) {
  setSort(next);
}

function handleToggleSortOrder() {
  const flipped: SortOrder = sortOrder.value === "asc" ? "desc" : "asc";
  setSort(sortBy.value, flipped);
}

function handleEmptyAction(action: "create" | "all") {
  if (action === "create") {
    openCreateModal({ dueToday: view.value === "today" });
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

async function handleSettingsUpdate(payload: Parameters<typeof updateSettings>[0]) {
  settingsSaving.value = true;
  settingsError.value = null;
  try {
    const settings = await updateSettings(payload);
    notificationEnabled.value = settings.notificationEnabled;
    listDefaultSortRaw.value = settings.listDefaultSort;
    floatAlwaysOnTop.value = settings.floatAlwaysOnTop;
    floatVisibleCount.value = settings.floatVisibleCount;
    floatAutoShow.value = settings.floatAutoShow;
    floatDefaultMode.value = settings.floatDefaultMode;
    floatHoverPreview.value = settings.floatHoverPreview;
    autostartEnabled.value = settings.autostartEnabled;
    uiTheme.value = settings.uiTheme;
    uiLocale.value = settings.uiLocale;
    applyAppearance(settings);
    if (payload.listDefaultSort) {
      const sort = parseListDefaultSort(settings.listDefaultSort);
      setSort(sort.sortBy, sort.sortOrder);
    }
    settingsOpen.value = false;
  } catch (err) {
    settingsError.value = formatErrorMessage(err);
  } finally {
    settingsSaving.value = false;
  }
}
</script>

<template>
  <div class="app">
    <AppHeader
      v-model:keyword="keyword"
      @search="search"
      @create="openCreateModal"
      @settings="
        settingsError = null;
        settingsOpen = true;
      "
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
        @toggle-sort-order="handleToggleSortOrder"
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
        :save-status="saveStatus"
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

    <aside v-if="recentEvents.length" class="activity" :aria-label="$t('activity.aria')">
      <header class="activity-header">
        <h2 class="activity-title">{{ $t("activity.title") }}</h2>
        <button
          type="button"
          class="activity-toggle"
          :aria-expanded="!activityCollapsed"
          @click="activityCollapsed = !activityCollapsed"
        >
          {{ activityCollapsed ? $t("activity.expand") : $t("activity.collapse") }}
        </button>
      </header>
      <ul v-show="!activityCollapsed" class="activity-list">
        <li v-for="ev in recentEvents" :key="ev.id" class="activity-item">
          <span class="activity-type">{{ translateEventType(ev.eventType, t) }}</span>
          <span class="activity-meta">
            {{ translateEntityType(ev.entityType, t) }} ·
            {{ formatRelativeTime(ev.createdAt, t) }}
          </span>
        </li>
      </ul>
    </aside>

    <CreateTodoModal
      ref="createModalRef"
      :open="createOpen"
      :initial-due-date="createInitialDue"
      @close="
        createOpen = false;
        createInitialDue = null;
      "
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
      :ui-theme="uiTheme"
      :ui-locale="uiLocale"
      :error="settingsError"
      :saving="settingsSaving"
      @close="settingsOpen = false"
      @update="handleSettingsUpdate"
    />
  </div>
</template>

<style>
#app {
  height: 100%;
}
</style>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background:
    radial-gradient(1200px 400px at 10% -10%, var(--color-accent-soft), transparent 60%),
    var(--color-bg);
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
  margin: 0;
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
}

.global-error {
  margin: 0;
  padding: 8px 16px;
  background: var(--color-danger-bg);
  color: var(--color-danger);
}

.activity {
  border-top: 1px solid var(--color-border);
  background: var(--color-surface-muted);
  padding: 8px 16px 12px;
  max-height: 120px;
  overflow: auto;
}

.activity-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 6px;
}

.activity-title {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-muted);
  letter-spacing: 0.02em;
}

.activity-toggle {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 4px;
  transition: color var(--transition-fast);
}

.activity-toggle:hover {
  color: var(--color-accent-hover);
}

.activity-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.activity-item {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.activity-type {
  font-weight: 600;
  color: var(--color-text);
  min-width: 140px;
}

.activity-meta {
  opacity: 0.85;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
