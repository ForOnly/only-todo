<script setup lang="ts">
import AppButton from "@/components/common/AppButton.vue";

defineProps<{
  keyword: string;
  filterCollapsed: boolean;
  filterPinned: boolean;
}>();

const emit = defineEmits<{
  "update:keyword": [value: string];
  search: [];
  create: [];
  settings: [];
  toggleFilter: [];
  toggleFilterPin: [];
}>();
</script>

<template>
  <header class="header">
    <div class="brand">Only Todo</div>
    <div class="actions">
      <AppButton
        :variant="filterCollapsed ? 'ghost' : 'default'"
        :disabled="filterPinned"
        :title="filterPinned ? '已固定筛选栏' : '展开/收起筛选'"
        @click="emit('toggleFilter')"
      >
        筛选
      </AppButton>
      <AppButton
        :variant="filterPinned ? 'primary' : 'ghost'"
        title="固定筛选栏展开"
        @click="emit('toggleFilterPin')"
      >
        {{ filterPinned ? "已固定" : "固定" }}
      </AppButton>
      <input
        class="search"
        type="search"
        placeholder="搜索任务..."
        :value="keyword"
        @input="emit('update:keyword', ($event.target as HTMLInputElement).value)"
        @keydown.enter="emit('search')"
      />
      <AppButton variant="primary" @click="emit('search')">搜索</AppButton>
      <AppButton @click="emit('create')">+ 新建</AppButton>
      <AppButton variant="ghost" @click="emit('settings')">设置</AppButton>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  background: #fff;
}

.brand {
  font-weight: 700;
  font-size: 18px;
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
  width: 200px;
  padding: 8px 10px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font: inherit;
}

.search:focus {
  outline: none;
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
}
</style>
