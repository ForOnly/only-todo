<script setup lang="ts">
import { ref, watch } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import AppModal from "@/components/common/AppModal.vue";
import type { FloatDefaultMode, SortBy, SortOrder, UpdateSettingsDto } from "@/api/types";
import { parseListDefaultSort, serializeListDefaultSort } from "@/utils/listSort";

const props = defineProps<{
  open: boolean;
  notificationEnabled: boolean;
  listDefaultSort: string;
  floatAlwaysOnTop: boolean;
  floatVisibleCount: number;
  floatAutoShow: boolean;
  floatDefaultMode: FloatDefaultMode;
  floatHoverPreview: boolean;
  autostartEnabled: boolean;
  /** 父组件保存失败时展示；成功时父组件关闭弹窗 */
  error?: string | null;
  saving?: boolean;
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
const localHoverPreview = ref(props.floatHoverPreview);
const localAutostart = ref(props.autostartEnabled);
const localSortBy = ref<SortBy>("priority");
const localSortOrder = ref<SortOrder>("desc");

watch(
  () => props.open,
  (open) => {
    if (open) {
      localNotification.value = props.notificationEnabled;
      localAlwaysOnTop.value = props.floatAlwaysOnTop;
      localVisibleCount.value = props.floatVisibleCount;
      localAutoShow.value = props.floatAutoShow;
      localDefaultMode.value = props.floatDefaultMode;
      localHoverPreview.value = props.floatHoverPreview;
      localAutostart.value = props.autostartEnabled;
      const sort = parseListDefaultSort(props.listDefaultSort);
      localSortBy.value = sort.sortBy;
      localSortOrder.value = sort.sortOrder;
    }
  },
);

function save() {
  emit("update", {
    notificationEnabled: localNotification.value,
    listDefaultSort: serializeListDefaultSort({
      sortBy: localSortBy.value,
      sortOrder: localSortOrder.value,
    }),
    floatAlwaysOnTop: localAlwaysOnTop.value,
    floatVisibleCount: localVisibleCount.value,
    floatAutoShow: localAutoShow.value,
    floatDefaultMode: localDefaultMode.value,
    floatHoverPreview: localHoverPreview.value,
    autostartEnabled: localAutostart.value,
  });
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
      <h3>列表</h3>
      <label class="setting-row">
        <span>默认排序</span>
        <select v-model="localSortBy" class="select-input">
          <option value="priority">优先级</option>
          <option value="dueDate">截止日期</option>
          <option value="updatedAt">最近更新</option>
          <option value="createdAt">创建时间</option>
          <option value="title">标题</option>
        </select>
      </label>
      <label class="setting-row">
        <span>方向</span>
        <select v-model="localSortOrder" class="select-input">
          <option value="desc">降序</option>
          <option value="asc">升序</option>
        </select>
      </label>
      <p class="hint">打开主窗或切换视图时使用此默认排序（可在列表内临时改）。</p>
    </section>

    <section class="group">
      <h3>伴侣</h3>
      <label class="setting-row">
        <input v-model="localAlwaysOnTop" type="checkbox" />
        <span>伴侣始终置顶</span>
      </label>
      <label class="setting-row">
        <input v-model="localAutoShow" type="checkbox" />
        <span>启动时显示伴侣</span>
      </label>
      <div class="setting-block">
        <label class="setting-row">
          <span>默认形态</span>
          <select v-model="localDefaultMode" class="select-input">
            <option value="ball">小圆球</option>
            <option value="panel">面板</option>
          </select>
        </label>
        <p class="hint">小圆球为日常气泡，− 回到圆球；面板为迷你窗，− 贴到边缘。贴边后 − 收成条。</p>
      </div>
      <label class="setting-row">
        <input v-model="localHoverPreview" type="checkbox" />
        <span>贴边悬停预览</span>
      </label>
      <p class="hint">默认关。悬停约 0.4s 预览，离开约 0.3s 收起；单击预览可钉住。</p>
      <label class="setting-row">
        <span>伴侣列表条数</span>
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

    <p v-if="error" class="error">{{ error }}</p>

    <div class="app-modal-actions">
      <AppButton variant="primary" :disabled="saving" @click="save">
        {{ saving ? "保存中…" : "保存" }}
      </AppButton>
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

.setting-block {
  padding: 4px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  cursor: pointer;
  padding: 4px 0;
}

.hint {
  margin: 0 0 4px;
  padding-left: 0;
  font-size: 12px;
  color: #6b7280;
  line-height: 1.4;
}

.error {
  margin: 0 0 8px;
  font-size: 13px;
  color: #b91c1c;
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
