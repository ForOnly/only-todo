<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppSelect from "@/components/common/AppSelect.vue";

const props = defineProps<{
  viewYear: number;
  viewMonth: number;
  size?: "default" | "compact";
}>();

const emit = defineEmits<{
  "update:viewYear": [value: number];
  "update:viewMonth": [value: number];
}>();

const { locale } = useI18n();

const yearOptions = computed(() => {
  const years: { value: string; label: string }[] = [];
  for (let y = 1900; y <= 2100; y++) years.push({ value: String(y), label: String(y) });
  return years;
});

const yearModel = computed({
  get: () => String(props.viewYear),
  set: (v: string) => emit("update:viewYear", Number(v)),
});

const monthOptions = computed(() => {
  return Array.from({ length: 12 }, (_, i) => {
    const d = new Date(2000, i, 1);
    return {
      value: String(i),
      label: d.toLocaleString(locale.value, { month: "long" }),
    };
  });
});

const monthModel = computed({
  get: () => String(props.viewMonth),
  set: (v: string) => emit("update:viewMonth", Number(v)),
});

function prevYear() {
  emit("update:viewYear", props.viewYear - 1);
}

function nextYear() {
  emit("update:viewYear", props.viewYear + 1);
}

function prevMonth() {
  if (props.viewMonth === 0) {
    emit("update:viewMonth", 11);
    emit("update:viewYear", props.viewYear - 1);
  } else {
    emit("update:viewMonth", props.viewMonth - 1);
  }
}

function nextMonth() {
  if (props.viewMonth === 11) {
    emit("update:viewMonth", 0);
    emit("update:viewYear", props.viewYear + 1);
  } else {
    emit("update:viewMonth", props.viewMonth + 1);
  }
}
</script>

<template>
  <div class="cal-nav">
    <div class="nav-row">
      <AppSelect
        v-model="yearModel"
        :options="yearOptions"
        :compact="size === 'compact'"
        :teleport="true"
      />
      <button type="button" class="nav-btn" :aria-label="$t('datetime.prevYear')" @click="prevYear">
        ‹
      </button>
      <button type="button" class="nav-btn" :aria-label="$t('datetime.nextYear')" @click="nextYear">
        ›
      </button>
    </div>
    <div class="nav-row">
      <AppSelect
        v-model="monthModel"
        :options="monthOptions"
        :compact="size === 'compact'"
        :teleport="true"
      />
      <button
        type="button"
        class="nav-btn"
        :aria-label="$t('datetime.prevMonth')"
        @click="prevMonth"
      >
        ‹
      </button>
      <button
        type="button"
        class="nav-btn"
        :aria-label="$t('datetime.nextMonth')"
        @click="nextMonth"
      >
        ›
      </button>
    </div>
  </div>
</template>

<style scoped>
.cal-nav {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.nav-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.nav-row :deep(.trigger) {
  padding: 4px 6px;
  font-size: 12px;
  min-width: 0;
  flex: 1;
}

.nav-btn {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background var(--transition-fast);
}

.nav-btn:hover {
  background: var(--color-bg-accent);
}
</style>
