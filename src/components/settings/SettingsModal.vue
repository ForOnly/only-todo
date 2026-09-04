<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getVersion } from "@tauri-apps/api/app";

import AppButton from "@/components/common/AppButton.vue";
import AppModal from "@/components/common/AppModal.vue";
import AppSelect from "@/components/common/AppSelect.vue";
import type {
  FloatDefaultMode,
  SortBy,
  SortOrder,
  UiLocale,
  UiTheme,
  UpdateSettingsDto,
  WorkbenchView,
} from "@/api/types";
import { DEFAULT_VIEW_OPTIONS, coerceDefaultView } from "@/constants/workbenchViews";
import { confirm } from "@/composables/useAppConfirm";
import { parseListDefaultSort, serializeListDefaultSort } from "@/utils/listSort";

const props = defineProps<{
  open: boolean;
  notificationEnabled: boolean;
  listDefaultSort: string;
  listDefaultView: WorkbenchView;
  floatAlwaysOnTop: boolean;
  floatVisibleCount: number;
  floatAutoShow: boolean;
  floatDefaultMode: FloatDefaultMode;
  floatHoverPreview: boolean;
  autostartEnabled: boolean;
  uiTheme: UiTheme;
  uiLocale: UiLocale;
  saving?: boolean;
  checkingUpdate?: boolean;
}>();

const emit = defineEmits<{
  close: [];
  update: [payload: UpdateSettingsDto];
  "check-update": [];
}>();

const { t } = useI18n();

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
const localDefaultView = ref<WorkbenchView>(coerceDefaultView(props.listDefaultView));
const closing = ref(false);
const baseline = ref("");
const appVersion = ref("");

type SettingsDraft = {
  notificationEnabled: boolean;
  floatAlwaysOnTop: boolean;
  floatVisibleCount: number;
  floatAutoShow: boolean;
  floatDefaultMode: FloatDefaultMode;
  floatHoverPreview: boolean;
  autostartEnabled: boolean;
  uiTheme: UiTheme;
  uiLocale: UiLocale;
  sortBy: SortBy;
  sortOrder: SortOrder;
  listDefaultView: WorkbenchView;
};

function captureDraft(): SettingsDraft {
  return {
    notificationEnabled: localNotification.value,
    floatAlwaysOnTop: localAlwaysOnTop.value,
    floatVisibleCount: localVisibleCount.value,
    floatAutoShow: localAutoShow.value,
    floatDefaultMode: localDefaultMode.value,
    floatHoverPreview: localHoverPreview.value,
    autostartEnabled: localAutostart.value,
    uiTheme: localTheme.value,
    uiLocale: localLocale.value,
    sortBy: localSortBy.value,
    sortOrder: localSortOrder.value,
    listDefaultView: localDefaultView.value,
  };
}

const dirty = computed(() => JSON.stringify(captureDraft()) !== baseline.value);

const themeOptions = computed(() => [
  { value: "system", label: t("settings.themeSystem") },
  { value: "light", label: t("settings.themeLight") },
  { value: "dark", label: t("settings.themeDark") },
]);
const localeOptions = computed(() => [
  { value: "zh-CN", label: t("settings.langZh") },
  { value: "en-US", label: t("settings.langEn") },
]);
const viewOptions = computed(() =>
  DEFAULT_VIEW_OPTIONS.map((id) => ({
    value: id,
    label: t(`views.${id}`),
  })),
);
const sortByOptions = computed(() => [
  { value: "priority", label: t("sort.priority") },
  { value: "dueDate", label: t("sort.dueDate") },
  { value: "updatedAt", label: t("sort.updatedAt") },
  { value: "createdAt", label: t("sort.createdAt") },
  { value: "title", label: t("sort.title") },
]);
const sortOrderOptions = computed(() => [
  { value: "desc", label: t("settings.desc") },
  { value: "asc", label: t("settings.asc") },
]);
const shapeOptions = computed(() => [
  { value: "ball", label: t("settings.ball") },
  { value: "panel", label: t("settings.panel") },
]);

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
      localDefaultView.value = coerceDefaultView(props.listDefaultView);
      const sort = parseListDefaultSort(props.listDefaultSort);
      localSortBy.value = sort.sortBy;
      localSortOrder.value = sort.sortOrder;
      closing.value = false;
      baseline.value = JSON.stringify(captureDraft());
      void loadAppVersion();
    }
  },
);

async function loadAppVersion() {
  try {
    appVersion.value = await getVersion();
  } catch {
    appVersion.value = "";
  }
}

function checkUpdate() {
  emit("check-update");
}

function save() {
  emit("update", {
    notificationEnabled: localNotification.value,
    listDefaultSort: serializeListDefaultSort({
      sortBy: localSortBy.value,
      sortOrder: localSortOrder.value,
    }),
    listDefaultView: localDefaultView.value,
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

/** Esc / × / 取消：有未保存改动则先确认 */
async function requestClose() {
  if (closing.value || props.saving) return;
  if (dirty.value) {
    closing.value = true;
    const ok = await confirm({
      title: t("common.discardTitle"),
      message: t("settings.discardMessage"),
      confirmLabel: t("common.discard"),
      danger: true,
    });
    closing.value = false;
    if (!ok) return;
  }
  emit("close");
}
</script>

<template>
  <AppModal
    :open="open"
    :title="$t('settings.title')"
    :close-on-backdrop="false"
    @close="requestClose"
  >
    <section class="group">
      <h3>{{ $t("settings.appearance") }}</h3>
      <label class="setting-row">
        <span>{{ $t("settings.theme") }}</span>
        <AppSelect v-model="localTheme" class="select-grow" :options="themeOptions" />
      </label>
      <label class="setting-row">
        <span>{{ $t("settings.language") }}</span>
        <AppSelect v-model="localLocale" class="select-grow" :options="localeOptions" />
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
        <span>{{ $t("settings.defaultView") }}</span>
        <AppSelect v-model="localDefaultView" class="select-grow" :options="viewOptions" />
      </label>
      <p class="hint">{{ $t("settings.defaultViewHint") }}</p>
      <label class="setting-row">
        <span>{{ $t("settings.defaultSort") }}</span>
        <AppSelect v-model="localSortBy" class="select-grow" :options="sortByOptions" />
      </label>
      <label class="setting-row">
        <span>{{ $t("settings.direction") }}</span>
        <AppSelect v-model="localSortOrder" class="select-grow" :options="sortOrderOptions" />
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
          <AppSelect v-model="localDefaultMode" class="select-grow" :options="shapeOptions" />
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
      <div class="setting-row about-row">
        <span>{{ $t("settings.version") }}</span>
        <span class="version-value">{{ appVersion || "—" }}</span>
      </div>
      <div class="setting-row">
        <span>{{ $t("settings.updates") }}</span>
        <AppButton variant="default" :disabled="checkingUpdate" @click="checkUpdate">
          {{ checkingUpdate ? $t("updater.checking") : $t("settings.checkUpdate") }}
        </AppButton>
      </div>
    </section>

    <div class="app-modal-actions">
      <AppButton variant="ghost" :disabled="saving" @click="requestClose">
        {{ $t("common.cancel") }}
      </AppButton>
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

.select-grow {
  flex: 1;
  min-width: 140px;
  max-width: 240px;
}

.hint {
  margin: 0 0 4px;
  padding-left: 0;
  font-size: 12px;
  color: var(--color-muted);
  line-height: 1.4;
}

.number-input {
  width: 64px;
  padding: 4px 8px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  background: var(--color-surface);
}

.about-row {
  cursor: default;
}

.version-value {
  font-variant-numeric: tabular-nums;
  color: var(--color-muted);
}
</style>
