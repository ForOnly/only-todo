<script setup lang="ts">
import { computed, ref } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import {
  PRIORITY_LABELS,
  PRIORITY_OPTIONS,
  REPEAT_TYPE_LABELS,
  REPEAT_TYPE_OPTIONS,
  STATUS_LABELS,
  type ReminderDto,
  type RepeatType,
  type TodoDto,
  type TodoStatus,
} from "@/api/types";
import { formatDate } from "@/utils/date";
import { useStatusActions } from "@/utils/statusActions";

const props = defineProps<{
  detail: TodoDto;
  editTitle: string;
  editDescription: string;
  editPriority: TodoDto["priority"];
  editDueDate: string;
  saving: boolean;
  error: string | null;
  reminders: ReminderDto[];
  remindersLoading: boolean;
  remindersError: string | null;
}>();

const emit = defineEmits<{
  "update:editTitle": [value: string];
  "update:editDescription": [value: string];
  "update:editPriority": [value: TodoDto["priority"]];
  "update:editDueDate": [value: string];
  close: [];
  save: [];
  transition: [status: TodoStatus];
  addReminder: [datetime: string, repeatType?: RepeatType];
  removeReminder: [id: string];
}>();

const reminderInput = ref("");
const repeatType = ref<RepeatType>("none");
const statusRef = computed(() => props.detail.status);
const { actions } = useStatusActions(statusRef);

function submitReminder() {
  if (!reminderInput.value) return;
  emit("addReminder", reminderInput.value, repeatType.value);
  reminderInput.value = "";
}
</script>

<template>
  <section class="detail-panel">
    <div class="toolbar">
      <AppButton variant="ghost" @click="emit('close')">关闭</AppButton>
      <AppButton variant="primary" :disabled="saving" @click="emit('save')">
        {{ saving ? "保存中..." : "保存" }}
      </AppButton>
    </div>

    <AppErrorBanner v-if="error" :message="error" />

    <label class="app-field">
      <span>标题</span>
      <input
        :value="editTitle"
        @input="emit('update:editTitle', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <label class="app-field">
      <span>描述</span>
      <textarea
        rows="3"
        :value="editDescription"
        @input="emit('update:editDescription', ($event.target as HTMLTextAreaElement).value)"
      />
    </label>

    <label class="app-field">
      <span>优先级</span>
      <select
        :value="editPriority"
        @change="
          emit(
            'update:editPriority',
            ($event.target as HTMLSelectElement).value as TodoDto['priority'],
          )
        "
      >
        <option v-for="p in PRIORITY_OPTIONS" :key="p" :value="p">
          {{ PRIORITY_LABELS[p] }}
        </option>
      </select>
    </label>

    <label class="app-field">
      <span>截止日期</span>
      <input
        type="datetime-local"
        :value="editDueDate"
        @input="emit('update:editDueDate', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <div class="status-row">
      <span>状态 · {{ STATUS_LABELS[detail.status] }}</span>
      <div class="status-buttons">
        <AppButton
          v-for="action in actions"
          :key="action.target"
          :variant="action.target === 'Done' ? 'primary' : 'ghost'"
          @click="emit('transition', action.target)"
        >
          {{ action.label }}
        </AppButton>
      </div>
    </div>

    <section class="reminders">
      <h3>提醒</h3>
      <AppErrorBanner v-if="remindersError" :message="remindersError" />
      <div class="reminder-form">
        <input v-model="reminderInput" type="datetime-local" class="datetime-input" />
        <select v-model="repeatType">
          <option v-for="rt in REPEAT_TYPE_OPTIONS" :key="rt" :value="rt">
            {{ REPEAT_TYPE_LABELS[rt] }}
          </option>
        </select>
        <AppButton @click="submitReminder">添加</AppButton>
      </div>
      <div v-if="remindersLoading" class="hint">加载提醒...</div>
      <ul v-else class="reminder-list">
        <li v-for="item in reminders" :key="item.id" :class="{ disabled: !item.enabled }">
          <span>{{ formatDate(item.nextTriggerAt) }}</span>
          <span>{{ REPEAT_TYPE_LABELS[item.repeatType] }}</span>
          <span v-if="!item.enabled" class="hint">已关闭</span>
          <button class="link" type="button" @click="emit('removeReminder', item.id)">删除</button>
        </li>
      </ul>
    </section>
  </section>
</template>

<style scoped>
@import "@/styles/forms.css";

.detail-panel {
  border-top: 1px solid #e5e7eb;
  padding: 12px;
  max-height: 50vh;
  overflow-y: auto;
  background: #fff;
}

.toolbar {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-bottom: 12px;
}

.status-row {
  margin-bottom: 12px;
}

.status-row > span {
  display: block;
  font-size: 13px;
  color: #6b7280;
  margin-bottom: 6px;
}

.status-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.reminders h3 {
  margin: 0 0 8px;
  font-size: 14px;
}

.reminder-form {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.datetime-input {
  flex: 1;
  min-width: 140px;
  padding: 6px 8px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
}

.reminder-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.reminder-list li {
  display: flex;
  gap: 8px;
  padding: 6px 0;
  font-size: 13px;
  border-bottom: 1px solid #f3f4f6;
}

.reminder-list li.disabled {
  opacity: 0.65;
}

.link {
  margin-left: auto;
  border: none;
  background: none;
  color: #2563eb;
  cursor: pointer;
}

.hint {
  color: #6b7280;
  font-size: 13px;
}
</style>
