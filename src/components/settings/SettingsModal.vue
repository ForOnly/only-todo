<script setup lang="ts">
import { ref, watch } from "vue";

import AppButton from "@/components/common/AppButton.vue";
import AppModal from "@/components/common/AppModal.vue";
import type { FloatDefaultMode, SortBy, SortOrder, UiLocale, UiTheme, UpdateSettingsDto } from "@/api/types";
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
  uiTheme: UiTheme;
  uiLocale: UiLocale;
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
const localTheme = ref<UiTheme>(props.uiTheme);
const localLocale = ref<UiLocale>(props.uiLocale);
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
      localTheme.value = props.uiTheme;
      localLocale.value = props.uiLocale;
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
    uiTheme: localTheme.value,
    uiLocale: localLocale.value,
  });
}
</script>

<template>
  <AppModal :open="open" :title="$t('settings.title')" @close="emit('close')">
    <section class="group">
      <h3>{{ $t("settings.appearance") }}</h3>
      <label class="setting-row">
        <span>{{ $t("settings.theme") }}</span>
        <select v-model="localTheme" class="select-input">
          <option value="system">{{ $t("settings.themeSystem") }}</option>
          <option value="light">{{ $t("settings.themeLight") }}</option>
          <option value="dark">{{ $t("settings.themeDark") }}</option>
        </select>
      </label>
      <label class="setting-row">
        <span>{{ $t("settings.language") }}</span>
        <select v-model="localLocale" class="select-input">
          <option value="zh-CN">{{ $t("settings.langZh") }}</option>
          <option value="en-US">{{ $t("settings.langEn") }}</option>
        </select>
      </label>
    </section>

    <section class="group">
      <h3>{{ $t("settings.notifications") }}</h3>
      <label class="setting-row">
        <input v-model="localNotification" type="checkbox" />
        <span>{{ $t("settings.enableNotifications") }}</span>
      </label>
    </section>

    <section class="group">
      <h3>{{ $t("settings.list") }}</h3>
      <label class="setting-row">
        <span>{{ $t("settings.defaultSort") }}</span>
        <select v-model="localSortBy" class="select-input">
          <option value="priority">{{ $t("sort.priority") }}</option>
          <option value="dueDate">{{ $t("sort.dueDate") }}</option>
          <option value="updatedAt">{{ $t("sort.updatedAt") }}</option>
          <option value="createdAt">{{ $t("sort.createdAt") }}</option>
          <option value="title">{{ $t("sort.title") }}</option>
        </select>
      </label>
      <label class="setting-row">
        <span>{{ $t("settings.direction") }}</span>
        <select v-model="localSortOrder" class="select-input">
          <option value="desc">{{ $t("settings.desc") }}</option>
          <option value="asc">{{ $t("settings.asc") }}</option>
        </select>
      </label>
      <p class="hint">{{ $t("settings.sortHint") }}</p>
    </section>

    <section class="group">
      <h3>{{ $t("settings.companion") }}</h3>
      <label class="setting-row">
        <input v-model="localAlwaysOnTop" type="checkbox" />
        <span>{{ $t("settings.alwaysOnTop") }}</span>
      </label>
      <label class="setting-row">
        <input v-model="localAutoShow" type="checkbox" />
        <span>{{ $t("settings.autoShow") }}</span>
      </label>
      <div class="setting-block">
        <label class="setting-row">
          <span>{{ $t("settings.defaultShape") }}</span>
          <select v-model="localDefaultMode" class="select-input">
            <option value="ball">{{ $t("settings.ball") }}</option>
            <option value="panel">{{ $t("settings.panel") }}</option>
          </select>
        </label>
        <p class="hint">{{ $t("settings.shapeHint") }}</p>
      </div>
      <label class="setting-row">
        <input v-model="localHoverPreview" type="checkbox" />
        <span>{{ $t("settings.hoverPreview") }}</span>
      </label>
      <p class="hint">{{ $t("settings.hoverHint") }}</p>
      <label class="setting-row">
        <span>{{ $t("settings.visibleCount") }}</span>
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
      <h3>{{ $t("settings.system") }}</h3>
      <label class="setting-row">
        <input v-model="localAutostart" type="checkbox" />
        <span>{{ $t("settings.autostart") }}</span>
      </label>
    </section>

    <p v-if="error" class="error">{{ error }}</p>

    <div class="app-modal-actions">
      <AppButton variant="primary" :disabled="saving" @click="save">
        {{ saving ? $t("common.saving") : $t("common.save") }}
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
  color: var(--color-muted);
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
  color: var(--color-text);
}

.hint {
  margin: 0 0 4px;
  padding-left: 0;
  font-size: 12px;
  color: var(--color-muted);
  line-height: 1.4;
}

.error {
  margin: 0 0 8px;
  font-size: 13px;
  color: var(--color-danger);
}

.number-input {
  width: 64px;
  padding: 4px 8px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  background: var(--color-surface);
}

.select-input {
  padding: 4px 8px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font-size: 14px;
  color: var(--color-text);
  background: var(--color-surface);
}
</style>
