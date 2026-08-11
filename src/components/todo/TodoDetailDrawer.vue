<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";

import type { ReminderDto, RepeatType, TodoDto, TodoStatus } from "@/api/types";
import TodoDetail from "@/components/todo/TodoDetail.vue";

defineProps<{
  open: boolean;
  detail: TodoDto | null;
  editTitle: string;
  editDescription: string;
  editPriority: TodoDto["priority"];
  editDueDate: string;
  editTags: string;
  saving: boolean;
  error: string | null;
  reminders: ReminderDto[];
  remindersLoading: boolean;
  remindersError: string | null;
}>();

const emit = defineEmits<{
  close: [];
  "update:editTitle": [value: string];
  "update:editDescription": [value: string];
  "update:editPriority": [value: TodoDto["priority"]];
  "update:editDueDate": [value: string];
  "update:editTags": [value: string];
  save: [];
  transition: [status: TodoStatus];
  remove: [];
  addReminder: [datetime: string, repeatType?: RepeatType];
  removeReminder: [id: string];
  snoozeReminder: [id: string, minutes: number];
}>();

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    emit("close");
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="drawer-backdrop" @click.self="emit('close')">
      <aside class="drawer" role="dialog" aria-label="编辑任务">
        <header class="drawer-header">
          <h2>编辑任务</h2>
          <button class="close-btn" type="button" aria-label="关闭" @click="emit('close')">
            ×
          </button>
        </header>
        <div class="drawer-body">
          <TodoDetail
            :detail="detail"
            :edit-title="editTitle"
            :edit-description="editDescription"
            :edit-priority="editPriority"
            :edit-due-date="editDueDate"
            :edit-tags="editTags"
            :saving="saving"
            :error="error"
            :reminders="reminders"
            :reminders-loading="remindersLoading"
            :reminders-error="remindersError"
            @update:edit-title="emit('update:editTitle', $event)"
            @update:edit-description="emit('update:editDescription', $event)"
            @update:edit-priority="emit('update:editPriority', $event)"
            @update:edit-due-date="emit('update:editDueDate', $event)"
            @update:edit-tags="emit('update:editTags', $event)"
            @save="emit('save')"
            @close="emit('close')"
            @transition="emit('transition', $event)"
            @remove="emit('remove')"
            @add-reminder="(dt, rt) => emit('addReminder', dt, rt)"
            @remove-reminder="emit('removeReminder', $event)"
            @snooze-reminder="(id, min) => emit('snoozeReminder', id, min)"
          />
        </div>
      </aside>
    </div>
  </Teleport>
</template>

<style scoped>
.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.35);
  z-index: 900;
  display: flex;
  justify-content: flex-end;
  backdrop-filter: blur(2px);
}

.drawer {
  width: min(420px, 92vw);
  height: 100%;
  background: #fff;
  box-shadow: -8px 0 24px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  animation: slideIn 0.2s ease-out;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.drawer-header h2 {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
}

.close-btn {
  border: none;
  background: none;
  font-size: 24px;
  line-height: 1;
  color: #6b7280;
  cursor: pointer;
  padding: 0 4px;
}

.close-btn:hover {
  color: #111827;
}

.drawer-body {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
</style>
