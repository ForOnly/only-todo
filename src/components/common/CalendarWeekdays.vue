<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  firstDayOfWeek: 0 | 1;
}>();

const { locale } = useI18n();

const weekdays = computed(() => {
  const start = new Date(2024, 0, 7 + props.firstDayOfWeek);
  return Array.from({ length: 7 }, (_, i) => {
    const d = new Date(start);
    d.setDate(start.getDate() + i);
    return d.toLocaleDateString(locale.value, { weekday: "short" });
  });
});
</script>

<template>
  <div class="weekdays">
    <span v-for="w in weekdays" :key="w">{{ w }}</span>
  </div>
</template>

<style scoped>
.weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
  margin-bottom: 4px;
  font-size: 11px;
  color: var(--color-muted);
  text-align: center;
}
</style>