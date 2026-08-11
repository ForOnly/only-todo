<script setup lang="ts">
import { ref, watch } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import AppModal from "@/components/common/AppModal.vue";
import type { FloatDefaultMode, UpdateSettingsDto } from "@/api/types";

const props = defineProps<{
  open: boolean;
  notificationEnabled: boolean;
  floatAlwaysOnTop: boolean;
  floatVisibleCount: number;
  floatAutoShow: boolean;
  floatDefaultMode: FloatDefaultMode;
  autostartEnabled: boolean;
}>();

const emit = defineEmits<{
  close: [];
  update: [payload: UpdateSettingsDto];
}>();

const localNotification = ref(props.notificationEnabled);
const localAlwaysOnTop = ref(props.floatAlwaysOnTop);
const localVisibleCount = ref(props.floatVisibleCount);
const localAutoShow = ref(props.floatAutoShow);
const localDefaultMode = ref<FloatDefaultMode>(props.floatDefaultMode);
const localAutostart = ref(props.autostartEnabled);

watch(
  () => props.open,
  (open) => {
    if (open) {
      localNotification.value = props.notificationEnabled;
      localAlwaysOnTop.value = props.floatAlwaysOnTop;
      localVisibleCount.value = props.floatVisibleCount;
      localAutoShow.value = props.floatAutoShow;
      localDefaultMode.value = props.floatDefaultMode;
      localAutostart.value = props.autostartEnabled;
    }
  },
);

function save() {
  emit("update", {
    notificationEnabled: localNotification.value,
    floatAlwaysOnTop: localAlwaysOnTop.value,
    floatVisibleCount: localVisibleCount.value,
    floatAutoShow: localAutoShow.value,
    floatDefaultMode: localDefaultMode.value,
    autostartEnabled: localAutostart.value,
  });
  emit("close");
}
</script>

<template>
  <AppModal :open="open" title="设置" @close="emit('close')">
    <section class="group">
      <h3>通知</h3>
      <label class="setting-row">
        <input v-model="localNotification" type="checkbox" />
        <span>启用提醒通知</span>
      </label>
    </section>

    <section class="group">
      <h3>悬浮窗</h3>
      <label class="setting-row">
        <input v-model="localAlwaysOnTop" type="checkbox" />
        <span>窗口置顶</span>
      </label>
      <label class="setting-row">
        <input v-model="localAutoShow" type="checkbox" />
        <span>启动时显示悬浮窗</span>
      </label>
      <label class="setting-row">
        <span>默认形态</span>
        <select v-model="localDefaultMode" class="select-input">
          <option value="ball">小圆球</option>
          <option value="panel">面板</option>
        </select>
      </label>
      <label class="setting-row">
        <span>显示条数</span>
        <input
          v-model.number="localVisibleCount"
          type="number"
          min="1"
          max="20"
          class="number-input"
        />
      </label>
    </section>

    <section class="group">
      <h3>系统</h3>
      <label class="setting-row">
        <input v-model="localAutostart" type="checkbox" />
        <span>开机自启</span>
      </label>
    </section>

    <div class="app-modal-actions">
      <AppButton variant="primary" @click="save">保存</AppButton>
    </div>
  </AppModal>
</template>

<style scoped>
@import "@/styles/forms.css";

.group {
  margin-bottom: 16px;
}

.group h3 {
  margin: 0 0 8px;
  font-size: 13px;
  color: #6b7280;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  cursor: pointer;
  padding: 4px 0;
}

.number-input {
  width: 64px;
  padding: 4px 8px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
}

.select-input {
  padding: 4px 8px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
}
</style>
