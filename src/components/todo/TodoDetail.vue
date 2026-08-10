<script setup lang="ts">
import { ref } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import {
  PRIORITY_LABELS,
  PRIORITY_OPTIONS,
  STATUS_LABELS,
  type ReminderDto,
  type TodoDto,
} from "@/api/types";
import { formatDate } from "@/utils/date";

defineProps<{
  detail: TodoDto | null;
  editTitle: string;
  editDescription: string;
  editPriority: TodoDto["priority"];
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
  save: [];
  close: [];
  toggleComplete: [];
  remove: [];
  addReminder: [datetime: string];
  removeReminder: [id: string];
}>();

const reminderInput = ref("");

function submitReminder() {
  if (!reminderInput.value) return;
  emit("addReminder", reminderInput.value);
  reminderInput.value = "";
}
</script>

<template>
  <section class="detail-panel">
    <div v-if="!detail" class="empty">选择任务查看详情</div>
    <template v-else>
      <div class="toolbar">
        <AppButton @click="emit('toggleComplete')">
          {{ detail.status === "Done" ? "恢复为待办" : "标记完成" }}
        </AppButton>
        <AppButton variant="danger" @click="emit('remove')">删除</AppButton>
        <AppButton variant="ghost" @click="emit('close')">取消</AppButton>
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
          rows="6"
          :value="editDescription"
          @input="
            emit(
              'update:editDescription',
              ($event.target as HTMLTextAreaElement).value,
            )
          "
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

      <div class="meta">
        <span>状态：{{ STATUS_LABELS[detail.status] }}</span>
        <span>更新：{{ formatDate(detail.updatedAt) }}</span>
      </div>

      <section class="reminders">
        <h3>提醒</h3>
        <AppErrorBanner v-if="remindersError" :message="remindersError" />
        <div class="reminder-form">
          <input v-model="reminderInput" type="datetime-local" class="datetime-input" />
          <AppButton @click="submitReminder">添加</AppButton>
        </div>
        <div v-if="remindersLoading" class="hint">加载提醒...</div>
        <ul v-else class="reminder-list">
          <li v-for="item in reminders" :key="item.id">
            <span>{{ formatDate(item.remindAt) }}</span>
            <span :class="{ disabled: !item.enabled }">
              {{ item.enabled ? "启用" : "已触发" }}
            </span>
            <button class="link" type="button" @click="emit('removeReminder', item.id)">
              删除
            </button>
          </li>
          <li v-if="reminders.length === 0" class="hint">暂无提醒</li>
        </ul>
      </section>
    </template>
  </section>
</template>

<style scoped>
@import "../../styles/forms.css";

.detail-panel {
  padding: 16px;
  background: #fff;
}

.empty,
.hint {
  color: #6b7280;
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.meta {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #6b7280;
  margin-bottom: 20px;
}

.reminders h3 {
  margin: 0 0 8px;
  font-size: 15px;
}

.reminder-form {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.datetime-input {
  flex: 1;
  padding: 8px 10px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font: inherit;
}

.reminder-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.reminder-list li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid #f3f4f6;
  font-size: 14px;
}

.disabled {
  color: #9ca3af;
}

.link {
  margin-left: auto;
  border: none;
  background: none;
  color: #2563eb;
  cursor: pointer;
}
</style>
