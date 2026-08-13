<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/common/AppButton.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import {
  PRIORITY_OPTIONS,
  REPEAT_TYPE_OPTIONS,
  type ReminderDto,
  type RepeatType,
  type TodoDto,
  type TodoStatus,
} from "@/api/types";
import { formatDate } from "@/utils/date";
import { statusActionLabel, useStatusActions } from "@/utils/statusActions";

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

const { t } = useI18n();
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
      <AppButton variant="ghost" @click="emit('close')">{{ $t("companion.detailClose") }}</AppButton>
      <AppButton variant="primary" :disabled="saving" @click="emit('save')">
        {{ saving ? $t("common.saving") : $t("common.save") }}
      </AppButton>
    </div>

    <AppErrorBanner v-if="error" :message="error" />

    <label class="app-field">
      <span>{{ $t("companion.detailTitle") }}</span>
      <input
        :value="editTitle"
        @input="emit('update:editTitle', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <label class="app-field">
      <span>{{ $t("companion.detailDescription") }}</span>
      <textarea
        rows="3"
        :value="editDescription"
        @input="emit('update:editDescription', ($event.target as HTMLTextAreaElement).value)"
      />
    </label>

    <label class="app-field">
      <span>{{ $t("companion.detailPriority") }}</span>
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
          {{ $t(`priority.${p}`) }}
        </option>
      </select>
    </label>

    <label class="app-field">
      <span>{{ $t("companion.detailDue") }}</span>
      <input
        type="datetime-local"
        :value="editDueDate"
        @input="emit('update:editDueDate', ($event.target as HTMLInputElement).value)"
      />
    </label>

    <div class="status-row">
      <span>{{ $t("companion.detailStatus", { status: $t(`status.${detail.status}`) }) }}</span>
      <div class="status-buttons">
        <AppButton
          v-for="action in actions"
          :key="action.target"
          :variant="action.target === 'Done' ? 'primary' : 'ghost'"
          @click="emit('transition', action.target)"
        >
          {{ statusActionLabel(detail.status, action.target, t) }}
        </AppButton>
      </div>
    </div>

    <section class="reminders">
      <h3>{{ $t("companion.reminders") }}</h3>
      <AppErrorBanner v-if="remindersError" :message="remindersError" />
      <div class="reminder-form">
        <input v-model="reminderInput" type="datetime-local" class="datetime-input" />
        <select v-model="repeatType">
          <option v-for="rt in REPEAT_TYPE_OPTIONS" :key="rt" :value="rt">
            {{ $t(`repeat.${rt}`) }}
          </option>
        </select>
        <AppButton @click="submitReminder">{{ $t("companion.addReminder") }}</AppButton>
      </div>
      <div v-if="remindersLoading" class="hint">{{ $t("companion.loadingReminders") }}</div>
      <ul v-else class="reminder-list">
        <li v-for="item in reminders" :key="item.id" :class="{ disabled: !item.enabled }">
          <span>{{ formatDate(item.nextTriggerAt) }}</span>
          <span>{{ $t(`repeat.${item.repeatType}`) }}</span>
          <span v-if="!item.enabled" class="hint">{{ $t("companion.disabled") }}</span>
          <button
            class="link"
            type="button"
            :aria-label="$t('companion.deleteReminder')"
            @click="emit('removeReminder', item.id)"
          >
            {{ $t("companion.deleteReminder") }}
          </button>
        </li>
      </ul>
    </section>
  </section>
</template>

<style scoped>
@import "@/styles/forms.css";

.detail-panel {
  border-top: 1px solid var(--color-border);
  padding: 12px;
  max-height: 50vh;
  overflow-y: auto;
  background: var(--color-surface);
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
  color: var(--color-muted);
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
  color: var(--color-text);
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
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  background: var(--color-surface);
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
  border-bottom: 1px solid var(--color-border);
}

.reminder-list li.disabled {
  opacity: 0.65;
}

.link {
  margin-left: auto;
  border: none;
  background: none;
  color: var(--color-accent);
  cursor: pointer;
}

.hint {
  color: var(--color-muted);
  font-size: 13px;
}
</style>
