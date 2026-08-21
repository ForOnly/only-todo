<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/common/AppButton.vue";
import AppDateTimePicker from "@/components/common/AppDateTimePicker.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import AppSelect from "@/components/common/AppSelect.vue";
import AppTagChip from "@/components/common/AppTagChip.vue";
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
import {
  formatAbsoluteDateTime,
  formatRelativeAge,
  formatRelativeDue,
} from "@/utils/timeMeta";
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
const showAddReminder = ref(false);

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
      return "";
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

const timeLines = computed(() => {
  const detail = props.detail;
  if (!detail) return [];
  const lines: { key: string; label: string; value: string }[] = [
    {
      key: "created",
      label: t("inspector.createdAt"),
      value: formatAbsoluteDateTime(detail.createdAt),
    },
    {
      key: "updated",
      label: t("inspector.updatedAt"),
      value: formatRelativeAge(detail.updatedAt, t) ?? formatAbsoluteDateTime(detail.updatedAt),
    },
  ];
  if (detail.dueDate) {
    lines.push({
      key: "due",
      label: t("inspector.dueDate"),
      value: formatRelativeDue(detail.dueDate, detail.status, t) ?? formatAbsoluteDateTime(detail.dueDate),
    });
  }
  if (detail.completedAt) {
    lines.push({
      key: "completed",
      label: t("inspector.completedAt"),
      value: formatAbsoluteDateTime(detail.completedAt),
    });
  }
  return lines;
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
  showAddReminder.value = false;
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
  <aside v-if="detail" class="inspector" :aria-label="$t('inspector.aria')">
    <header class="inspector-header">
      <div class="header-left">
        <span class="status-pill" :data-status="detail.status">{{ $t(`status.${detail.status}`) }}</span>
        <span
          v-if="autosaveLabel"
          class="save-hint"
          :class="{
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
            class="title-input"
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

        <section class="time-block" :aria-label="$t('inspector.timeSection')">
          <h3>{{ $t("inspector.timeSection") }}</h3>
          <dl class="time-lines">
            <div v-for="line in timeLines" :key="line.key" class="time-line">
              <dt>{{ line.label }}</dt>
              <dd>{{ line.value }}</dd>
            </div>
          </dl>
        </section>

        <div class="app-field">
          <span>{{ $t("inspector.tags") }}</span>
          <div class="chips">
            <AppTagChip
              v-for="tag in editTags"
              :key="tag"
              :label="tag"
              removable
              @remove="removeTag"
            />
          </div>
          <input
            v-model="tagInput"
            type="text"
            :placeholder="$t('inspector.tagPlaceholder')"
            @keydown="onTagKeydown"
          />
          <div v-if="tagSuggestions.length" class="suggestions">
            <AppTagChip
              v-for="tag in tagSuggestions"
              :key="tag"
              :label="tag"
              clickable
              @click="addTag"
            />
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

          <div v-if="!showAddReminder">
            <AppButton variant="ghost" @click="showAddReminder = true">
              {{ $t("inspector.addReminder") }}
            </AppButton>
          </div>
          <div v-else class="add-reminder">
            <AppDateTimePicker v-model="reminderInput" :clearable="false" />
            <AppSelect v-model="repeatType" :options="repeatOptions" />
            <div class="reminder-actions">
              <AppButton variant="primary" @click="submitReminder">{{ $t("common.add") }}</AppButton>
              <AppButton variant="ghost" @click="showAddReminder = false">
                {{ $t("common.cancel") }}
              </AppButton>
            </div>
          </div>
        </section>
      </template>
    </div>
  </aside>
</template>

<style scoped>
@import "@/styles/forms.css";

.inspector {
  width: 100%;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
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

.status-pill[data-status="Doing"] {
  background: var(--color-accent-soft);
  color: var(--color-accent);
}

.status-pill[data-status="Done"] {
  background: var(--color-surface-muted);
  color: var(--color-success);
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

.suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;
}

.reminders h3,
.time-block h3 {
  margin: 8px 0 10px;
  font-size: 13px;
  color: var(--color-muted);
}

.time-lines {
  margin: 0 0 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.time-line {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}

.time-line dt {
  margin: 0;
  color: var(--color-muted);
  flex-shrink: 0;
}

.time-line dd {
  margin: 0;
  text-align: right;
  color: var(--color-text-secondary);
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

.title-input {
  font-size: 17px;
  font-weight: 650;
}
</style>
