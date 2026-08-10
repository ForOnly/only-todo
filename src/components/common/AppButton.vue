<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: "default" | "primary" | "ghost" | "danger";
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  {
    variant: "default",
    disabled: false,
    type: "button",
  },
);
</script>

<template>
  <button
    class="app-btn"
    :class="`app-btn--${variant}`"
    :disabled="disabled"
    :type="type"
  >
    <slot />
  </button>
</template>

<style scoped>
@import "../../styles/forms.css";
</style>
