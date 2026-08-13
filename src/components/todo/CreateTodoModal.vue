<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { CreateTodoDto, CreateTodoFormModel } from "@/api/types";
import { DEFAULT_CREATE_TODO_FORM, PRIORITY_OPTIONS } from "@/api/types";
import { fromLocalDatetimeInput } from "@/utils/date";
import { validateCreateTodoForm } from "@/utils/validation";
import AppButton from "@/components/common/AppButton.vue";
import AppDateTimePicker from "@/components/common/AppDateTimePicker.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import AppModal from "@/components/common/AppModal.vue";
import AppSelect from "@/components/common/AppSelect.vue";

const props = defineProps<{
  open: boolean;
  compact?: boolean;
  /** 默认 true；悬浮窗内禁用 Teleport */
  teleport?: boolean;
  /** 打开时预填截止日期（datetime-local） */
  initialDueDate?: string | null;
}>();

const emit = defineEmits<{
  close: [];
  submit: [dto: CreateTodoDto];
}>();

const { t } = useI18n();
const form = ref<CreateTodoFormModel>(DEFAULT_CREATE_TODO_FORM());
const error = ref<string | null>(null);
const submitting = ref(false);
const titleInput = ref<HTMLInputElement | null>(null);

const priorityOptions = computed(() =>
  PRIORITY_OPTIONS.map((p) => ({ value: p, label: t(`priority.${p}`) })),
);

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      form.value = DEFAULT_CREATE_TODO_FORM();
      if (props.initialDueDate) {
        form.value.dueDate = props.initialDueDate;
      }
      error.value = null;
      submitting.value = false;
      await nextTick();
      titleInput.value?.focus();
    }
  },
);

function parseTags(text: string): string[] {
  const seen = new Set<string>();
  const tags: string[] = [];
  for (const part of text.split(/[,，\s]+/)) {
    const tag = part.trim();
    if (!tag) continue;
    const key = tag.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    tags.push(tag);
  }
  return tags;
}

function handleSubmit() {
  const validationError = validateCreateTodoForm(form.value);
  if (validationError) {
    error.value = validationError;
    return;
  }

  error.value = null;
  submitting.value = true;

  const dto: CreateTodoDto = {
    title: form.value.title.trim(),
  };
  if (form.value.description.trim()) {
    dto.description = form.value.description.trim();
  }
  if (form.value.priority !== "Medium") {
    dto.priority = form.value.priority;
  }
  const due = fromLocalDatetimeInput(form.value.dueDate);
  if (due) {
    dto.dueDate = due;
  }
  const tags = parseTags(form.value.tagsText);
  if (tags.length) {
    dto.tags = tags;
  }

  emit("submit", dto);
}

/** 父组件在异步创建完成后调用，重置提交状态 */
function resetSubmitting() {
  submitting.value = false;
}

function setError(message: string) {
  error.value = message;
  submitting.value = false;
}

defineExpose({ resetSubmitting, setError });
</script>

<template>
  <AppModal
    :open="open"
    :compact="compact"
    :teleport="teleport"
    :title="$t('create.title')"
    @close="emit('close')"
  >
    <AppErrorBanner v-if="error" :message="error" />

    <label class="app-field">
      <span>
        {{ $t("create.titleLabel") }}
        <span class="required">{{ $t("create.titleRequired") }}</span>
      </span>
      <input
        ref="titleInput"
        v-model="form.title"
        type="text"
        :placeholder="$t('create.titlePlaceholder')"
        maxlength="200"
        @keydown.enter.prevent="handleSubmit"
      />
    </label>

    <label class="app-field">
      <span>{{ $t("create.description") }}</span>
      <textarea
        v-model="form.description"
        :placeholder="$t('create.descriptionPlaceholder')"
        rows="3"
        maxlength="5000"
      />
    </label>

    <label class="app-field">
      <span>{{ $t("create.priority") }}</span>
      <AppSelect
        v-model="form.priority"
        :options="priorityOptions"
        :compact="compact"
        :teleport="teleport !== false"
      />
    </label>

    <label class="app-field">
      <span>{{ $t("create.dueDate") }}</span>
      <AppDateTimePicker
        v-model="form.dueDate"
        :compact="compact"
        :teleport="teleport !== false"
      />
    </label>

    <label class="app-field">
      <span>{{ $t("create.tags") }}</span>
      <input v-model="form.tagsText" type="text" :placeholder="$t('create.tagsPlaceholder')" />
    </label>

    <div class="app-modal-actions">
      <AppButton variant="ghost" @click="emit('close')">{{ $t("common.cancel") }}</AppButton>
      <AppButton variant="primary" :disabled="submitting" @click="handleSubmit">
        {{ submitting ? $t("common.creating") : $t("common.create") }}
      </AppButton>
    </div>
  </AppModal>
</template>

<style scoped>
@import "../../styles/forms.css";

.required {
  color: var(--color-danger);
}
</style>
