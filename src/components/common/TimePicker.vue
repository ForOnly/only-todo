<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = withDefaults(
  defineProps<{
    hour: number;
    minute: number;
    second: number;
    precision?: "minute" | "second";
    size?: "default" | "compact";
  }>(),
  {
    precision: "minute",
    size: "default",
  },
);

const emit = defineEmits<{
  "update:hour": [value: number];
  "update:minute": [value: number];
  "update:second": [value: number];
}>();

useI18n();

const hourText = computed(() => String(props.hour).padStart(2, "0"));
const minuteText = computed(() => String(props.minute).padStart(2, "0"));
const secondText = computed(() => String(props.second).padStart(2, "0"));

function clamp(v: number, max: number): number {
  if (!Number.isFinite(v)) return 0;
  return Math.min(max, Math.max(0, Math.round(v)));
}

function onInput(event: Event) {
  const raw = (event.target as HTMLInputElement).value.replace(/\D/g, "").slice(0, 2);
  (event.target as HTMLInputElement).value = raw;
}

function commitHour(event: Event) {
  const raw = (event.target as HTMLInputElement).value;
  const val = raw.trim() === "" ? 0 : Number(raw);
  emit("update:hour", clamp(val, 23));
}

function commitMinute(event: Event) {
  const raw = (event.target as HTMLInputElement).value;
  const val = raw.trim() === "" ? 0 : Number(raw);
  emit("update:minute", clamp(val, 59));
}

function commitSecond(event: Event) {
  const raw = (event.target as HTMLInputElement).value;
  const val = raw.trim() === "" ? 0 : Number(raw);
  emit("update:second", clamp(val, 59));
}
</script>

<template>
  <div class="time-row" :class="{ compact: size === 'compact' }">
    <label>
      <span>{{ $t("datetime.hour") }}</span>
      <input
        :value="hourText"
        type="text"
        inputmode="numeric"
        maxlength="2"
        autocomplete="off"
        @input="onInput"
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
        @input="onInput"
        @change="commitMinute"
        @blur="commitMinute"
      />
    </label>
    <label v-if="precision === 'second'">
      <span>{{ $t("datetime.second") }}</span>
      <input
        :value="secondText"
        type="text"
        inputmode="numeric"
        maxlength="2"
        autocomplete="off"
        @input="onInput"
        @change="commitSecond"
        @blur="commitSecond"
      />
    </label>
  </div>
</template>

<style scoped>
.time-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.time-row.compact {
  gap: 8px;
}

.time-row label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
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
  text-align: center;
  width: 100%;
}
</style>
