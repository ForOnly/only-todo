<script setup lang="ts">
import type { DayCell } from "./calendarTypes";

defineProps<{
  dayCells: DayCell[];
  selectedDay: number | null;
  today: { year: number; month: number; day: number };
  viewYear: number;
  viewMonth: number;
}>();

const emit = defineEmits<{
  select: [day: number];
}>();

function pickDay(cell: DayCell) {
  if (!cell.inMonth || cell.disabled) return;
  emit("select", cell.day);
}
</script>

<template>
  <div class="days">
    <button
      v-for="cell in dayCells"
      :key="cell.key"
      type="button"
      class="day"
      :class="{
        muted: !cell.inMonth,
        selected: cell.inMonth && cell.day === selectedDay,
        today:
          cell.inMonth &&
          cell.day === today.day &&
          viewYear === today.year &&
          viewMonth === today.month &&
          cell.day !== selectedDay,
      }"
      :disabled="cell.disabled"
      @click="pickDay(cell)"
    >
      {{ cell.day }}
    </button>
  </div>
</template>

<style scoped>
.days {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.day {
  aspect-ratio: 1;
  border: 1.5px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text);
  font: inherit;
  font-size: 13px;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast);
}

.day:hover:not(:disabled) {
  background: var(--color-bg-accent);
}

.day.selected:hover {
  background: var(--color-accent-hover);
}

.day.today:hover:not(.selected) {
  background: var(--color-accent-soft);
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

.day.today {
  border-color: var(--color-accent);
}
</style>
