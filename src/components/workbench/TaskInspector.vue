<script setup lang="ts">
import { computed, ref } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import {
  PRIORITY_LABELS,
  PRIORITY_OPTIONS,
  REPEAT_TYPE_LABELS,
  REPEAT_TYPE_OPTIONS,
  SNOOZE_OPTIONS,
  STATUS_LABELS,
  type ReminderDto,
  type RepeatType,
  type TodoDto,
  type TodoStatus,
} from "@/api/types";
import { formatDate, toLocalDatetimeInput } from "@/utils/date";
import { useStatusActions } from "@/utils/statusActions";

const props = defineProps<{
  detail: TodoDto | null;
  editTitle: string;
  editDescription: string;
  editPriority: TodoDto["priority"];
  editDueDate: string;
  editTags: string[];
  saving: boolean;
  error: string | null;
  reminders: ReminderDto[];
  remindersLoading: boolean;
  remindersError: string | null;
  suggestedTags: string[];
}>();

const emit = defineEmits<{
  "update:editTitle": [value: string];
  "update:editDescription": [value: string];
  "update:editPriority": [value: TodoDto["priority"]];
  "update:editDueDate": [value: string];
  "update:editTags": [value: string[]];
  fieldChange: [];
  close: [];
  transition: [status: TodoStatus];
  remove: [];
  restore: [];
  addReminder: [datetime: string, repeatType?: RepeatType];
  updateReminder: [id: string, datetime: string];
  removeReminder: [id: string];
  snoozeReminder: [id: string, minutes: number];
}>();

const tagInput = ref("");
const reminderInput = ref("");
const repeatType = ref<RepeatType>("none");
const editingReminderId = ref<string | null>(null);
const editingReminderAt = ref("");

const isDeleted = computed(() => Boolean(props.detail?.deletedAt));
const statusRef = computed(() =>
  props.detail && !isDeleted.value ? props.detail.status : null,
);
const { actions } = useStatusActions(statusRef);

const tagSuggestions = computed(() => {
  const current = new Set(props.editTags.map((t) => t.toLowerCase()));
  const q = tagInput.value.trim().toLowerCase();
  return props.suggestedTags
    .filter((t) => !current.has(t.toLowerCase()))
    .filter((t) => !q || t.toLowerCase().includes(q))
    .slice(0, 8);
});

function onField() {
  emit("fieldChange");
}

function addTag(raw: string) {
  const tag = raw.trim();
  if (!tag) return;
  if (props.editTags.some((t) => t.toLowerCase() === tag.toLowerCase())) {
    tagInput.value = "";
    return;
  }
  emit("update:editTags", [...props.editTags, tag]);
  tagInput.value = "";
  onField();
}

function removeTag(tag: string) {
  emit(
    "update:editTags",
    props.editTags.filter((t) => t !== tag),
  );
  onField();
}

function onTagKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === ",") {
    event.preventDefault();
    addTag(tagInput.value.replace(/,/g, ""));
  }
}

function submitReminder() {
  if (!reminderInput.value) return;
  emit("addReminder", reminderInput.value, repeatType.value);
  reminderInput.value = "";
}

function startEditReminder(reminder: ReminderDto) {
  editingReminderId.value = reminder.id;
  editingReminderAt.value = toLocalDatetimeInput(reminder.nextTriggerAt);
}

function commitEditReminder() {
  if (!editingReminderId.value || !editingReminderAt.value) return;
  emit("updateReminder", editingReminderId.value, editingReminderAt.value);
  editingReminderId.value = null;
  editingReminderAt.value = "";
}

function cancelEditReminder() {
  editingReminderId.value = null;
  editingReminderAt.value = "";
}
</script>

<template>
  <aside class="inspector" aria-label="任务编辑">
    <div v-if="!detail" class="empty">
      <p class="empty-title">选择一条任务</p>
      <p class="empty-hint">在列表中点选即可在此编辑；字段会自动保存。</p>
    </div>

    <template v-else>
      <header class="inspector-header">
        <div class="header-left">
          <span class="status-pill">{{ STATUS_LABELS[detail.status] }}</span>
          <span v-if="saving" class="save-hint">保存中…</span>
          <span v-else class="save-hint muted">自动保存</span>
        </div>
        <button type="button" class="close-btn" aria-label="关闭" @click="emit('close')">×</button>
      </header>

      <div class="inspector-body">
        <AppErrorBanner v-if="error" :message="error" />

        <template v-if="isDeleted">
          <p class="trash-note">此任务在回收站中。可恢复，或保持删除。</p>
          <div class="action-row">
            <AppButton variant="primary" @click="emit('restore')">恢复</AppButton>
          </div>
          <label class="app-field">
            <span>标题</span>
            <input :value="editTitle" disabled />
          </label>
          <label class="app-field">
            <span>描述</span>
            <textarea rows="4" :value="editDescription" disabled />
          </label>
        </template>

        <template v-else>
          <div class="action-row">
            <AppButton
              v-for="action in actions"
              :key="action.target"
              :variant="action.target === 'Done' ? 'primary' : 'ghost'"
              @click="emit('transition', action.target)"
            >
              {{ action.label }}
            </AppButton>
            <AppButton variant="danger" @click="emit('remove')">删除</AppButton>
          </div>

          <label class="app-field">
            <span>标题</span>
            <input
              :value="editTitle"
              maxlength="200"
              @input="
                emit('update:editTitle', ($event.target as HTMLInputElement).value);
                onField();
              "
            />
          </label>

          <label class="app-field">
            <span>描述</span>
            <textarea
              rows="5"
              maxlength="5000"
              :value="editDescription"
              @input="
                emit('update:editDescription', ($event.target as HTMLTextAreaElement).value);
                onField();
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
                );
                onField();
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
              @input="
                emit('update:editDueDate', ($event.target as HTMLInputElement).value);
                onField();
              "
            />
          </label>

          <div class="app-field">
            <span>标签</span>
            <div class="chips">
              <span v-for="tag in editTags" :key="tag" class="chip">
                {{ tag }}
                <button type="button" class="chip-x" :aria-label="`移除 ${tag}`" @click="removeTag(tag)">
                  ×
                </button>
              </span>
            </div>
            <input
              v-model="tagInput"
              type="text"
              placeholder="输入后回车添加"
              @keydown="onTagKeydown"
            />
            <div v-if="tagSuggestions.length" class="suggestions">
              <button
                v-for="tag in tagSuggestions"
                :key="tag"
                type="button"
                class="suggest"
                @click="addTag(tag)"
              >
                {{ tag }}
              </button>
            </div>
          </div>

          <section class="reminders">
            <h3>提醒</h3>
            <AppErrorBanner v-if="remindersError" :message="remindersError" />
            <div v-if="remindersLoading" class="muted">加载提醒…</div>
            <ul v-else class="reminder-list">
              <li v-for="reminder in reminders" :key="reminder.id" class="reminder-item">
                <template v-if="editingReminderId === reminder.id">
                  <input v-model="editingReminderAt" type="datetime-local" />
                  <div class="reminder-actions">
                    <AppButton variant="primary" @click="commitEditReminder">保存</AppButton>
                    <AppButton variant="ghost" @click="cancelEditReminder">取消</AppButton>
                  </div>
                </template>
                <template v-else>
                  <div class="reminder-main" :class="{ disabled: !reminder.enabled }">
                    <span>{{ formatDate(reminder.nextTriggerAt) }}</span>
                    <span class="muted">{{ REPEAT_TYPE_LABELS[reminder.repeatType] }}</span>
                    <span v-if="!reminder.enabled" class="muted">已关闭</span>
                    <span
                      v-else-if="reminder.snoozeCount > 0"
                      class="muted"
                      title="原定时间"
                    >
                      原定 {{ formatDate(reminder.remindAt) }}
                    </span>
                  </div>
                  <div class="reminder-actions">
                    <AppButton
                      variant="ghost"
                      :disabled="!reminder.enabled"
                      @click="startEditReminder(reminder)"
                    >
                      改期
                    </AppButton>
                    <select
                      class="snooze"
                      :disabled="!reminder.enabled"
                      @change="
                        emit(
                          'snoozeReminder',
                          reminder.id,
                          Number(($event.target as HTMLSelectElement).value),
                        );
                        ($event.target as HTMLSelectElement).value = '';
                      "
                    >
                      <option value="" disabled selected>延后</option>
                      <option v-for="m in SNOOZE_OPTIONS" :key="m" :value="m">{{ m }} 分</option>
                    </select>
                    <AppButton variant="danger" @click="emit('removeReminder', reminder.id)">
                      删
                    </AppButton>
                  </div>
                </template>
              </li>
              <li v-if="!reminders.length" class="muted">暂无提醒</li>
            </ul>

            <div class="add-reminder">
              <input v-model="reminderInput" type="datetime-local" />
              <select v-model="repeatType">
                <option v-for="rt in REPEAT_TYPE_OPTIONS" :key="rt" :value="rt">
                  {{ REPEAT_TYPE_LABELS[rt] }}
                </option>
              </select>
              <AppButton variant="primary" @click="submitReminder">添加</AppButton>
            </div>
          </section>
        </template>
      </div>
    </template>
  </aside>
</template>

<style scoped>
@import "@/styles/forms.css";

.inspector {
  width: 340px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: #fff;
  min-height: 0;
}

.inspector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid #e2e8f0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-pill {
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  background: #f1f5f9;
  color: #334155;
}

.save-hint {
  font-size: 12px;
  color: #2563eb;
}

.save-hint.muted,
.muted {
  color: #94a3b8;
  font-size: 12px;
}

.close-btn {
  border: none;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  color: #64748b;
  cursor: pointer;
  padding: 0 4px;
}

.inspector-body {
  padding: 14px;
  overflow-y: auto;
  flex: 1;
}

.action-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}

.trash-note {
  margin: 0 0 12px;
  font-size: 13px;
  color: #64748b;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
  min-height: 8px;
}

.chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 999px;
  background: #f1f5f9;
  font-size: 12px;
  color: #334155;
}

.chip-x {
  border: none;
  background: transparent;
  cursor: pointer;
  color: #64748b;
  padding: 0;
  font-size: 14px;
  line-height: 1;
}

.suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;
}

.suggest {
  border: 1px solid #e2e8f0;
  background: #fff;
  border-radius: 4px;
  padding: 2px 8px;
  font: inherit;
  font-size: 12px;
  color: #475569;
  cursor: pointer;
}

.suggest:hover {
  background: #f8fafc;
}

.reminders h3 {
  margin: 8px 0 10px;
  font-size: 13px;
  color: #64748b;
}

.reminder-list {
  list-style: none;
  margin: 0 0 12px;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.reminder-item {
  padding: 8px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.reminder-main {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 13px;
}

.reminder-main.disabled {
  opacity: 0.65;
}

.reminder-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.snooze {
  padding: 6px 8px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font: inherit;
  font-size: 13px;
}

.add-reminder {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty {
  padding: 40px 24px;
  text-align: center;
}

.empty-title {
  margin: 0 0 6px;
  font-weight: 600;
  color: #334155;
}

.empty-hint {
  margin: 0;
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.5;
}
</style>
