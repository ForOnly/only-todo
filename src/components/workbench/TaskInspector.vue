<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/common/AppButton.vue";
import AppDateTimePicker from "@/components/common/AppDateTimePicker.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import AppSelect from "@/components/common/AppSelect.vue";
import {
  PRIORITY_OPTIONS,
  REPEAT_TYPE_OPTIONS,
  SNOOZE_OPTIONS,
  type ReminderDto,
  type RepeatType,
  type TodoDto,
  type TodoStatus,
} from "@/api/types";
import type { SaveStatus } from "@/composables/useTodoDetail";
import { formatDate, toLocalDatetimeInput } from "@/utils/date";
import { statusActionLabel, useStatusActions } from "@/utils/statusActions";

const props = defineProps<{
  detail: TodoDto | null;
  editTitle: string;
  editDescription: string;
  editPriority: TodoDto["priority"];
  editDueDate: string;
  editTags: string[];
  saving: boolean;
  saveStatus: SaveStatus;
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

const { t } = useI18n();

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

const priorityOptions = computed(() =>
  PRIORITY_OPTIONS.map((p) => ({ value: p, label: t(`priority.${p}`) })),
);
const repeatOptions = computed(() =>
  REPEAT_TYPE_OPTIONS.map((rt) => ({ value: rt, label: t(`repeat.${rt}`) })),
);
const snoozeOptions = computed(() =>
  SNOOZE_OPTIONS.map((m) => ({
    value: String(m),
    label: t("inspector.snoozeMinutes", { n: m }),
  })),
);

const priorityModel = computed({
  get: () => props.editPriority,
  set: (v: string) => {
    emit("update:editPriority", v as TodoDto["priority"]);
    onField();
  },
});
const dueModel = computed({
  get: () => props.editDueDate,
  set: (v: string) => {
    emit("update:editDueDate", v);
    onField();
  },
});

const autosaveLabel = computed(() => {
  switch (props.saveStatus) {
    case "saving":
      return t("inspector.autosaveSaving");
    case "saved":
      return t("inspector.autosaveSaved");
    case "dirty":
      return t("inspector.autosaveDirty");
    default:
      return t("inspector.autosaveIdle");
  }
});

const tagSuggestions = computed(() => {
  const current = new Set(props.editTags.map((tag) => tag.toLowerCase()));
  const q = tagInput.value.trim().toLowerCase();
  return props.suggestedTags
    .filter((tag) => !current.has(tag.toLowerCase()))
    .filter((tag) => !q || tag.toLowerCase().includes(q))
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

function onSnooze(reminderId: string, value: string) {
  if (!value) return;
  emit("snoozeReminder", reminderId, Number(value));
}
</script>

<template>
  <aside class="inspector" :aria-label="$t('inspector.aria')">
    <div v-if="!detail" class="empty">
      <p class="empty-title">{{ $t("inspector.emptyTitle") }}</p>
      <p class="empty-hint">{{ $t("inspector.emptyHint") }}</p>
    </div>

    <template v-else>
      <header class="inspector-header">
        <div class="header-left">
          <span class="status-pill">{{ $t(`status.${detail.status}`) }}</span>
          <span
            class="save-hint"
            :class="{
              muted: saveStatus === 'idle',
              dirty: saveStatus === 'dirty',
              saving: saveStatus === 'saving',
              saved: saveStatus === 'saved',
            }"
          >
            {{ autosaveLabel }}
          </span>
        </div>
        <button
          type="button"
          class="close-btn"
          :aria-label="$t('common.close')"
          @click="emit('close')"
        >
          ×
        </button>
      </header>

      <div class="inspector-body">
        <AppErrorBanner v-if="error" :message="error" />

        <template v-if="isDeleted">
          <p class="trash-note">{{ $t("inspector.trashNote") }}</p>
          <div class="action-row">
            <AppButton variant="primary" @click="emit('restore')">
              {{ $t("inspector.restore") }}
            </AppButton>
          </div>
          <label class="app-field">
            <span>{{ $t("inspector.title") }}</span>
            <input :value="editTitle" disabled />
          </label>
          <label class="app-field">
            <span>{{ $t("inspector.description") }}</span>
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
              {{ statusActionLabel(detail!.status, action.target, t) }}
            </AppButton>
            <AppButton variant="danger" @click="emit('remove')">{{ $t("inspector.remove") }}</AppButton>
          </div>

          <label class="app-field">
            <span>{{ $t("inspector.title") }}</span>
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
            <span>{{ $t("inspector.description") }}</span>
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
            <span>{{ $t("inspector.priority") }}</span>
            <AppSelect v-model="priorityModel" :options="priorityOptions" />
          </label>

          <label class="app-field">
            <span>{{ $t("inspector.dueDate") }}</span>
            <AppDateTimePicker v-model="dueModel" />
          </label>

          <div class="app-field">
            <span>{{ $t("inspector.tags") }}</span>
            <div class="chips">
              <span v-for="tag in editTags" :key="tag" class="chip">
                {{ tag }}
                <button
                  type="button"
                  class="chip-x"
                  :aria-label="$t('inspector.removeTag', { tag })"
                  @click="removeTag(tag)"
                >
                  ×
                </button>
              </span>
            </div>
            <input
              v-model="tagInput"
              type="text"
              :placeholder="$t('inspector.tagPlaceholder')"
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
            <h3>{{ $t("inspector.reminders") }}</h3>
            <AppErrorBanner v-if="remindersError" :message="remindersError" />
            <div v-if="remindersLoading" class="muted">{{ $t("inspector.loadingReminders") }}</div>
            <ul v-else class="reminder-list">
              <li v-for="reminder in reminders" :key="reminder.id" class="reminder-item">
                <template v-if="editingReminderId === reminder.id">
                  <AppDateTimePicker v-model="editingReminderAt" :clearable="false" />
                  <div class="reminder-actions">
                    <AppButton variant="primary" @click="commitEditReminder">
                      {{ $t("common.save") }}
                    </AppButton>
                    <AppButton variant="ghost" @click="cancelEditReminder">
                      {{ $t("common.cancel") }}
                    </AppButton>
                  </div>
                </template>
                <template v-else>
                  <div class="reminder-main" :class="{ disabled: !reminder.enabled }">
                    <span>{{ formatDate(reminder.nextTriggerAt) }}</span>
                    <span class="muted">{{ $t(`repeat.${reminder.repeatType}`) }}</span>
                    <span v-if="!reminder.enabled" class="muted">{{ $t("inspector.disabled") }}</span>
                    <span
                      v-else-if="reminder.snoozeCount > 0"
                      class="muted"
                      :title="$t('inspector.originalTime')"
                    >
                      {{ $t("inspector.originalTimeLabel", { time: formatDate(reminder.remindAt) }) }}
                    </span>
                  </div>
                  <div class="reminder-actions">
                    <AppButton
                      variant="ghost"
                      :disabled="!reminder.enabled"
                      @click="startEditReminder(reminder)"
                    >
                      {{ $t("inspector.reschedule") }}
                    </AppButton>
                    <AppSelect
                      class="snooze"
                      model-value=""
                      :options="snoozeOptions"
                      :placeholder="$t('inspector.snooze')"
                      :disabled="!reminder.enabled"
                      @update:model-value="onSnooze(reminder.id, $event)"
                    />
                    <AppButton variant="danger" @click="emit('removeReminder', reminder.id)">
                      {{ $t("inspector.deleteShort") }}
                    </AppButton>
                  </div>
                </template>
              </li>
              <li v-if="!reminders.length" class="muted">{{ $t("inspector.noReminders") }}</li>
            </ul>

            <div class="add-reminder">
              <AppDateTimePicker v-model="reminderInput" :clearable="false" />
              <AppSelect v-model="repeatType" :options="repeatOptions" />
              <AppButton variant="primary" @click="submitReminder">{{ $t("common.add") }}</AppButton>
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
  background: var(--color-surface);
  min-height: 0;
}

.inspector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid var(--color-border);
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
  border-radius: var(--radius-pill);
  background: var(--color-surface-muted);
  color: var(--color-text-secondary);
}

.save-hint {
  font-size: 12px;
  color: var(--color-accent);
}

.save-hint.muted,
.muted {
  color: var(--color-muted);
  font-size: 12px;
}

.save-hint.dirty {
  color: var(--color-overdue);
}

.save-hint.saved {
  color: var(--color-success);
}

.close-btn {
  border: none;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  color: var(--color-muted);
  cursor: pointer;
  padding: 0 4px;
  transition: color var(--transition-fast);
}

.close-btn:hover {
  color: var(--color-text);
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
  color: var(--color-muted);
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
  border-radius: var(--radius-pill);
  background: var(--color-surface-muted);
  font-size: 12px;
  color: var(--color-text-secondary);
}

.chip-x {
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--color-muted);
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
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  border-radius: 4px;
  padding: 2px 8px;
  font: inherit;
  font-size: 12px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.suggest:hover {
  background: var(--color-surface-muted);
}

.reminders h3 {
  margin: 8px 0 10px;
  font-size: 13px;
  color: var(--color-muted);
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
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
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
  min-width: 100px;
}

.snooze :deep(.trigger) {
  padding: 6px 8px;
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
  color: var(--color-text-secondary);
}

.empty-hint {
  margin: 0;
  font-size: 13px;
  color: var(--color-muted);
  line-height: 1.5;
}
</style>
