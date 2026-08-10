/**
 * 任务详情编辑与提醒管理。
 * 绑定当前选中的 todoId，切换选中时自动加载详情与提醒列表。
 */
import { ref, watch, type Ref } from "vue";

import * as reminderApi from "@/api/reminders";
import * as todoApi from "@/api/todos";
import type { ReminderDto, TodoDto, TodoStatus } from "@/api/types";
import { formatErrorMessage } from "@/utils/error";

export interface UseRemindersReturn {
  reminders: Ref<ReminderDto[]>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  fetchReminders: (todoId: string) => Promise<void>;
  addReminder: (todoId: string, localDatetime: string) => Promise<void>;
  removeReminder: (id: string, todoId: string) => Promise<void>;
}

export interface UseTodoDetailReturn {
  detail: Ref<TodoDto | null>;
  editTitle: Ref<string>;
  editDescription: Ref<string>;
  editPriority: Ref<TodoDto["priority"]>;
  saving: Ref<boolean>;
  error: Ref<string | null>;
  save: () => Promise<boolean>;
  toggleComplete: () => Promise<void>;
  remove: () => Promise<void>;
}

export function useReminders(selectedId: Ref<string | null>): UseRemindersReturn {
  const reminders = ref<ReminderDto[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchReminders(todoId: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      reminders.value = await reminderApi.listReminders(todoId);
    } catch (err) {
      error.value = formatErrorMessage(err);
      reminders.value = [];
    } finally {
      loading.value = false;
    }
  }

  watch(selectedId, (id) => {
    if (id) {
      void fetchReminders(id);
    } else {
      reminders.value = [];
    }
  });

  async function addReminder(
    todoId: string,
    localDatetime: string,
  ): Promise<void> {
    const remindAt = new Date(localDatetime).toISOString();
    await reminderApi.createReminder({ todoId, remindAt });
    await fetchReminders(todoId);
  }

  async function removeReminder(id: string, todoId: string): Promise<void> {
    await reminderApi.deleteReminder(id);
    await fetchReminders(todoId);
  }

  return { reminders, loading, error, fetchReminders, addReminder, removeReminder };
}

export function useTodoDetail(
  selectedId: Ref<string | null>,
  onUpdated: () => Promise<void>,
): UseTodoDetailReturn {
  const detail = ref<TodoDto | null>(null);
  const editTitle = ref("");
  const editDescription = ref("");
  const editPriority = ref<TodoDto["priority"]>("Medium");
  const saving = ref(false);
  const error = ref<string | null>(null);

  watch(selectedId, async (id) => {
    if (!id) {
      detail.value = null;
      return;
    }
    try {
      const todo = await todoApi.getTodo(id);
      detail.value = todo;
      editTitle.value = todo.title;
      editDescription.value = todo.description;
      editPriority.value = todo.priority;
      error.value = null;
    } catch (err) {
      error.value = formatErrorMessage(err);
      detail.value = null;
    }
  });

  async function save(): Promise<boolean> {
    if (!selectedId.value) return false;
    saving.value = true;
    error.value = null;
    try {
      detail.value = await todoApi.updateTodo({
        id: selectedId.value,
        title: editTitle.value,
        description: editDescription.value,
        priority: editPriority.value,
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

  async function toggleComplete(): Promise<void> {
    if (!detail.value) return;
    const next: TodoStatus = detail.value.status === "Done" ? "Todo" : "Done";
    detail.value = await todoApi.transitionTodo(detail.value.id, next);
    editTitle.value = detail.value.title;
    await onUpdated();
  }

  async function remove(): Promise<void> {
    if (!selectedId.value) return;
    if (!window.confirm("确定删除此任务？删除后将从列表中隐藏。")) return;
    await todoApi.deleteTodo(selectedId.value);
    selectedId.value = null;
    await onUpdated();
  }

  return {
    detail,
    editTitle,
    editDescription,
    editPriority,
    saving,
    error,
    save,
    toggleComplete,
    remove,
  };
}
