<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import * as todoApi from "@/api/todos";
import { getSettings } from "@/api/settings";
import {
  dockFloatWindow,
  getFloatWindowState,
  hideFloatingWindow,
  isPrimaryMouseDown,
  peekDockedFloat,
  setFloatDisplayMode,
  tryDockOnEdge,
  undockFloatWindow,
  unpeekDockedFloat,
} from "@/api/window";
import type {
  CreateTodoDto,
  FloatDefaultMode,
  FloatDisplayMode,
  RepeatType,
  SettingsDto,
  TodoDto,
} from "@/api/types";
import CreateTodoModal from "@/components/todo/CreateTodoModal.vue";
import FloatBall from "@/components/float/FloatBall.vue";
import FloatDetailPanel from "@/components/float/FloatDetailPanel.vue";
import FloatDockStrip from "@/components/float/FloatDockStrip.vue";
import FloatTaskList from "@/components/float/FloatTaskList.vue";
import { TAURI_EVENTS } from "@/constants/events";
import { useReminders, useTodoDetail } from "@/composables/useTodoDetail";
import { formatErrorMessage } from "@/utils/error";

const todos = ref<TodoDto[]>([]);
const displayMode = ref<FloatDisplayMode>("ball");
const defaultMode = ref<FloatDefaultMode>("ball");
const dockEdge = ref<"left" | "right" | "top" | "bottom">("right");
const activeCount = ref(0);
const overdueCount = ref(0);
const dueTodayCount = ref(0);
const loading = ref(false);
const error = ref<string | null>(null);
const selectedId = ref<string | null>(null);
const createOpen = ref(false);
const visibleCount = ref(5);
const createModalRef = ref<InstanceType<typeof CreateTodoModal> | null>(null);
const reminderError = ref<string | null>(null);

/** 真实拖拽进行中 */
const didDrag = ref(false);
/** 挂边悬停展开态 */
const dockPeeked = ref(false);
/** peek 进行中（先切 UI，避免蓝块闪烁） */
const peekInFlight = ref(false);

let unpeekTimer: ReturnType<typeof setTimeout> | null = null;
let dragSafetyTimer: ReturnType<typeof setTimeout> | null = null;
let dragSettleTimer: ReturnType<typeof setTimeout> | null = null;
let unlistenSettings: (() => void) | null = null;
let unlistenMoved: (() => void) | null = null;
let peekGeneration = 0;
let dragSessionId = 0;
let fetchSeq = 0;
let pointerInsideFloat = false;
let movedListenReady: Promise<void> | null = null;
/** 主按键是否按下（document capture 快路径；settle 以 OS GetAsyncKeyState 为准） */
let primaryButtonDown = false;

const DRAG_SETTLE_MS = 180;
const DRAG_SAFETY_MS = 3000;
const HEADER_DRAG_THRESHOLD = 5;

const isBall = computed(() => displayMode.value === "ball");
const isPanel = computed(() => displayMode.value === "panel");
const isDocked = computed(() => displayMode.value === "docked");
const isBallHome = computed(() => defaultMode.value === "ball");
const showPanelUi = computed(
  () => isPanel.value || (isDocked.value && (dockPeeked.value || peekInFlight.value)),
);
const showDockStrip = computed(() => isDocked.value && !dockPeeked.value && !peekInFlight.value);

const {
  detail,
  editTitle,
  editDescription,
  editPriority,
  editDueDate,
  saving,
  error: detailError,
  save,
  transitionTo,
} = useTodoDetail(selectedId, fetchFloatTodos);

const {
  reminders,
  loading: remindersLoading,
  error: remindersError,
  addReminder,
  removeReminder,
} = useReminders(selectedId);

function applyFloatState(
  state: Awaited<ReturnType<typeof getFloatWindowState>>,
  opts?: { forceClearPeek?: boolean },
) {
  displayMode.value = state.displayMode;
  defaultMode.value = state.defaultMode === "panel" ? "panel" : "ball";
  dockEdge.value = state.dockEdge;
  activeCount.value = state.activeCount;
  overdueCount.value = state.overdueCount;
  dueTodayCount.value = state.dueTodayCount;
  if (state.displayMode !== "docked" || opts?.forceClearPeek) {
    dockPeeked.value = false;
    peekInFlight.value = false;
  }
}

async function refreshFloatState(forceClearPeek = false): Promise<void> {
  try {
    const state = await getFloatWindowState();
    applyFloatState(state, { forceClearPeek });
  } catch (err) {
    console.error("refreshFloatState failed", err);
  }
}

async function fetchFloatTodos(): Promise<void> {
  const seq = ++fetchSeq;
  loading.value = true;
  error.value = null;
  try {
    const [listResult, state] = await Promise.all([
      todoApi.listTodos({
        status: ["Todo", "Doing"],
        sortBy: "priority",
        sortOrder: "desc",
        page: 1,
        pageSize: visibleCount.value,
      }),
      getFloatWindowState(),
    ]);
    if (seq !== fetchSeq) return;
    todos.value = listResult.items;
    applyFloatState(state);
  } catch (err) {
    if (seq !== fetchSeq) return;
    error.value = formatErrorMessage(err);
  } finally {
    if (seq === fetchSeq) {
      loading.value = false;
    }
  }
}

async function loadSettings(): Promise<void> {
  try {
    const settings = await getSettings();
    visibleCount.value = settings.floatVisibleCount;
    defaultMode.value = settings.floatDefaultMode;
  } catch (err) {
    console.error("loadSettings failed", err);
  }
}

function selectTodo(id: string) {
  selectedId.value = selectedId.value === id ? null : id;
}

function openCreateModal() {
  selectedId.value = null;
  createOpen.value = true;
}

function openMainWithTodo(id: string) {
  void todoApi.showMainWindow(id);
}

async function handleCreateSubmit(dto: CreateTodoDto) {
  try {
    await todoApi.createTodo(dto);
    onCreateClose();
    await fetchFloatTodos();
  } catch (err) {
    createModalRef.value?.setError(formatErrorMessage(err));
  }
}

async function handleSave() {
  const ok = await save();
  if (ok) {
    selectedId.value = null;
    await fetchFloatTodos();
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

function resetTransientUi() {
  selectedId.value = null;
  createOpen.value = false;
  peekGeneration += 1;
  dockPeeked.value = false;
  peekInFlight.value = false;
  if (unpeekTimer) {
    clearTimeout(unpeekTimer);
    unpeekTimer = null;
  }
}

async function expandToPanel() {
  resetTransientUi();
  try {
    const state = await setFloatDisplayMode("panel");
    applyFloatState(state, { forceClearPeek: true });
    await fetchFloatTodos();
  } catch (err) {
    console.error("expandToPanel failed", err);
    error.value = formatErrorMessage(err);
  }
}

async function collapseToBall() {
  resetTransientUi();
  try {
    const state = await setFloatDisplayMode("ball");
    applyFloatState(state, { forceClearPeek: true });
  } catch (err) {
    console.error("collapseToBall failed", err);
    error.value = formatErrorMessage(err);
  }
}

/** 主形态 ball：关面板回到圆球；主形态 panel：隐藏至托盘 */
async function closePanel() {
  if (isBallHome.value) {
    if (isDocked.value) {
      await handleUndock();
      return;
    }
    await collapseToBall();
    return;
  }
  await hideFloatingWindow();
}

async function handleDock() {
  resetTransientUi();
  try {
    const state = await dockFloatWindow();
    applyFloatState(state, { forceClearPeek: true });
  } catch (err) {
    console.error("handleDock failed", err);
    error.value = formatErrorMessage(err);
  }
}

async function handleUndock() {
  resetTransientUi();
  try {
    const state = await undockFloatWindow();
    applyFloatState(state, { forceClearPeek: true });
    await fetchFloatTodos();
  } catch (err) {
    console.error("handleUndock failed", err);
    error.value = formatErrorMessage(err);
  }
}

function clearDragTimers() {
  if (dragSafetyTimer) {
    clearTimeout(dragSafetyTimer);
    dragSafetyTimer = null;
  }
  if (dragSettleTimer) {
    clearTimeout(dragSettleTimer);
    dragSettleTimer = null;
  }
}

function armDragSafetyTimer() {
  if (dragSafetyTimer) clearTimeout(dragSafetyTimer);
  dragSafetyTimer = setTimeout(() => {
    void (async () => {
      if (!didDrag.value) return;
      // 左键仍按下：重臂，避免中途静止 3s 误挂边
      if (await isOsPrimaryDown()) {
        armDragSafetyTimer();
        return;
      }
      void finishDragIfNeeded();
    })();
  }, DRAG_SAFETY_MS);
}

function armDragSettleTimer() {
  if (dragSettleTimer) clearTimeout(dragSettleTimer);
  dragSettleTimer = setTimeout(() => {
    void (async () => {
      if (!didDrag.value) return;
      // 静止但左键仍按下：只重臂（避免中途停顿误挂边）
      if (await isOsPrimaryDown()) {
        armDragSettleTimer();
        return;
      }
      void finishDragIfNeeded();
    })();
  }, DRAG_SETTLE_MS);
}

async function isOsPrimaryDown(): Promise<boolean> {
  try {
    return await isPrimaryMouseDown();
  } catch {
    return primaryButtonDown;
  }
}

function onWindowMovedDuringDrag() {
  if (!didDrag.value) return;
  armDragSettleTimer();
  // 仍在移动则延长「自上次 Moved」安全超时
  armDragSafetyTimer();
}

function onPrimaryPointerDown(event: PointerEvent) {
  if (event.button === 0) {
    primaryButtonDown = true;
  }
}

function onPrimaryPointerUp(event: PointerEvent) {
  if (event.button === 0 || event.type === "pointercancel") {
    primaryButtonDown = false;
  }
  // 抬起时若处于拖拽且已静止，立即尝试吸附
  if (didDrag.value && !primaryButtonDown) {
    void finishDragIfNeeded();
  }
}

async function ensureMovedListener() {
  if (unlistenMoved) return;
  if (!movedListenReady) {
    movedListenReady = (async () => {
      unlistenMoved = await getCurrentWindow().onMoved(() => {
        onWindowMovedDuringDrag();
      });
    })().catch((err) => {
      console.error("onMoved listen failed", err);
      movedListenReady = null;
    });
  }
  await movedListenReady;
}

async function finishDragIfNeeded() {
  if (!didDrag.value) return;
  const session = dragSessionId;
  didDrag.value = false;
  clearDragTimers();
  try {
    const state = await tryDockOnEdge();
    if (session !== dragSessionId) return;
    applyFloatState(state, { forceClearPeek: true });
    if (state.displayMode === "panel") {
      await fetchFloatTodos();
    }
  } catch (err) {
    console.error("tryDockOnEdge failed", err);
    await refreshFloatState(true);
  }
}

async function startWindowDrag() {
  dragSessionId += 1;
  didDrag.value = true;
  clearDragTimers();
  armDragSafetyTimer();
  await ensureMovedListener();

  try {
    await getCurrentWindow().startDragging();
  } catch (err) {
    console.error("startDragging failed", err);
    didDrag.value = false;
    clearDragTimers();
  }
}

let headerPointerId: number | null = null;
let headerStartX = 0;
let headerStartY = 0;
let headerDragging = false;

function onHeaderPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  if ((event.target as HTMLElement).closest("button, input, textarea, select, a")) {
    return;
  }
  headerPointerId = event.pointerId;
  headerStartX = event.clientX;
  headerStartY = event.clientY;
  headerDragging = false;
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onHeaderPointerMove(event: PointerEvent) {
  if (headerPointerId !== event.pointerId || headerDragging) return;
  const dx = Math.abs(event.clientX - headerStartX);
  const dy = Math.abs(event.clientY - headerStartY);
  if (dx > HEADER_DRAG_THRESHOLD || dy > HEADER_DRAG_THRESHOLD) {
    headerDragging = true;
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {
      // ignore
    }
    void startWindowDrag();
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

async function handlePeek(clientY: number) {
  if (unpeekTimer) {
    clearTimeout(unpeekTimer);
    unpeekTimer = null;
  }
  if (!isDocked.value || dockPeeked.value || peekInFlight.value) return;

  const gen = ++peekGeneration;
  peekInFlight.value = true;
  pointerInsideFloat = true;

  try {
    await peekDockedFloat(clientY);
    if (gen !== peekGeneration) {
      peekInFlight.value = false;
      // 世代失配：补偿收回，避免 HWND 已展开而 UI 仍为 strip
      try {
        await unpeekDockedFloat();
      } catch (err) {
        console.error("peek mismatch unpeek failed", err);
      }
      return;
    }
    dockPeeked.value = true;
    peekInFlight.value = false;
    if (!pointerInsideFloat) {
      await handleUnpeek();
      return;
    }
    void fetchFloatTodos();
  } catch (err) {
    console.error("peekDockedFloat failed", err);
    if (gen === peekGeneration) {
      peekInFlight.value = false;
      dockPeeked.value = false;
    }
  }
}

/** strip leave：仅 peek 进行中忽略（卸载 strip 会合成 mouseleave） */
function onStripUnpeek() {
  if (peekInFlight.value) return;
  scheduleUnpeek();
}

function scheduleUnpeek() {
  // 始终记录离开，createOpen 时仅跳过启动 timer
  pointerInsideFloat = false;
  if (createOpen.value) return;
  if (unpeekTimer) clearTimeout(unpeekTimer);
  unpeekTimer = setTimeout(() => {
    void handleUnpeek();
  }, 300);
}

function cancelScheduledUnpeek() {
  pointerInsideFloat = true;
  if (unpeekTimer) {
    clearTimeout(unpeekTimer);
    unpeekTimer = null;
  }
}

function onCreateClose() {
  createOpen.value = false;
  if (isDocked.value && (dockPeeked.value || peekInFlight.value) && !pointerInsideFloat) {
    scheduleUnpeek();
  }
}

async function handleUnpeek() {
  if (unpeekTimer) {
    clearTimeout(unpeekTimer);
    unpeekTimer = null;
  }
  if (createOpen.value) return;
  if (!isDocked.value || (!dockPeeked.value && !peekInFlight.value)) return;

  peekGeneration += 1;
  selectedId.value = null;
  try {
    await unpeekDockedFloat();
    dockPeeked.value = false;
    peekInFlight.value = false;
  } catch (err) {
    console.error("unpeekDockedFloat failed", err);
    await refreshFloatState(true);
  }
}

function onPanelMouseEnter() {
  cancelScheduledUnpeek();
}

function onPanelMouseLeave() {
  if (isDocked.value) {
    scheduleUnpeek();
  }
}

/** pointerup 仅作辅助；主路径为 onMoved settle + 主按键抬起 */
function onPointerUpCapture(event: PointerEvent) {
  onPrimaryPointerUp(event);
}

async function onSettingsUpdated(payload: SettingsDto) {
  visibleCount.value = payload.floatVisibleCount;
  defaultMode.value = payload.floatDefaultMode;
  await refreshFloatState();
  await fetchFloatTodos();
}

onMounted(async () => {
  document.addEventListener("pointerdown", onPrimaryPointerDown, true);
  document.addEventListener("pointerup", onPointerUpCapture, true);
  document.addEventListener("pointercancel", onPointerUpCapture, true);
  unlistenSettings = await listen<SettingsDto>(TAURI_EVENTS.SETTINGS_UPDATED, (event) => {
    void onSettingsUpdated(event.payload);
  });
  await ensureMovedListener();
  await loadSettings();
  await fetchFloatTodos();
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onPrimaryPointerDown, true);
  document.removeEventListener("pointerup", onPointerUpCapture, true);
  document.removeEventListener("pointercancel", onPointerUpCapture, true);
  if (unpeekTimer) clearTimeout(unpeekTimer);
  clearDragTimers();
  unlistenSettings?.();
  unlistenMoved?.();
  unlistenMoved = null;
  movedListenReady = null;
});
</script>

<template>
  <div
    class="float-root"
    :class="{
      'mode-ball': isBall,
      'mode-panel': showPanelUi && !isDocked,
      'mode-docked-strip': showDockStrip,
      'mode-docked-peek': isDocked && (dockPeeked || peekInFlight),
      'edge-left': dockEdge === 'left',
      'edge-right': dockEdge === 'right',
    }"
  >
    <FloatBall
      v-if="isBall"
      :active-count="activeCount"
      :overdue-count="overdueCount"
      :due-today-count="dueTodayCount"
      @expand="expandToPanel"
      @drag-start="startWindowDrag"
    />

    <FloatDockStrip
      v-else-if="showDockStrip"
      :edge="dockEdge"
      @peek="handlePeek"
      @unpeek="onStripUnpeek"
      @drag-start="startWindowDrag"
    />

    <div
      v-else-if="showPanelUi"
      class="float-app"
      @mouseenter="onPanelMouseEnter"
      @mouseleave="onPanelMouseLeave"
    >
      <header
        class="float-header"
        @pointerdown="onHeaderPointerDown"
        @pointermove="onHeaderPointerMove"
        @pointerup="onHeaderPointerUp"
        @pointercancel="onHeaderPointerUp"
      >
        <span class="title">Only Todo</span>
        <span class="count">{{ activeCount }} 待办</span>
        <template v-if="isDocked">
          <button type="button" class="icon-btn" title="取消挂边" @click="handleUndock">⬚</button>
        </template>
        <template v-else>
          <button
            v-if="isBallHome"
            type="button"
            class="icon-btn"
            title="收起"
            @click="collapseToBall"
          >
            −
          </button>
          <button type="button" class="icon-btn" title="挂边" @click="handleDock">▐</button>
        </template>
        <button type="button" class="icon-btn" title="新建" @click="openCreateModal">+</button>
        <button type="button" class="icon-btn" title="关闭" @click="closePanel">×</button>
      </header>

      <p v-if="error" class="error">{{ error }}</p>

      <FloatTaskList
        :todos="todos"
        :selected-id="selectedId"
        :loading="loading"
        @select="selectTodo"
        @open-main="openMainWithTodo"
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
        @close="selectedId = null"
        @save="handleSave"
        @transition="transitionTo"
        @add-reminder="handleAddReminder"
        @remove-reminder="(id) => selectedId && removeReminder(id, selectedId)"
      />

      <CreateTodoModal
        ref="createModalRef"
        :open="createOpen"
        compact
        :teleport="false"
        @close="onCreateClose"
        @submit="handleCreateSubmit"
      />
    </div>
  </div>
</template>

<style>
/* 透明窗口底色，避免圆球/竖线背后白底 */
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
.float-root {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  font-family: Inter, system-ui, sans-serif;
  background: transparent;
}

.float-app {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.96);
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  overflow: hidden;
  color: #111827;
}

/* 右挂边：左侧圆角；左挂边：右侧圆角 */
.mode-docked-peek.edge-right .float-app {
  border-radius: 8px 0 0 8px;
}

.mode-docked-peek.edge-left .float-app {
  border-radius: 0 8px 8px 0;
}

.float-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
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
  color: #6b7280;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.icon-btn:hover {
  background: #f3f4f6;
}

.error {
  margin: 0;
  padding: 8px 12px;
  background: #fef2f2;
  color: #dc2626;
  font-size: 13px;
}
</style>
