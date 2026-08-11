/**
 * 任务详情编辑与提醒管理。
 * 绑定当前选中的 todoId，切换选中时自动加载详情与提醒列表。
 */
import { ref, watch, type Ref } from "vue";

import * as reminderApi from "@/api/reminders";
import * as todoApi from "@/api/todos";
import type { ReminderDto, RepeatType, TodoDto, TodoStatus } from "@/api/types";
import { fromLocalDatetimeInput, toLocalDatetimeInput } from "@/utils/date";
import { formatErrorMessage } from "@/utils/error";

export interface UseRemindersReturn {
  reminders: Ref<ReminderDto[]>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  fetchReminders: (todoId: string) => Promise<void>;
  addReminder: (todoId: string, localDatetime: string, repeatType?: RepeatType) => Promise<void>;
  removeReminder: (id: string, todoId: string) => Promise<void>;
  snoozeReminder: (id: string, todoId: string, minutes: number) => Promise<void>;
}

export interface UseTodoDetailReturn {
  detail: Ref<TodoDto | null>;
  editTitle: Ref<string>;
  editDescription: Ref<string>;
  editPriority: Ref<TodoDto["priority"]>;
  editDueDate: Ref<string>;
  editTags: Ref<string>;
  saving: Ref<boolean>;
  error: Ref<string | null>;
  save: () => Promise<boolean>;
  transitionTo: (status: TodoStatus) => Promise<void>;
  remove: () => Promise<boolean>;
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
  const editTags = ref("");
  const saving = ref(false);
  const error = ref<string | null>(null);
  let loadGen = 0;

  watch(selectedId, async (id) => {
    const gen = ++loadGen;
    if (!id) {
      detail.value = null;
      return;
    }
    try {
      const todo = await todoApi.getTodo(id);
      if (gen !== loadGen) return;
      detail.value = todo;
      editTitle.value = todo.title;
      editDescription.value = todo.description;
      editPriority.value = todo.priority;
      editDueDate.value = toLocalDatetimeInput(todo.dueDate);
      editTags.value = todo.tags.join(", ");
      error.value = null;
    } catch (err) {
      if (gen !== loadGen) return;
      error.value = formatErrorMessage(err);
      detail.value = null;
    }
  });

  async function save(): Promise<boolean> {
    if (!selectedId.value) return false;
    saving.value = true;
    error.value = null;
    try {
      const tags = editTags.value
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean);
      detail.value = await todoApi.updateTodo({
        id: selectedId.value,
        title: editTitle.value,
        description: editDescription.value,
        priority: editPriority.value,
        dueDate: fromLocalDatetimeInput(editDueDate.value),
        tags,
      });
      await onUpdated();
      return true;
    } catch (err) {
      error.value = formatErrorMessage(err);
      return false;
    } finally {
      saving.value = false;
    }
  }

  async function transitionTo(status: TodoStatus): Promise<void> {
    if (!detail.value) return;
    try {
      detail.value = await todoApi.transitionTodo(detail.value.id, status);
      editTitle.value = detail.value.title;
      await onUpdated();
    } catch (err) {
      error.value = formatErrorMessage(err);
    }
  }

  async function remove(): Promise<boolean> {
    if (!selectedId.value) return false;
    if (!window.confirm("确定删除此任务？删除后将从列表中隐藏。")) return false;
    try {
      await todoApi.deleteTodo(selectedId.value);
      selectedId.value = null;
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
    transitionTo,
    remove,
  };
}
