<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { listen } from "@tauri-apps/api/event";

import * as todoApi from "@/api/todos";
import { getSettings } from "@/api/settings";
import type { CompanionSession, RepeatType, SettingsDto, TodoDto } from "@/api/types";
import { TITLE_MAX_LENGTH } from "@/api/types";
import {
  companionClickChrome,
  companionMinimize,
  companionPointerCluster,
  companionRefreshSession,
  getCompanionSession,
  hideFloatingWindow,
} from "@/api/window";
import FloatDetailPanel from "@/components/float/FloatDetailPanel.vue";
import FloatTaskList from "@/components/float/FloatTaskList.vue";
import AppShellOverlays from "@/components/common/AppShellOverlays.vue";
import { useCompanionDrag } from "@/composables/useCompanionDrag";
import { useReminders, useTodoDetail } from "@/composables/useTodoDetail";
import { TAURI_EVENTS } from "@/constants/events";
import { applyAppearance } from "@/utils/appearance";
import { formatErrorMessage } from "@/utils/error";

const { t } = useI18n();

const todos = ref<TodoDto[]>([]);
const session = ref<CompanionSession | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const selectedId = ref<string | null>(null);
const visibleCount = ref(5);
const quickTitle = ref("");
const creating = ref(false);
const reminderError = ref<string | null>(null);
const quickInput = ref<HTMLInputElement | null>(null);
let fetchToken = 0;

const HEADER_DRAG_THRESHOLD = 10;
let headerPointerId: number | null = null;
let headerStartX = 0;
let headerStartY = 0;
let headerDragging = false;
let skipPreviewPin = false;
let skipPreviewTimer: ReturnType<typeof setTimeout> | null = null;

const { startDrag, onPointerUp } = useCompanionDrag("body");

const isDocked = computed(() => session.value?.placement === "docked");
const activeCount = computed(() => session.value?.activeCount ?? 0);
const showMinus = computed(() => {
  const current = session.value;
  if (!current) return false;
  return current.panelMode !== "closed";
});
const minusTitle = computed(() => {
  const current = session.value;
  if (!current) return t("companion.dockToEdge");
  if (current.placement === "docked") return t("companion.collapseToStrip");
  if (current.homeShape === "ball") return t("companion.backToBall");
  return t("companion.dockToEdge");
});
const isPreview = computed(() => session.value?.panelMode === "preview");

const {
  detail,
  editTitle,
  editDescription,
  editPriority,
  editDueDate,
  saving,
  error: detailError,
  isDirty,
  flushAutosave,
  save,
  reload,
  transitionTo,
} = useTodoDetail(selectedId, fetchTodos);

const {
  reminders,
  loading: remindersLoading,
  error: remindersError,
  fetchReminders,
  addReminder,
  removeReminder,
} = useReminders(selectedId);

let unlistenSession: (() => void) | null = null;
let unlistenSettings: (() => void) | null = null;
let unlistenTodosChanged: (() => void) | null = null;
let todosChangedTimer: ReturnType<typeof setTimeout> | null = null;

function applySession(next: CompanionSession) {
  session.value = next;
}

const panelVisible = computed(() => {
  const current = session.value;
  if (!current || current.visibility !== "shown") return false;
  if (current.placement === "free") return current.panelMode === "pinned";
  return current.panelMode !== "closed";
});

const shouldFocusQuickAdd = computed(() => {
  const current = session.value;
  if (!current || current.visibility !== "shown") return false;
  if (selectedId.value) return false;
  return current.panelMode === "pinned";
});

async function fetchTodos(): Promise<void> {
  const token = ++fetchToken;
  loading.value = todos.value.length === 0;
  error.value = null;
  try {
    const listResult = await todoApi.listTodos({
      status: ["Todo", "Doing"],
      sortBy: "priority",
      sortOrder: "desc",
      page: 1,
      pageSize: visibleCount.value,
    });
    if (token !== fetchToken) return;
    todos.value = listResult.items;
  } catch (err) {
    if (token !== fetchToken) return;
    error.value = formatErrorMessage(err);
  } finally {
    if (token === fetchToken) {
      loading.value = false;
    }
  }
}

async function closeDetail(): Promise<boolean> {
  if (isDirty()) {
    if (!(await flushAutosave())) return false;
  }
  selectedId.value = null;
  return true;
}

async function selectTodo(id: string) {
  if (selectedId.value === id) {
    await closeDetail();
    return;
  }
  if (selectedId.value && isDirty()) {
    if (!(await flushAutosave())) return;
  }
  selectedId.value = id;
}

async function openMainWithTodo(id: string) {
  await todoApi.showMainWindow(id);
}

async function completeTodo(id: string) {
  try {
    await todoApi.transitionTodo(id, "Done");
    if (selectedId.value === id) selectedId.value = null;
    try {
      applySession(await companionRefreshSession());
    } catch {
      // ignore
    }
    await fetchTodos();
  } catch (err) {
    error.value = formatErrorMessage(err);
  }
}

async function submitQuickAdd() {
  const title = quickTitle.value.trim();
  if (!title || creating.value) return;
  if (title.length > TITLE_MAX_LENGTH) {
    error.value = t("validation.titleTooLong", { n: TITLE_MAX_LENGTH });
    return;
  }
  creating.value = true;
  error.value = null;
  try {
    await todoApi.createTodo({ title, priority: "Medium" });
    quickTitle.value = "";
    try {
      applySession(await companionRefreshSession());
    } catch {
      // 计数刷新失败不阻断列表
    }
    await fetchTodos();
    await nextTick();
    quickInput.value?.focus();
  } catch (err) {
    error.value = formatErrorMessage(err);
  } finally {
    creating.value = false;
  }
}

async function minimize() {
  if (!(await closeDetail())) return;
  try {
    applySession(await companionMinimize());
  } catch (err) {
    console.error("minimize failed", err);
    error.value = formatErrorMessage(err);
  }
}

async function hideCompanion() {
  if (!(await closeDetail())) return;
  try {
    await hideFloatingWindow();
  } catch (err) {
    console.error("hide companion failed", err);
  }
}

async function handleSave() {
  if (await save()) {
    selectedId.value = null;
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

async function handleRemoveReminder(id: string) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await removeReminder(id, selectedId.value);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

function onHeaderPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement;
  if (target.closest("button, input, textarea, select, a")) return;
  headerPointerId = event.pointerId;
  headerStartX = event.clientX;
  headerStartY = event.clientY;
  headerDragging = false;
  try {
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  } catch {
    // ignore
  }
}

function onHeaderPointerMove(event: PointerEvent) {
  if (headerPointerId !== event.pointerId || headerDragging) return;
  const dx = Math.abs(event.clientX - headerStartX);
  const dy = Math.abs(event.clientY - headerStartY);
  if (dx > HEADER_DRAG_THRESHOLD || dy > HEADER_DRAG_THRESHOLD) {
    headerDragging = true;
    skipPreviewPin = true;
    if (skipPreviewTimer) clearTimeout(skipPreviewTimer);
    skipPreviewTimer = setTimeout(() => {
      skipPreviewPin = false;
      skipPreviewTimer = null;
    }, 400);
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {
      // ignore
    }
    void startDrag();
  }
}

function onHeaderPointerUp(event: PointerEvent) {
  if (headerPointerId !== event.pointerId) return;
  headerPointerId = null;
  headerDragging = false;
  try {
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  } catch {
    // ignore
  }
}

async function onClusterEnter() {
  if (!session.value?.hoverPreview) return;
  try {
    applySession(await companionPointerCluster("body", { inside: true }));
  } catch (err) {
    console.error("body cluster enter failed", err);
  }
}

async function onClusterLeave() {
  if (!session.value?.hoverPreview) return;
  try {
    applySession(await companionPointerCluster("body", { inside: false }));
  } catch (err) {
    console.error("body cluster leave failed", err);
  }
}

async function onFocusIn() {
  if (!session.value?.hoverPreview) return;
  try {
    applySession(await companionPointerCluster("body", { focused: true }));
  } catch {
    // ignore
  }
}

async function onFocusOut() {
  if (!session.value?.hoverPreview) return;
  try {
    applySession(await companionPointerCluster("body", { focused: false }));
  } catch {
    // ignore
  }
}

async function onPreviewClick(event: MouseEvent) {
  if (skipPreviewPin) {
    skipPreviewPin = false;
    return;
  }
  if (session.value?.panelMode !== "preview") return;
  const target = event.target as HTMLElement;
  if (target.closest(".icon-btn")) return;
  try {
    applySession(await companionClickChrome());
  } catch (err) {
    console.error("pin preview failed", err);
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  if (selectedId.value) {
    void closeDetail();
    return;
  }
  const current = session.value;
  if (!current) return;
  if (current.placement === "docked" || current.homeShape === "ball") {
    void minimize();
  }
}

watch(panelVisible, async (visible) => {
  if (visible) {
    await fetchTodos();
    return;
  }
  // Chrome Esc/收条等不经 Body minimize：收起时 flush，失败则保留选中
  if (selectedId.value && isDirty()) {
    if (!(await flushAutosave())) return;
  }
  selectedId.value = null;
});

watch(shouldFocusQuickAdd, async (focus) => {
  if (!focus) return;
  await nextTick();
  quickInput.value?.focus();
});

onMounted(async () => {
  document.addEventListener("pointerup", onPointerUp, true);
  document.addEventListener("pointercancel", onPointerUp, true);
  window.addEventListener("keydown", onKeydown);
  unlistenSession = await listen<CompanionSession>(TAURI_EVENTS.FLOAT_SESSION_CHANGED, (event) => {
    applySession(event.payload);
  });
  unlistenSettings = await listen<SettingsDto>(TAURI_EVENTS.SETTINGS_UPDATED, (event) => {
    visibleCount.value = event.payload.floatVisibleCount;
    applyAppearance(event.payload);
    if (panelVisible.value) {
      void fetchTodos();
    }
  });
  unlistenTodosChanged = await listen(TAURI_EVENTS.TODOS_CHANGED, () => {
    if (todosChangedTimer) clearTimeout(todosChangedTimer);
    todosChangedTimer = setTimeout(() => {
      todosChangedTimer = null;
      void (async () => {
        await fetchTodos();
        try {
          applySession(await companionRefreshSession());
        } catch {
          // ignore
        }
        if (!selectedId.value || saving.value) return;
        if (isDirty()) {
          await flushAutosave();
        } else {
          await reload();
          await fetchReminders(selectedId.value);
        }
      })();
    }, 200);
  });
  try {
    const settings = await getSettings();
    visibleCount.value = settings.floatVisibleCount;
    applyAppearance(settings);
  } catch {
    // ignore
  }
  try {
    applySession(await getCompanionSession());
  } catch (err) {
    console.error("getCompanionSession failed", err);
  }
});

onUnmounted(() => {
  document.removeEventListener("pointerup", onPointerUp, true);
  document.removeEventListener("pointercancel", onPointerUp, true);
  window.removeEventListener("keydown", onKeydown);
  if (todosChangedTimer) clearTimeout(todosChangedTimer);
  unlistenSession?.();
  unlistenSettings?.();
  unlistenTodosChanged?.();
});
</script>

<template>
  <div
    class="float-app"
    :class="{
      'edge-left': session?.dockEdge === 'left' && isDocked,
      'edge-right': session?.dockEdge === 'right' && isDocked,
    }"
    @mouseenter="onClusterEnter"
    @mouseleave="onClusterLeave"
    @focusin="onFocusIn"
    @focusout="onFocusOut"
    @click="onPreviewClick"
  >
    <p v-if="isPreview" class="preview-hint">{{ $t("companion.previewHint") }}</p>
    <header
      class="float-header"
      @pointerdown="onHeaderPointerDown"
      @pointermove="onHeaderPointerMove"
      @pointerup="onHeaderPointerUp"
      @pointercancel="onHeaderPointerUp"
    >
      <span class="title">{{ $t("common.brand") }}</span>
      <span class="count">{{ $t("companion.activeCount", { n: activeCount }) }}</span>
      <button
        v-if="showMinus"
        type="button"
        class="icon-btn"
        :title="minusTitle"
        :aria-label="minusTitle"
        @click.stop="minimize"
      >
        −
      </button>
      <button
        type="button"
        class="icon-btn"
        :title="$t('companion.hideToTray')"
        :aria-label="$t('companion.hideToTray')"
        @click.stop="hideCompanion"
      >
        ×
      </button>
    </header>

    <form class="quick-add" @submit.prevent="submitQuickAdd">
      <input
        ref="quickInput"
        v-model="quickTitle"
        type="text"
        maxlength="200"
        :placeholder="$t('companion.quickAddPlaceholder')"
        :disabled="creating"
      />
    </form>

    <p v-if="error" class="error">{{ error }}</p>

    <p v-if="!selectedId" class="edit-hint">{{ $t("companion.editHint") }}</p>

    <FloatTaskList
      :todos="todos"
      :selected-id="selectedId"
      :loading="loading"
      @select="selectTodo"
      @open-main="openMainWithTodo"
      @complete="completeTodo"
    />

    <FloatDetailPanel
      v-if="selectedId && detail"
      :detail="detail"
      v-model:edit-title="editTitle"
      v-model:edit-description="editDescription"
      v-model:edit-priority="editPriority"
      v-model:edit-due-date="editDueDate"
      :saving="saving"
      :error="detailError"
      :reminders="reminders"
      :reminders-loading="remindersLoading"
      :reminders-error="remindersError ?? reminderError"
      @close="closeDetail"
      @save="handleSave"
      @transition="transitionTo"
      @add-reminder="handleAddReminder"
      @remove-reminder="handleRemoveReminder"
    />

    <AppShellOverlays />
  </div>
</template>

<style>
html,
body,
#app {
  margin: 0;
  height: 100%;
  background: transparent;
  overflow: hidden;
}
</style>

<style scoped>
.float-app {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: color-mix(in srgb, var(--color-surface) 96%, transparent);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
  color: var(--color-text);
  font-family: var(--font-ui);
}

.preview-hint {
  margin: 0;
  padding: 4px 12px;
  font-size: 11px;
  color: var(--color-accent);
  background: var(--color-accent-soft);
  text-align: center;
}

.edge-right {
  border-radius: var(--radius-sm) 0 0 var(--radius-sm);
}

.edge-left {
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
}

.float-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  background: var(--color-surface-muted);
  border-bottom: 1px solid var(--color-border);
  cursor: grab;
  user-select: none;
  touch-action: none;
}

.title {
  font-weight: 600;
  font-size: 14px;
}

.count {
  flex: 1;
  font-size: 12px;
  color: var(--color-muted);
}

.icon-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  transition: background var(--transition-fast);
}

.icon-btn:hover {
  background: var(--color-bg-accent);
}

.quick-add {
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-border);
}

.quick-add input {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--color-text);
  background: var(--color-surface);
}

.edit-hint {
  margin: 0;
  padding: 4px 12px;
  font-size: 11px;
  color: var(--color-muted);
  text-align: center;
}

.error {
  margin: 0;
  padding: 8px 12px;
  background: var(--color-danger-bg);
  color: var(--color-danger);
  font-size: 13px;
}
</style>
