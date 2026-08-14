<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    disabled?: boolean;
    clearable?: boolean;
    compact?: boolean;
    teleport?: boolean;
  }>(),
  {
    disabled: false,
    clearable: true,
    compact: false,
    teleport: true,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t, locale } = useI18n();

const open = ref(false);
const triggerRef = ref<HTMLButtonElement | null>(null);
const panelRef = ref<HTMLElement | null>(null);
const panelStyle = ref<Record<string, string>>({});
const viewYear = ref(new Date().getFullYear());
const viewMonth = ref(new Date().getMonth()); // 0-11
const hour = ref(0);
const minute = ref(0);
/** 时分展示用两位文本，避免原生 number 步进器 */
const hourText = ref("00");
const minuteText = ref("00");

function parseValue(v: string): Date | null {
  if (!v.trim()) return null;
  const d = new Date(v);
  return Number.isNaN(d.getTime()) ? null : d;
}

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

function toValue(y: number, m: number, day: number, h: number, min: number): string {
  return `${y}-${pad(m + 1)}-${pad(day)}T${pad(h)}:${pad(min)}`;
}

const displayText = computed(() => {
  const d = parseValue(props.modelValue);
  if (!d) return t("datetime.placeholder");
  return d.toLocaleString(locale.value, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
});

const monthTitle = computed(() => {
  const d = new Date(viewYear.value, viewMonth.value, 1);
  return d.toLocaleString(locale.value, { year: "numeric", month: "long" });
});

const weekdays = computed(() => {
  // 从已知周日生成周日到周六的短星期文案
  const start = new Date(2024, 0, 7);
  return Array.from({ length: 7 }, (_, i) => {
    const d = new Date(start);
    d.setDate(start.getDate() + i);
    return d.toLocaleDateString(locale.value, { weekday: "short" });
  });
});

type DayCell = { day: number; inMonth: boolean; key: string };

const dayCells = computed((): DayCell[] => {
  const first = new Date(viewYear.value, viewMonth.value, 1);
  const startPad = first.getDay(); // 0 Sun
  const daysInMonth = new Date(viewYear.value, viewMonth.value + 1, 0).getDate();
  const prevDays = new Date(viewYear.value, viewMonth.value, 0).getDate();
  const cells: DayCell[] = [];
  for (let i = 0; i < startPad; i++) {
    const day = prevDays - startPad + i + 1;
    cells.push({
      day,
      inMonth: false,
      key: `p-${day}`,
    });
  }
  for (let day = 1; day <= daysInMonth; day++) {
    cells.push({ day, inMonth: true, key: `c-${day}` });
  }
  while (cells.length % 7 !== 0) {
    const day = cells.length - startPad - daysInMonth + 1;
    cells.push({ day, inMonth: false, key: `n-${day}` });
  }
  return cells;
});

const selectedDay = computed(() => {
  const d = parseValue(props.modelValue);
  if (!d) return null;
  if (d.getFullYear() !== viewYear.value || d.getMonth() !== viewMonth.value) return null;
  return d.getDate();
});

function syncTimeTexts() {
  hourText.value = pad(hour.value);
  minuteText.value = pad(minute.value);
}

function syncFromModel() {
  const d = parseValue(props.modelValue) ?? new Date();
  viewYear.value = d.getFullYear();
  viewMonth.value = d.getMonth();
  hour.value = d.getHours();
  minute.value = d.getMinutes();
  syncTimeTexts();
}

function clampHour(v: number) {
  if (!Number.isFinite(v)) return 0;
  return Math.min(23, Math.max(0, Math.round(v)));
}
function clampMinute(v: number) {
  if (!Number.isFinite(v)) return 0;
  return Math.min(59, Math.max(0, Math.round(v)));
}

function normalizeTimeFields() {
  hour.value = clampHour(hour.value);
  minute.value = clampMinute(minute.value);
  syncTimeTexts();
}

function onHourInput(event: Event) {
  const raw = (event.target as HTMLInputElement).value.replace(/\D/g, "").slice(0, 2);
  hourText.value = raw;
}

function onMinuteInput(event: Event) {
  const raw = (event.target as HTMLInputElement).value.replace(/\D/g, "").slice(0, 2);
  minuteText.value = raw;
}

function commitHour() {
  const parsed = hourText.value.trim() === "" ? Number.NaN : Number(hourText.value);
  hour.value = clampHour(parsed);
  syncTimeTexts();
  applyTime();
}

function commitMinute() {
  const parsed = minuteText.value.trim() === "" ? Number.NaN : Number(minuteText.value);
  minute.value = clampMinute(parsed);
  syncTimeTexts();
  applyTime();
}

function emitCurrent(day: number) {
  normalizeTimeFields();
  emit(
    "update:modelValue",
    toValue(viewYear.value, viewMonth.value, day, hour.value, minute.value),
  );
}

function pickDay(cell: DayCell) {
  if (!cell.inMonth) return;
  emitCurrent(cell.day);
}

function applyTime() {
  normalizeTimeFields();
  const d = parseValue(props.modelValue);
  // 尚未选定日期时只改本地时分，不强制写成当月 1 日
  if (!d) return;
  emit(
    "update:modelValue",
    toValue(d.getFullYear(), d.getMonth(), d.getDate(), hour.value, minute.value),
  );
}

function clear() {
  emit("update:modelValue", "");
  open.value = false;
}

function prevMonth() {
  if (viewMonth.value === 0) {
    viewMonth.value = 11;
    viewYear.value -= 1;
  } else {
    viewMonth.value -= 1;
  }
}

function nextMonth() {
  if (viewMonth.value === 11) {
    viewMonth.value = 0;
    viewYear.value += 1;
  } else {
    viewMonth.value += 1;
  }
}

async function placePanel() {
  await nextTick();
  const el = triggerRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const panelH = panelRef.value?.offsetHeight ?? 320;
  const panelW = panelRef.value?.offsetWidth ?? (props.compact ? 260 : 300);
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

async function toggle() {
  if (props.disabled) return;
  if (open.value) {
    open.value = false;
    return;
  }
  syncFromModel();
  open.value = true;
  await placePanel();
}

function onDocPointer(event: PointerEvent) {
  if (!open.value) return;
  const t = event.target as Node;
  if (triggerRef.value?.contains(t) || panelRef.value?.contains(t)) return;
  open.value = false;
}

function onKeydown(event: KeyboardEvent) {
  if (!open.value) return;
  if (event.key === "Escape") {
    // capture + stopImmediatePropagation：避免父 AppModal 同帧也响应 Esc
    event.preventDefault();
    event.stopImmediatePropagation();
    open.value = false;
  }
}

function onScrollClose(event: Event) {
  if (!open.value) return;
  const t = event.target;
  // 面板自身滚动不关闭
  if (t instanceof Node && panelRef.value?.contains(t)) return;
  open.value = false;
}

function bindOpenListeners() {
  document.addEventListener("scroll", onScrollClose, true);
}

function unbindOpenListeners() {
  document.removeEventListener("scroll", onScrollClose, true);
}

watch(open, (v) => {
  if (v) {
    bindOpenListeners();
    void placePanel();
  } else {
    unbindOpenListeners();
  }
});

watch(
  () => props.modelValue,
  () => {
    if (!open.value) return;
    syncFromModel();
  },
);

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointer, true);
  window.addEventListener("keydown", onKeydown, true);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointer, true);
  window.removeEventListener("keydown", onKeydown, true);
  unbindOpenListeners();
});
</script>

<template>
  <div class="app-datetime" :class="{ compact, disabled }">
    <button
      ref="triggerRef"
      type="button"
      class="trigger"
      :class="{ empty: !modelValue }"
      :disabled="disabled"
      :aria-expanded="open"
      @click="toggle"
    >
      {{ displayText }}
    </button>
    <Teleport to="body" :disabled="teleport === false">
      <div v-if="open" ref="panelRef" class="panel" :style="panelStyle">
        <header class="month-nav">
          <button type="button" class="nav-btn" :aria-label="$t('datetime.prevMonth')" @click="prevMonth">
            ‹
          </button>
          <span class="month-title">{{ monthTitle }}</span>
          <button type="button" class="nav-btn" :aria-label="$t('datetime.nextMonth')" @click="nextMonth">
            ›
          </button>
        </header>
        <div class="weekdays">
          <span v-for="w in weekdays" :key="w">{{ w }}</span>
        </div>
        <div class="days">
          <button
            v-for="cell in dayCells"
            :key="cell.key"
            type="button"
            class="day"
            :class="{
              muted: !cell.inMonth,
              selected: cell.inMonth && cell.day === selectedDay,
            }"
            :disabled="!cell.inMonth"
            @click="pickDay(cell)"
          >
            {{ cell.day }}
          </button>
        </div>
        <div class="time-row">
          <label>
            <span>{{ $t("datetime.hour") }}</span>
            <input
              :value="hourText"
              type="text"
              inputmode="numeric"
              maxlength="2"
              autocomplete="off"
              @input="onHourInput"
              @change="commitHour"
              @blur="commitHour"
            />
          </label>
          <label>
            <span>{{ $t("datetime.minute") }}</span>
            <input
              :value="minuteText"
              type="text"
              inputmode="numeric"
              maxlength="2"
              autocomplete="off"
              @input="onMinuteInput"
              @change="commitMinute"
              @blur="commitMinute"
            />
          </label>
        </div>
        <div class="actions">
          <button v-if="clearable" type="button" class="link-btn" @click="clear">
            {{ $t("datetime.clear") }}
          </button>
          <button type="button" class="link-btn primary" @click="open = false">
            {{ $t("common.confirm") }}
          </button>
        </div>
      </div>
    </Teleport>
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

.panel {
  width: 288px;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  box-shadow: var(--shadow-md);
  color: var(--color-text);
}

.month-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.month-title {
  font-size: 14px;
  font-weight: 600;
}

.nav-btn {
  border: none;
  background: transparent;
  color: var(--color-text);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
}

.nav-btn:hover {
  background: var(--color-bg-accent);
}

.weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
  margin-bottom: 4px;
  font-size: 11px;
  color: var(--color-muted);
  text-align: center;
}

.days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.day {
  aspect-ratio: 1;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
}

.day:hover:not(:disabled) {
  background: var(--color-bg-accent);
}

.day.muted,
.day:disabled {
  color: var(--color-muted);
  opacity: 0.45;
  cursor: default;
}

.day.selected {
  background: var(--color-accent);
  color: var(--color-on-accent);
  font-weight: 600;
}

.time-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.time-row label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  font-size: 12px;
  color: var(--color-muted);
}

.time-row input {
  padding: 6px 8px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font: inherit;
  color: var(--color-text);
  background: var(--color-surface);
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

.link-btn {
  border: none;
  background: transparent;
  color: var(--color-muted);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
}

.link-btn.primary {
  color: var(--color-accent);
  font-weight: 600;
}
</style>
