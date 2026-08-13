<script setup lang="ts">
import AppConfirmDialog from "@/components/common/AppConfirmDialog.vue";
import AppContextMenu from "@/components/common/AppContextMenu.vue";

withDefaults(
  defineProps<{
    /** 主窗挂载确认框；助理窗仅右键菜单 */
    confirm?: boolean;
  }>(),
  { confirm: false },
);
</script>

<template>
  <AppConfirmDialog v-if="confirm" />
  <AppContextMenu />
</template>
