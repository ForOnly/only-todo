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
  border-radius: 6px;
  background: #fef2f2;
  color: #dc2626;
  font-size: 14px;
  border: 1px solid #fecaca;
}
</style>
