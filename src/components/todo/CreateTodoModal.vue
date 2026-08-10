<script setup lang="ts">
import { nextTick, ref, watch } from "vue";

import type { CreateTodoDto, CreateTodoFormModel } from "@/api/types";
import { DEFAULT_CREATE_TODO_FORM, PRIORITY_LABELS, PRIORITY_OPTIONS } from "@/api/types";
import { validateCreateTodoForm } from "@/utils/validation";
import AppButton from "@/components/common/AppButton.vue";
import AppErrorBanner from "@/components/common/AppErrorBanner.vue";
import AppModal from "@/components/common/AppModal.vue";

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
  submit: [dto: CreateTodoDto];
}>();

const form = ref<CreateTodoFormModel>(DEFAULT_CREATE_TODO_FORM());
const error = ref<string | null>(null);
const submitting = ref(false);
const titleInput = ref<HTMLInputElement | null>(null);

watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen) {
      form.value = DEFAULT_CREATE_TODO_FORM();
      error.value = null;
      submitting.value = false;
      await nextTick();
      titleInput.value?.focus();
    }
  },
);

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
  <AppModal :open="open" title="新建任务" @close="emit('close')">
    <AppErrorBanner v-if="error" :message="error" />

    <label class="app-field">
      <span>标题 <span class="required">*</span></span>
      <input
        ref="titleInput"
        v-model="form.title"
        type="text"
        placeholder="输入任务标题"
        maxlength="200"
        @keydown.enter.prevent="handleSubmit"
      />
    </label>

    <label class="app-field">
      <span>描述</span>
      <textarea
        v-model="form.description"
        placeholder="可选，补充任务详情"
        rows="4"
        maxlength="5000"
      />
    </label>

    <label class="app-field">
      <span>优先级</span>
      <select v-model="form.priority">
        <option v-for="p in PRIORITY_OPTIONS" :key="p" :value="p">
          {{ PRIORITY_LABELS[p] }}
        </option>
      </select>
    </label>

    <div class="app-modal-actions">
      <AppButton variant="ghost" @click="emit('close')">取消</AppButton>
      <AppButton variant="primary" :disabled="submitting" @click="handleSubmit">
        {{ submitting ? "创建中..." : "创建" }}
      </AppButton>
    </div>
  </AppModal>
</template>

<style scoped>
@import "../../styles/forms.css";

.required {
  color: #dc2626;
}
</style>
