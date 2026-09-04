<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
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
import { listAllTags, getTodo, getWorkbenchMeta, transitionTodo } from "@/api/todos";
import { listEvents } from "@/api/events";
import { getSettings, updateSettings } from "@/api/settings";
import type { EventDto } from "@/api/types";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import CreateTodoModal from "@/components/todo/CreateTodoModal.vue";
import AppShellOverlays from "@/components/common/AppShellOverlays.vue";
import UpdateProgressModal from "@/components/common/UpdateProgressModal.vue";
import TaskEditDrawer from "@/components/common/TaskEditDrawer.vue";
import ViewSidebar from "@/components/workbench/ViewSidebar.vue";
import TaskListPane from "@/components/workbench/TaskListPane.vue";
import TaskInspector from "@/components/workbench/TaskInspector.vue";
import ActivityStrip from "@/components/workbench/ActivityStrip.vue";
import { TAURI_EVENTS } from "@/constants/events";
import { coerceDefaultView, isLibraryView } from "@/constants/workbenchViews";
import { modalEscDepth } from "@/composables/useModalEscStack";
import { useAppUpdater } from "@/composables/useAppUpdater";
import { useReminders, useTodoDetail } from "@/composables/useTodoDetail";
import { useTodos } from "@/composables/useTodos";
import { applyAppearance } from "@/utils/appearance";
import { fromLocalDatetimeInput, localTodayDueInput } from "@/utils/date";
import { formatErrorMessage } from "@/utils/error";
import { parseListDefaultSort } from "@/utils/listSort";

const { t } = useI18n();

const {
  todos,
  focusBoard,
  total,
  page,
  pageSize,
  loading,
  error: listError,
  keyword,
  searchActive,
  mode,
  view,
  activeTag,
  sortBy,
  sortOrder,
  selectedId,
  fetchTodos,
  createTodo,
  selectTodo,
  setMode,
  setView,
  setSort,
  search,
  clearSearch,
  goToPage,
} = useTodos();

/** 驱动右侧编辑面板（双击/peek 编辑按钮打开） */
const editingId = ref<string | null>(null);

const {
  reminders,
  loading: remindersLoading,
  error: remindersError,
  fetchReminders,
  addReminder,
  updateReminder,
  removeReminder,
  snoozeReminder,
} = useReminders(editingId);

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
} = useTodoDetail(editingId, async () => {
  await Promise.all([fetchTodos(), loadWorkbenchMeta()]);
});

const { updating, checking, progress, phase, statusMessage, indeterminate, runUpdateFlow } =
  useAppUpdater({
    t,
    flushBeforeInstall: flushAutosave,
  });

const settingsOpen = ref(false);
const settingsError = ref<string | null>(null);
const settingsSaving = ref(false);
const createOpen = ref(false);
/** 打开新建弹窗时可选预填截止日期（今日空态） */
const createInitialDue = ref<string | null>(null);
const notificationEnabled = ref(true);
const listDefaultSortRaw = ref('{"sort_by":"priority","sort_order":"desc"}');
const listDefaultView = ref<WorkbenchView>("today");
const floatAlwaysOnTop = ref(true);
const floatVisibleCount = ref(5);
const floatAutoShow = ref(true);
const floatDefaultMode = ref<FloatDefaultMode>("ball");
const floatHoverPreview = ref(false);
const autostartEnabled = ref(false);
const uiTheme = ref<UiTheme>("system");
const uiLocale = ref<UiLocale>("zh-CN");
const allTags = ref<string[]>([]);
const createModalRef = ref<InstanceType<typeof CreateTodoModal> | null>(null);
const reminderError = ref<string | null>(null);
const recentEvents = ref<EventDto[]>([]);
const eventUnlisteners: (() => void)[] = [];

const showQuickAdd = computed(() => {
  if (searchActive.value) return false;
  if (mode.value === "focus") return true;
  return view.value === "all" || view.value === "tag";
});

/** 同步设置到本地状态；不切换侧栏 view（避免主题/语言保存时跳视图） */
async function loadAndApplySettings() {
  try {
    const settings = await getSettings();
    notificationEnabled.value = settings.notificationEnabled;
    listDefaultSortRaw.value = settings.listDefaultSort;
    listDefaultView.value = coerceDefaultView(settings.listDefaultView);
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

function applyColdStartView() {
  const next = coerceDefaultView(listDefaultView.value);
  listDefaultView.value = next;
  if (isLibraryView(next)) {
    mode.value = "library";
    view.value = next;
  } else {
    mode.value = "focus";
    view.value = "today";
  }
  activeTag.value = null;
}

onMounted(async () => {
  await loadAndApplySettings();
  applyColdStartView();

  await Promise.all([fetchTodos(), loadWorkbenchMeta()]);

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

  // 跨窗写操作后刷新；无 dirty 不回写，避免覆盖助理修改与 update 风暴
  let todosChangedTimer: ReturnType<typeof setTimeout> | null = null;
  eventUnlisteners.push(
    await listen(TAURI_EVENTS.TODOS_CHANGED, () => {
      if (todosChangedTimer) clearTimeout(todosChangedTimer);
      todosChangedTimer = setTimeout(() => {
        todosChangedTimer = null;
        void (async () => {
          // 列表与侧栏元数据并行；meta 合并为一次 IPC
          await Promise.all([fetchTodos(), loadWorkbenchMeta()]);
          if (!editingId.value || saving.value) return;
          if (isDirty()) {
            await flushAutosave();
          } else {
            await reload();
            await fetchReminders(editingId.value!);
          }
        })();
      }, 200);
    }),
  );

  window.addEventListener("keydown", onGlobalKeydown);

  // 启动后延迟检测更新（仅主窗）
  window.setTimeout(() => {
    void runUpdateFlow(false);
  }, 3000);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onGlobalKeydown);
  for (const unlisten of eventUnlisteners) {
    unlisten();
  }
});

async function loadWorkbenchMeta() {
  try {
    const meta = await getWorkbenchMeta(8);
    allTags.value = meta.tags;
    recentEvents.value = meta.events;
  } catch {
    // 回退拆分调用，避免单点失败整侧栏空白
    await Promise.all([loadTags(), loadRecentEvents()]);
  }
}

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

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

function onGlobalKeydown(event: KeyboardEvent) {
  if (settingsOpen.value || createOpen.value) {
    return;
  }

  if (event.key === "Escape") {
    if (modalEscDepth() > 0) return;
    if (editingId.value) {
      event.preventDefault();
      void closeDetail();
    } else if (selectedId.value) {
      event.preventDefault();
      selectedId.value = null;
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
    const quickAdd = document.getElementById("workbench-quick-add");
    if (quickAdd) {
      quickAdd.focus();
    } else {
      openCreateModal();
    }
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
      mode.value = "library";
      view.value = "trash";
      activeTag.value = null;
      searchActive.value = false;
      page.value = 1;
      await fetchTodos();
    } else if (!todos.value.some((item) => item.id === id)) {
      await fetchTodos();
      if (!todos.value.some((item) => item.id === id)) {
        mode.value = "library";
        view.value = "all";
        activeTag.value = null;
        searchActive.value = false;
        page.value = 1;
        await fetchTodos();
      }
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
  editingId.value = null;
  return true;
}

async function handleSelect(id: string) {
  if (selectedId.value === id) {
    selectedId.value = null;
    return;
  }
  await navigateToTodo(id);
}

function handlePeekEdit(id: string) {
  editingId.value = id;
}

async function handleToggleComplete(todo: TodoDto) {
  try {
    const next = todo.status === "Done" ? "Todo" : "Done";
    await transitionTodo(todo.id, next);
    if (editingId.value === todo.id) {
      await reload();
    }
    await fetchTodos();
  } catch (err) {
    listError.value = formatErrorMessage(err);
  }
}

async function handleRemove() {
  const ok = await remove();
  if (ok) {
    selectedId.value = null;
    await loadTags();
  }
}

async function handleRestore() {
  const restoredId = editingId.value;
  const ok = await restore();
  if (ok && restoredId) {
    mode.value = "library";
    view.value = "all";
    activeTag.value = null;
    searchActive.value = false;
    page.value = 1;
    await fetchTodos();
    selectedId.value = restoredId;
    editingId.value = restoredId;
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
    if (!todos.value.some((item) => item.id === created.id)) {
      mode.value = "library";
      view.value = "all";
      activeTag.value = null;
      searchActive.value = false;
      page.value = 1;
      await fetchTodos();
      await selectTodo(created.id);
      editingId.value = created.id;
    }
  } catch (err) {
    createModalRef.value?.setError(formatErrorMessage(err));
  }
}

async function handleQuickAdd(title: string) {
  try {
    const dto: CreateTodoDto = { title };
    if (mode.value === "focus") {
      const due = fromLocalDatetimeInput(localTodayDueInput());
      if (due) dto.dueDate = due;
    }
    await createTodo(dto);
    await loadTags();
  } catch (err) {
    listError.value = formatErrorMessage(err);
  }
}

async function handleAddReminder(datetime: string, repeatType?: RepeatType) {
  if (!editingId.value) return;
  reminderError.value = null;
  try {
    await addReminder(editingId.value, datetime, repeatType);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleUpdateReminder(id: string, datetime: string) {
  if (!editingId.value) return;
  reminderError.value = null;
  try {
    await updateReminder(id, editingId.value, datetime);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleRemoveReminder(id: string) {
  if (!editingId.value) return;
  reminderError.value = null;
  try {
    await removeReminder(id, editingId.value);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleSnoozeReminder(id: string, minutes: number) {
  if (!editingId.value) return;
  reminderError.value = null;
  try {
    await snoozeReminder(id, editingId.value, minutes);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleModeChange(next: typeof mode.value) {
  if (!(await closeDetail())) return;
  selectedId.value = null;
  setMode(next);
}

async function handleViewSelect(next: WorkbenchView, tag?: string | null) {
  if (!(await closeDetail())) return;
  selectedId.value = null;
  setView(next, tag);
}

function handleChangeSort(next: SortBy) {
  setSort(next);
}

function handleToggleSortOrder() {
  const flipped: SortOrder = sortOrder.value === "asc" ? "desc" : "asc";
  setSort(sortBy.value, flipped);
}

async function handleSettingsUpdate(payload: Parameters<typeof updateSettings>[0]) {
  settingsSaving.value = true;
  settingsError.value = null;
  try {
    const settings = await updateSettings(payload);
    notificationEnabled.value = settings.notificationEnabled;
    listDefaultSortRaw.value = settings.listDefaultSort;
    listDefaultView.value = coerceDefaultView(settings.listDefaultView);
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
    // 保存默认视图不强制切换当前侧栏
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
      :mode="mode"
      @update:mode="handleModeChange"
      @search="search"
      @clear-search="clearSearch"
      @settings="
        settingsError = null;
        settingsOpen = true;
      "
    />

    <div class="body">
      <ViewSidebar
        v-if="mode === 'library' && !searchActive"
        :view="view"
        :active-tag="activeTag"
        :all-tags="allTags"
        @select-view="handleViewSelect"
      />

      <div class="main-col">
        <p v-if="listError" class="list-error">{{ listError }}</p>
        <TaskListPane
          :todos="todos"
          :focus-board="focusBoard"
          :selected-id="selectedId"
          :loading="loading"
          :total="total"
          :page="page"
          :page-size="pageSize"
          :view="view"
          :active-tag="activeTag"
          :sort-by="sortBy"
          :sort-order="sortOrder"
          :grouped="mode === 'focus' && !searchActive"
          :searching="searchActive"
          :show-inline-add="showQuickAdd"
          :inline-placeholder="
            mode === 'focus' ? $t('list.inlineAddFocus') : $t('list.inlineAddLibrary')
          "
          @select="handleSelect"
          @toggle-complete="handleToggleComplete"
          @prev-page="goToPage(page - 1)"
          @next-page="goToPage(page + 1)"
          @change-sort="handleChangeSort"
          @toggle-sort-order="handleToggleSortOrder"
          @quick-add="handleQuickAdd"
          @peek-edit="handlePeekEdit"
        />
      </div>

      <TaskEditDrawer :open="!!editingId" @close="closeDetail">
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
      </TaskEditDrawer>
    </div>

    <ActivityStrip v-if="mode === 'library'" :events="recentEvents" />

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
      :list-default-view="listDefaultView"
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
      :checking-update="checking"
      @close="settingsOpen = false"
      @update="handleSettingsUpdate"
      @check-update="runUpdateFlow(true)"
    />

    <UpdateProgressModal
      :open="updating"
      :message="statusMessage"
      :progress="progress"
      :phase="phase"
      :indeterminate="indeterminate"
    />

    <AppShellOverlays confirm />
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
  background: var(--color-bg);
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
  margin: 0;
  background: var(--color-surface);
}

.main-col {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.list-error {
  margin: 0;
  padding: 8px 16px;
  background: var(--color-danger-bg);
  color: var(--color-danger);
}
</style>
