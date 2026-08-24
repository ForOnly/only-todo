<script setup lang="ts">
import { computed } from "vue";

import type { DayCell } from "./calendarTypes";
import ActionBar from "./ActionBar.vue";
import CalendarDays from "./CalendarDays.vue";
import CalendarNav from "./CalendarNav.vue";
import CalendarWeekdays from "./CalendarWeekdays.vue";
import TimePicker from "./TimePicker.vue";

const props = withDefaults(
  defineProps<{
    viewYear: number;
    viewMonth: number;
    hour: number;
    minute: number;
    second: number;
    selected: string;
    mode: "date" | "datetime";
    precision: "minute" | "second";
    firstDayOfWeek: 0 | 1;
    minDate: Date | null;
    maxDate: Date | null;
    clearable: boolean;
    disabledDate?: (d: Date) => boolean;
    size?: "default" | "compact";
  }>(),
  {
    size: "default",
  },
);

const emit = defineEmits<{
  "update:viewYear": [value: number];
  "update:viewMonth": [value: number];
  "update:hour": [value: number];
  "update:minute": [value: number];
  "update:second": [value: number];
  select: [value: string];
  clear: [];
  today: [];
  confirm: [];
}>();


// ── 今日 ──
const today = computed(() => {
  const d = new Date();
  return { year: d.getFullYear(), month: d.getMonth(), day: d.getDate() };
});

// ── 日期解析 ──
function parseValue(v: string): Date | null {
  if (!v.trim()) return null;
  const d = new Date(v);
  return Number.isNaN(d.getTime()) ? null : d;
}

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

function toValue(y: number, m: number, day: number, h: number, min: number, sec: number): string {
  return `${y}-${pad(m + 1)}-${pad(day)}T${pad(h)}:${pad(min)}:${pad(sec)}`;
}

// ── 日期禁用 ──
function isDayDisabled(year: number, month: number, day: number): boolean {
  const d = new Date(year, month, day);
  if (props.minDate && d < props.minDate) return true;
  if (props.maxDate && d > props.maxDate) return true;
  return props.disabledDate?.(d) ?? false;
}

// ── 日历网格 ──
const dayCells = computed((): DayCell[] => {
  const first = new Date(props.viewYear, props.viewMonth, 1);
  const startPad = (first.getDay() - props.firstDayOfWeek + 7) % 7;
  const daysInMonth = new Date(props.viewYear, props.viewMonth + 1, 0).getDate();
  const prevDays = new Date(props.viewYear, props.viewMonth, 0).getDate();
  const cells: DayCell[] = [];
  for (let i = 0; i < startPad; i++) {
    cells.push({
      day: prevDays - startPad + i + 1,
      inMonth: false,
      key: `p-${i}`,
      disabled: true,
    });
  }
  for (let day = 1; day <= daysInMonth; day++) {
    cells.push({
      day,
      inMonth: true,
      key: `c-${day}`,
      disabled: isDayDisabled(props.viewYear, props.viewMonth, day),
    });
  }
  while (cells.length % 7 !== 0) {
    const day = cells.length - startPad - daysInMonth + 1;
    cells.push({ day, inMonth: false, key: `n-${day}`, disabled: true });
  }
  return cells;
});

const selectedDay = computed(() => {
  const d = parseValue(props.selected);
  if (!d) return null;
  if (d.getFullYear() !== props.viewYear || d.getMonth() !== props.viewMonth) return null;
  return d.getDate();
});

// ── 事件 ──
function onSelectDay(day: number) {
  emit(
    "select",
    toValue(props.viewYear, props.viewMonth, day, props.hour, props.minute, props.second),
  );
}

function onClear() {
  emit("clear");
}

function onToday() {
  emit("today");
}

function onConfirm() {
  emit("confirm");
}
</script>

<template>
  <div class="cal-panel">
    <CalendarNav
      :view-year="viewYear"
      :view-month="viewMonth"
      :size="size"
      @update:view-year="emit('update:viewYear', $event)"
      @update:view-month="emit('update:viewMonth', $event)"
    />
    <CalendarWeekdays :first-day-of-week="firstDayOfWeek" />
    <CalendarDays
      :day-cells="dayCells"
      :selected-day="selectedDay"
      :today="today"
      :view-year="viewYear"
      :view-month="viewMonth"
      @select="onSelectDay"
    />
    <TimePicker
      v-if="mode === 'datetime'"
      :hour="hour"
      :minute="minute"
      :second="second"
      :precision="precision"
      :size="size"
      @update:hour="emit('update:hour', $event)"
      @update:minute="emit('update:minute', $event)"
      @update:second="emit('update:second', $event)"
    />
    <ActionBar
      :clearable="clearable"
      @today="onToday"
      @clear="onClear"
      @confirm="onConfirm"
    />
  </div>
</template>

<style scoped>
.cal-panel {
  display: flex;
  flex-direction: column;
}
</style>