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

const AUTOSAVE_MS = 450;

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
  error: Ref<string | null>;
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
  const error = ref<string | null>(null);
  let loadGen = 0;
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;
  let suppressAutosave = false;

  function applyDetailToForm(todo: TodoDto): void {
    detail.value = todo;
    editTitle.value = todo.title;
    editDescription.value = todo.description;
    editPriority.value = todo.priority;
    editDueDate.value = toLocalDatetimeInput(todo.dueDate);
    editTags.value = [...todo.tags];
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
    } finally {
      suppressAutosave = false;
    }
  }

  watch(selectedId, async (id) => {
    if (autosaveTimer) {
      clearTimeout(autosaveTimer);
      autosaveTimer = null;
    }
    if (!id) {
      loadGen += 1;
      detail.value = null;
      return;
    }
    await loadDetail(id);
  });

  async function reload(): Promise<void> {
    if (!selectedId.value) return;
    await loadDetail(selectedId.value);
  }

  async function save(): Promise<boolean> {
    if (!selectedId.value || !detail.value) return false;
    if (detail.value.deletedAt) return false;

    const snap = {
      id: selectedId.value,
      title: editTitle.value.trim(),
      description: editDescription.value,
      priority: editPriority.value,
      dueDate: fromLocalDatetimeInput(editDueDate.value),
      tags: [...editTags.value],
    };

    if (!snap.title) {
      error.value = "标题不能为空";
      return false;
    }

    saving.value = true;
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
      applyDetailToForm(updated);
      await onUpdated();
      return true;
    } catch (err) {
      if (selectedId.value === snap.id) {
        error.value = formatErrorMessage(err);
      }
      return false;
    } finally {
      saving.value = false;
    }
  }

  function scheduleAutosave(): void {
    if (suppressAutosave || !selectedId.value || detail.value?.deletedAt) return;
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
    if (!selectedId.value || !detail.value || detail.value.deletedAt) return false;
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
    if (!window.confirm("确定删除此任务？可在回收站恢复。")) return false;
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
    error,
    save,
    scheduleAutosave,
    flushAutosave,
    reload,
    transitionTo,
    remove,
    restore,
  };
}
