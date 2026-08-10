<script setup lang="ts">
import AppButton from "@/components/common/AppButton.vue";
import AppModal from "@/components/common/AppModal.vue";

defineProps<{
  open: boolean;
  notificationEnabled: boolean;
}>();

const emit = defineEmits<{
  close: [];
  "update:notificationEnabled": [value: boolean];
}>();
</script>

<template>
  <AppModal :open="open" title="设置" @close="emit('close')">
    <label class="setting-row">
      <input
        type="checkbox"
        :checked="notificationEnabled"
        @change="
          emit(
            'update:notificationEnabled',
            ($event.target as HTMLInputElement).checked,
          )
        "
      />
      <span>启用提醒通知</span>
    </label>
    <p class="hint">关闭后，到期提醒不会弹出系统通知，但提醒记录仍会保留。</p>

    <div class="app-modal-actions">
      <AppButton variant="primary" @click="emit('close')">完成</AppButton>
    </div>
  </AppModal>
</template>

<style scoped>
@import "../../styles/forms.css";

.setting-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  cursor: pointer;
}

.hint {
  margin: 8px 0 0;
  font-size: 13px;
  color: #6b7280;
  line-height: 1.5;
}
</style>
