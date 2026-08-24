<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import CalendarPanel from "./CalendarPanel.vue";
import DateTimePanel from "./DateTimePanel.vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    disabled?: boolean;
    clearable?: boolean;
    mode?: "date" | "datetime";
    precision?: "minute" | "second";
    firstDayOfWeek?: 0 | 1;
    min?: string;
    max?: string;
    placeholder?: string;
    disabledDate?: (d: Date) => boolean;
    format?: Intl.DateTimeFormatOptions;
    size?: "default" | "compact";
    teleport?: boolean;
  }>(),
  {
    disabled: false,
    clearable: true,
    mode: "datetime",
    precision: "minute",
    firstDayOfWeek: 0,
    size: "default",
    teleport: true,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t, locale } = useI18n();

const open = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const panelStyle = ref<Record<string, string>>({});

// 本地 buffer：选中后暂存，确认后才 emit
const bufferValue = ref("");
const viewYear = ref(new Date().getFullYear());
const viewMonth = ref(new Date().getMonth());
const hour = ref(0);
const minute = ref(0);
const second = ref(0);

// ── 日期范围解析 ──
function parseMinMax(value: string): Date | null {
  if (!value) return null;
  let d = new Date(value);
  if (!Number.isNaN(d.getTime())) return d;
  d = new Date(value + "T00:00");
  return Number.isNaN(d.getTime()) ? null : d;
}

const minDate = computed(() => (props.min ? parseMinMax(props.min) : null));
const maxDate = computed(() => (props.max ? parseMinMax(props.max) : null));

// ── 展示文本 ──
function parseValue(v: string): Date | null {
  if (!v.trim()) return null;
  const d = new Date(v);
  return Number.isNaN(d.getTime()) ? null : d;
}

const displayText = computed(() => {
  // 展示当前 modelValue（已确认的值），不是 buffer
  const d = parseValue(props.modelValue);
  if (!d) return props.placeholder ?? t("datetime.placeholder");
  if (props.format) {
    return d.toLocaleString(locale.value, props.format);
  }
  return d.toLocaleString(locale.value, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: props.mode === "datetime" ? ("2-digit" as const) : undefined,
    minute: props.mode === "datetime" ? ("2-digit" as const) : undefined,
    second: props.precision === "second" ? ("2-digit" as const) : undefined,
  });
});

// ── 面板定位 ──
function placePanel() {
  const el = triggerRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const panelW = 300;
  const panelH = 380;
  const spaceBelow = window.innerHeight - rect.bottom;
  const openUp = spaceBelow < panelH + 8 && rect.top > spaceBelow;
  let top = openUp ? Math.max(8, rect.top - panelH - 4) : rect.bottom + 4;
  let left = rect.left;
  if (left + panelW > window.innerWidth - 8) {
    left = Math.max(8, window.innerWidth - panelW - 8);
  }
  panelStyle.value = {
    position: "fixed",
    top: `${top}px`,
    left: `${left}px`,
    zIndex: "1100",
  };
}

// ── 打开面板 ──
function openPanel() {
  if (props.disabled) return;
  const d = parseValue(props.modelValue) ?? new Date();
  viewYear.value = d.getFullYear();
  viewMonth.value = d.getMonth();
  hour.value = d.getHours();
  minute.value = d.getMinutes();
  second.value = d.getSeconds();
  bufferValue.value = props.modelValue;
  placePanel();
  open.value = true;
}

// ── 事件 ──
function onSelect(value: string) {
  // 选择日期时只更新 buffer，不 emit
  bufferValue.value = value;
}

function onClear() {
  bufferValue.value = "";
  emit("update:modelValue", "");
  open.value = false;
}

function handlePanelToday() {
  const now = new Date();
  const y = now.getFullYear();
  const m = now.getMonth();
  const d = now.getDate();
  const h = props.mode === "datetime" ? now.getHours() : 0;
  const min = props.mode === "datetime" ? now.getMinutes() : 0;
  const sec = props.precision === "second" && props.mode === "datetime" ? now.getSeconds() : 0;
  const val = `${y}-${String(m + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}T${String(h).padStart(2, "0")}:${String(min).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
  bufferValue.value = val;
  emit("update:modelValue", val);
  open.value = false;
}

function handlePanelConfirm() {
  if (bufferValue.value) {
    emit("update:modelValue", bufferValue.value);
  }
  open.value = false;
}
</script>

<template>
  <div class="app-datetime" :class="{ compact: size === 'compact', disabled }">
    <button
      ref="triggerRef"
      type="button"
      class="trigger"
      :class="{ empty: !modelValue }"
      :disabled="disabled"
      :aria-expanded="open"
      @click="openPanel"
    >
      {{ displayText }}
    </button>
    <DateTimePanel
      :open="open"
      :teleport="teleport"
      :panel-style="panelStyle"
      :trigger-ref="triggerRef"
      @close="open = false"
    >
      <CalendarPanel
        :view-year="viewYear"
        :view-month="viewMonth"
        :hour="hour"
        :minute="minute"
        :second="second"
        :selected="bufferValue"
        :mode="mode"
        :precision="precision"
        :first-day-of-week="firstDayOfWeek"
        :min-date="minDate"
        :max-date="maxDate"
        :clearable="clearable"
        :disabled-date="disabledDate"
        :size="size"
        @update:view-year="viewYear = $event"
        @update:view-month="viewMonth = $event"
        @update:hour="hour = $event"
        @update:minute="minute = $event"
        @update:second="second = $event"
        @select="onSelect"
        @clear="onClear"
        @today="handlePanelToday"
        @confirm="handlePanelConfirm"
      />
    </DateTimePanel>
  </div>
</template>

<style scoped>
.app-datetime {
  width: 100%;
}

.trigger {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font: inherit;
  font-size: 14px;
  text-align: left;
  color: var(--color-text);
  background: var(--color-surface);
  cursor: pointer;
}

.trigger.empty {
  color: var(--color-muted);
}

.compact .trigger {
  padding: 6px 8px;
  font-size: 13px;
}

.trigger:focus-visible {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: var(--focus-ring);
}

.trigger:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>