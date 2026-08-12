<script setup lang="ts">
import AppButton from "@/components/common/AppButton.vue";

defineProps<{
  keyword: string;
}>();

const emit = defineEmits<{
  "update:keyword": [value: string];
  search: [];
  create: [];
  settings: [];
  openCompanion: [];
}>();
</script>

<template>
  <header class="header">
    <div class="brand">Only Todo</div>
    <div class="actions">
      <input
        id="workbench-search"
        class="search"
        type="search"
        placeholder="搜索任务…  /"
        :value="keyword"
        @input="emit('update:keyword', ($event.target as HTMLInputElement).value)"
        @keydown.enter="emit('search')"
      />
      <AppButton variant="primary" @click="emit('create')">+ 新建</AppButton>
      <AppButton variant="ghost" @click="emit('openCompanion')">打开伴侣</AppButton>
      <AppButton variant="ghost" @click="emit('settings')">设置</AppButton>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 16px;
  border-bottom: 1px solid #e2e8f0;
  background: linear-gradient(180deg, #ffffff 0%, #f8fafc 100%);
}

.brand {
  font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  font-weight: 750;
  font-size: 18px;
  letter-spacing: -0.02em;
  color: #0f172a;
  flex-shrink: 0;
}

.actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.search {
  width: min(280px, 40vw);
  padding: 8px 10px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font: inherit;
  background: #fff;
}

.search:focus {
  outline: none;
  border-color: #0f172a;
  box-shadow: 0 0 0 3px rgba(15, 23, 42, 0.08);
}
</style>
