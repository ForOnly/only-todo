<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { listen } from "@tauri-apps/api/event";

import * as todoApi from "@/api/todos";
import { getSettings } from "@/api/settings";
import type { CompanionSession, SettingsDto, TodoDto } from "@/api/types";
import { TITLE_MAX_LENGTH } from "@/api/types";
import {
  companionClickChrome,
  companionCollapseToStrip,
  companionMinimize,
  companionPointerCluster,
  companionRefreshSession,
  getCompanionSession,
} from "@/api/window";
import FloatTaskList from "@/components/float/FloatTaskList.vue";
import AppShellOverlays from "@/components/common/AppShellOverlays.vue";
import { useCompanionDrag } from "@/composables/useCompanionDrag";
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
const isPreview = computed(() => session.value?.panelMode === "preview");
/** × 提示：圆球家临时板「回球」，其余「收成条」 */
const collapseLabel = computed(() => {
  const current = session.value;
  if (current?.placement === "free" && current.homeShape === "ball") {
    return t("companion.backToBall");
  }
  return t("companion.collapseToStrip");
});

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

/** 预览态快加未提交文字：Rust hold_preview，指针/焦点仍如实上报 */
const hasQuickDraft = computed(() => quickTitle.value.trim().length > 0);
const pointerInside = ref(false);
const bodyFocused = ref(false);

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
    // 列表刷新后若预览项已不在，关闭预览
    if (selectedId.value && !todos.value.some((t) => t.id === selectedId.value)) {
      selectedId.value = null;
    }
  } catch (err) {
    if (token !== fetchToken) return;
    error.value = formatErrorMessage(err);
  } finally {
    if (token === fetchToken) {
      loading.value = false;
    }
  }
}

function closeDetail() {
  selectedId.value = null;
}

function selectTodo(id: string) {
  if (selectedId.value === id) {
    closeDetail();
    return;
  }
  selectedId.value = id;
}

async function openMainWithTodo(id?: string | null) {
  const todoId = id ?? selectedId.value;
  await todoApi.showMainWindow(todoId == null ? undefined : todoId);
}

async function openMainWindow() {
  await openMainWithTodo(selectedId.value);
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
  closeDetail();
  try {
    applySession(await companionMinimize());
  } catch (err) {
    console.error("minimize failed", err);
    error.value = formatErrorMessage(err);
  }
}

async function collapsePanel() {
  closeDetail();
  try {
    applySession(await companionCollapseToStrip());
  } catch (err) {
    console.error("collapsePanel failed", err);
    error.value = formatErrorMessage(err);
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
  pointerInside.value = true;
  if (!session.value?.hoverPreview) return;
  try {
    applySession(
      await companionPointerCluster("body", {
        inside: true,
        hold: hasQuickDraft.value,
      }),
    );
  } catch (err) {
    console.error("body cluster enter failed", err);
  }
}

async function onClusterLeave() {
  pointerInside.value = false;
  if (!session.value?.hoverPreview) return;
  try {
    applySession(
      await companionPointerCluster("body", {
        inside: false,
        hold: hasQuickDraft.value,
      }),
    );
  } catch (err) {
    console.error("body cluster leave failed", err);
  }
}

async function onFocusIn() {
  bodyFocused.value = true;
  if (!session.value?.hoverPreview) return;
  try {
    applySession(
      await companionPointerCluster("body", {
        focused: true,
        hold: hasQuickDraft.value,
      }),
    );
  } catch {
    // ignore
  }
}

async function onFocusOut(event: FocusEvent) {
  const root = event.currentTarget as HTMLElement;
  if (event.relatedTarget instanceof Node && root.contains(event.relatedTarget)) {
    return;
  }
  bodyFocused.value = false;
  if (!session.value?.hoverPreview) return;
  try {
    applySession(
      await companionPointerCluster("body", {
        focused: false,
        hold: hasQuickDraft.value,
      }),
    );
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
  selectedId.value = null;
});

watch(shouldFocusQuickAdd, async (focus) => {
  if (!focus) return;
  await nextTick();
  quickInput.value?.focus();
});

watch(hasQuickDraft, async (hasDraft) => {
  try {
    applySession(
      await companionPointerCluster("body", {
        inside: pointerInside.value,
        focused: bodyFocused.value,
        hold: hasDraft,
      }),
    );
  } catch {
    // ignore
  }
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
      <span class="count-pill">{{ $t("companion.activeCount", { n: activeCount }) }}</span>
      <div class="header-actions">
        <button
          type="button"
          class="open-main-btn"
          :title="$t('companion.openMain')"
          @click.stop="openMainWindow"
        >
          {{ $t("companion.openMain") }}
        </button>
        <button
          type="button"
          class="icon-btn"
          :title="collapseLabel"
          :aria-label="collapseLabel"
          @click.stop="collapsePanel"
        >
          ×
        </button>
      </div>
    </header>

    <form class="quick-add" :class="{ busy: creating }" @submit.prevent="submitQuickAdd">
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

    <FloatTaskList
      :todos="todos"
      :selected-id="selectedId"
      :loading="loading"
      @select="selectTodo"
      @open-main="openMainWithTodo"
      @complete="completeTodo"
      @close-peek="closeDetail"
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
  background: color-mix(in srgb, var(--color-surface) 94%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-border) 80%, transparent);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
  color: var(--color-text);
  font-family: var(--font-ui);
}

.preview-hint {
  margin: 0;
  padding: 3px 12px;
  font-size: 11px;
  letter-spacing: 0.02em;
  color: var(--color-accent);
  background: var(--color-accent-soft);
  text-align: center;
}

.edge-right {
  border-radius: var(--radius-md) 0 0 var(--radius-md);
}

.edge-left {
  border-radius: 0 var(--radius-md) var(--radius-md) 0;
}

.float-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px 6px;
  background: transparent;
  cursor: grab;
  user-select: none;
  touch-action: none;
}

.title {
  font-weight: 650;
  font-size: 13px;
  letter-spacing: 0.01em;
}

.count-pill {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  font-size: 11px;
  font-weight: 500;
  color: var(--color-muted);
  background: var(--color-bg-accent);
}

.header-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 2px;
}

.open-main-btn {
  margin-right: 4px;
  padding: 4px 10px;
  border: none;
  border-radius: var(--radius-pill);
  background: var(--color-accent-soft);
  color: var(--color-accent);
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.open-main-btn:hover {
  background: var(--color-accent);
  color: var(--color-on-accent);
}

.icon-btn {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-muted);
  cursor: pointer;
  font-size: 15px;
  line-height: 1;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.icon-btn:hover {
  background: var(--color-bg-accent);
  color: var(--color-text);
}

.quick-add {
  padding: 4px 12px 10px;
}

.quick-add input {
  width: 100%;
  box-sizing: border-box;
  padding: 9px 12px;
  border: 1px solid transparent;
  border-radius: var(--radius-pill);
  font-size: 13px;
  color: var(--color-text);
  background: var(--color-surface-muted);
  outline: none;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease,
    background 0.15s ease,
    opacity 0.15s ease;
}

.quick-add input:focus {
  border-color: var(--color-accent);
  background: var(--color-surface);
  box-shadow: var(--focus-ring);
}

.quick-add.busy input {
  opacity: 0.65;
}

.error {
  margin: 0 12px 8px;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  background: var(--color-danger-bg);
  color: var(--color-danger);
  font-size: 12px;
}
</style>
