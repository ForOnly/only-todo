<script setup lang="ts">
defineProps<{
  message: string;
}>();
</script>

<template>
  <p class="error-banner" role="alert">{{ message }}</p>
</template>

<style scoped>
.error-banner {
  margin: 0 0 12px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  background: var(--color-danger-bg);
  color: var(--color-danger);
  font-size: 14px;
  border: 1px solid var(--color-danger-border);
}
</style>
