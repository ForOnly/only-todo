<script setup lang="ts">
import AppConfirmDialog from "@/components/common/AppConfirmDialog.vue";
import AppContextMenu from "@/components/common/AppContextMenu.vue";
import AppMessageHost from "@/components/common/AppMessageHost.vue";

withDefaults(
  defineProps<{
    /** 主窗挂载确认框；助理窗仅右键菜单 + Message */
    confirm?: boolean;
  }>(),
  { confirm: false },
);
</script>

<template>
  <AppMessageHost />
  <AppConfirmDialog v-if="confirm" />
  <AppContextMenu />
</template>
