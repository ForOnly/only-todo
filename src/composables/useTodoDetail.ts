/**
 * 任务详情编辑与提醒管理。
 * 绑定当前选中的 todoId；主窗 Inspector 用 debounce autosave。
 */
import { ref, watch, type Ref } from "vue";

import * as reminderApi from "@/api/reminders";
import * as todoApi from "@/api/todos";
import type { ReminderDto, RepeatType, TodoDto, TodoStatus, UpdateTodoDto } from "@/api/types";
import { fromLocalDatetimeInput, toLocalDatetimeInput } from "@/utils/date";
import { formatErrorMessage } from "@/utils/error";
import { confirm } from "@/composables/useAppConfirm";
import { i18n } from "@/i18n";

const AUTOSAVE_MS = 450;
const SAVED_CLEAR_MS = 1500;

export type SaveStatus = "idle" | "dirty" | "saving" | "saved";

export interface UseRemindersReturn {
  reminders: Ref<ReminderDto[]>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  fetchReminders: (todoId: string) => Promise<void>;
  addReminder: (todoId: string, localDatetime: string, repeatType?: RepeatType) => Promise<void>;
  updateReminder: (id: string, todoId: string, localDatetime: string) => Promise<void>;
  removeReminder: (id: string, todoId: string) => Promise<void>;
  snoozeReminder: (id: string, todoId: string, minutes: number) => Promise<void>;
}

export interface UseTodoDetailReturn {
  detail: Ref<TodoDto | null>;
  editTitle: Ref<string>;
  editDescription: Ref<string>;
  editPriority: Ref<TodoDto["priority"]>;
  editDueDate: Ref<string>;
  editTags: Ref<string[]>;
  saving: Ref<boolean>;
  saveStatus: Ref<SaveStatus>;
  error: Ref<string | null>;
  isDirty: () => boolean;
  save: () => Promise<boolean>;
  scheduleAutosave: () => void;
  flushAutosave: () => Promise<boolean>;
  reload: () => Promise<void>;
  transitionTo: (status: TodoStatus) => Promise<void>;
  remove: () => Promise<boolean>;
  restore: () => Promise<boolean>;
}

export function useReminders(selectedId: Ref<string | null>): UseRemindersReturn {
  const reminders = ref<ReminderDto[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  let reminderGen = 0;

  async function fetchReminders(todoId: string): Promise<void> {
    const gen = ++reminderGen;
    loading.value = true;
    error.value = null;
    try {
      const list = await reminderApi.listReminders(todoId);
      if (gen !== reminderGen) return;
      reminders.value = list;
    } catch (err) {
      if (gen !== reminderGen) return;
      error.value = formatErrorMessage(err);
      reminders.value = [];
    } finally {
      if (gen === reminderGen) {
        loading.value = false;
      }
    }
  }

  watch(selectedId, (id) => {
    reminderGen += 1;
    if (id) {
      void fetchReminders(id);
    } else {
      reminders.value = [];
      loading.value = false;
    }
  });

  async function addReminder(
    todoId: string,
    localDatetime: string,
    repeatType: RepeatType = "none",
  ): Promise<void> {
    const remindAt = new Date(localDatetime).toISOString();
    await reminderApi.createReminder({ todoId, remindAt, repeatType });
    await fetchReminders(todoId);
  }

  async function updateReminder(id: string, todoId: string, localDatetime: string): Promise<void> {
    await reminderApi.updateReminder({
      id,
      remindAt: new Date(localDatetime).toISOString(),
      enabled: true,
    });
    await fetchReminders(todoId);
  }

  async function removeReminder(id: string, todoId: string): Promise<void> {
    await reminderApi.deleteReminder(id);
    await fetchReminders(todoId);
  }

  async function snoozeReminder(id: string, todoId: string, minutes: number): Promise<void> {
    await reminderApi.snoozeReminder(id, minutes);
    await fetchReminders(todoId);
  }

  return {
    reminders,
    loading,
    error,
    fetchReminders,
    addReminder,
    updateReminder,
    removeReminder,
    snoozeReminder,
  };
}

export function useTodoDetail(
  selectedId: Ref<string | null>,
  onUpdated: () => Promise<void>,
): UseTodoDetailReturn {
  const detail = ref<TodoDto | null>(null);
  const editTitle = ref("");
  const editDescription = ref("");
  const editPriority = ref<TodoDto["priority"]>("Medium");
  const editDueDate = ref("");
  const editTags = ref<string[]>([]);
  const saving = ref(false);
  const saveStatus = ref<SaveStatus>("idle");
  const error = ref<string | null>(null);
  let loadGen = 0;
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;
  let savedClearTimer: ReturnType<typeof setTimeout> | null = null;
  let suppressAutosave = false;
  let savePromise: Promise<boolean> | null = null;

  function clearSavedTimer(): void {
    if (savedClearTimer) {
      clearTimeout(savedClearTimer);
      savedClearTimer = null;
    }
  }

  function syncSaveStatus(): void {
    if (saving.value) {
      saveStatus.value = "saving";
    } else if (isDirty()) {
      // dirty 优先于 saved，避免「已保存」窗口内再编辑仍显示已保存
      saveStatus.value = "dirty";
    } else if (saveStatus.value === "saved") {
      // 保持 saved 直到定时器清除
    } else {
      saveStatus.value = "idle";
    }
  }

  function markSaved(): void {
    saveStatus.value = "saved";
    clearSavedTimer();
    savedClearTimer = setTimeout(() => {
      savedClearTimer = null;
      syncSaveStatus();
    }, SAVED_CLEAR_MS);
  }

  function applyDetailToForm(todo: TodoDto): void {
    detail.value = todo;
    editTitle.value = todo.title;
    editDescription.value = todo.description;
    editPriority.value = todo.priority;
    editDueDate.value = toLocalDatetimeInput(todo.dueDate);
    editTags.value = [...todo.tags];
    saveStatus.value = "idle";
    clearSavedTimer();
  }

  async function loadDetail(id: string): Promise<void> {
    const gen = ++loadGen;
    suppressAutosave = true;
    try {
      const todo = await todoApi.getTodo(id);
      if (gen !== loadGen) return;
      applyDetailToForm(todo);
      error.value = null;
    } catch (err) {
      if (gen !== loadGen) return;
      error.value = formatErrorMessage(err);
      detail.value = null;
      saveStatus.value = "idle";
    } finally {
      suppressAutosave = false;
    }
  }

  watch(selectedId, async (id) => {
    if (autosaveTimer) {
      clearTimeout(autosaveTimer);
      autosaveTimer = null;
    }
    clearSavedTimer();
    if (!id) {
      loadGen += 1;
      detail.value = null;
      saveStatus.value = "idle";
      return;
    }
    await loadDetail(id);
  });

  watch([editTitle, editDescription, editPriority, editDueDate, editTags], () => {
    if (suppressAutosave || !selectedId.value || detail.value?.deletedAt) return;
    syncSaveStatus();
  });

  async function reload(): Promise<void> {
    if (!selectedId.value) return;
    await loadDetail(selectedId.value);
  }

  /** 表单相对已加载 detail 是否有未保存变更 */
  function isDirty(): boolean {
    if (!detail.value || detail.value.deletedAt) return false;
    const d = detail.value;
    if (editTitle.value.trim() !== d.title) return true;
    if (editDescription.value !== d.description) return true;
    if (editPriority.value !== d.priority) return true;
    if (editDueDate.value !== toLocalDatetimeInput(d.dueDate)) return true;
    if (JSON.stringify(editTags.value) !== JSON.stringify(d.tags)) return true;
    return false;
  }

  async function save(): Promise<boolean> {
    if (savePromise) return savePromise;

    if (!selectedId.value || !detail.value) return false;
    if (detail.value.deletedAt) return false;
    if (!isDirty()) return true;

    const snap = {
      id: selectedId.value,
      title: editTitle.value.trim(),
      description: editDescription.value,
      priority: editPriority.value,
      dueDate: fromLocalDatetimeInput(editDueDate.value),
      tags: [...editTags.value],
    };

    if (!snap.title) {
      error.value = i18n.global.t("validation.titleRequired");
      return false;
    }

    savePromise = (async () => {
      saving.value = true;
      saveStatus.value = "saving";
      error.value = null;
      try {
        const dto: UpdateTodoDto = {
          id: snap.id,
          title: snap.title,
          description: snap.description,
          priority: snap.priority,
          dueDate: snap.dueDate,
          tags: snap.tags,
        };
        const updated = await todoApi.updateTodo(dto);
        if (selectedId.value !== snap.id) return false;
        // 保存期间用户可能又改了字段：仅当仍干净时套用服务端结果
        if (!isDirty() || formMatchesSnap(snap)) {
          applyDetailToForm(updated);
          markSaved();
        } else {
          detail.value = updated;
          syncSaveStatus();
        }
        await onUpdated();
        return true;
      } catch (err) {
        if (selectedId.value === snap.id) {
          error.value = formatErrorMessage(err);
          syncSaveStatus();
        }
        return false;
      } finally {
        saving.value = false;
        savePromise = null;
        if (saveStatus.value === "saving") {
          syncSaveStatus();
        }
      }
    })();

    return savePromise;
  }

  function formMatchesSnap(snap: {
    title: string;
    description: string;
    priority: TodoDto["priority"];
    dueDate: string | null;
    tags: string[];
  }): boolean {
    if (editTitle.value.trim() !== snap.title) return false;
    if (editDescription.value !== snap.description) return false;
    if (editPriority.value !== snap.priority) return false;
    if (fromLocalDatetimeInput(editDueDate.value) !== snap.dueDate) return false;
    if (JSON.stringify(editTags.value) !== JSON.stringify(snap.tags)) return false;
    return true;
  }

  function scheduleAutosave(): void {
    if (suppressAutosave || !selectedId.value || detail.value?.deletedAt) return;
    syncSaveStatus();
    if (autosaveTimer) clearTimeout(autosaveTimer);
    autosaveTimer = setTimeout(() => {
      autosaveTimer = null;
      void save();
    }, AUTOSAVE_MS);
  }

  async function flushAutosave(): Promise<boolean> {
    if (autosaveTimer) {
      clearTimeout(autosaveTimer);
      autosaveTimer = null;
    }
    if (!selectedId.value || !detail.value || detail.value.deletedAt) return true;
    if (!isDirty()) return true;
    return save();
  }

  async function transitionTo(status: TodoStatus): Promise<void> {
    if (!detail.value || detail.value.deletedAt) return;
    const todoId = detail.value.id;
    try {
      const updated = await todoApi.transitionTodo(todoId, status);
      if (selectedId.value !== todoId) return;
      applyDetailToForm(updated);
      await onUpdated();
    } catch (err) {
      error.value = formatErrorMessage(err);
    }
  }

  async function remove(): Promise<boolean> {
    if (!selectedId.value) return false;
    const ok = await confirm({
      title: i18n.global.t("common.delete"),
      message: i18n.global.t("inspector.confirmDelete"),
      confirmLabel: i18n.global.t("common.delete"),
      danger: true,
    });
    if (!ok) return false;
    const todoId = selectedId.value;
    try {
      await todoApi.deleteTodo(todoId);
      selectedId.value = null;
      await onUpdated();
      return true;
    } catch (err) {
      error.value = formatErrorMessage(err);
      return false;
    }
  }

  async function restore(): Promise<boolean> {
    if (!selectedId.value) return false;
    const todoId = selectedId.value;
    try {
      const updated = await todoApi.restoreTodo(todoId);
      if (selectedId.value !== todoId) return false;
      applyDetailToForm(updated);
      await onUpdated();
      return true;
    } catch (err) {
      error.value = formatErrorMessage(err);
      return false;
    }
  }

  return {
    detail,
    editTitle,
    editDescription,
    editPriority,
    editDueDate,
    editTags,
    saving,
    saveStatus,
    error,
    isDirty,
    save,
    scheduleAutosave,
    flushAutosave,
    reload,
    transitionTo,
    remove,
    restore,
  };
}
