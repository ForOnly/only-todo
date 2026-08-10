<script setup lang="ts">
import {
  PRIORITY_LABELS,
  PRIORITY_OPTIONS,
  STATUS_LABELS,
  STATUS_OPTIONS,
  type Priority,
  type TodoStatus,
} from "@/api/types";

defineProps<{
  selectedStatuses: TodoStatus[];
  selectedPriorities: Priority[];
  collapsed: boolean;
}>();

const emit = defineEmits<{
  toggleStatus: [status: TodoStatus];
  togglePriority: [priority: Priority];
}>();
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <div v-show="!collapsed" class="sidebar-content">
      <section>
        <h3>状态</h3>
        <label v-for="status in STATUS_OPTIONS" :key="status" class="filter-item">
          <input
            type="checkbox"
            :checked="selectedStatuses.includes(status)"
            @change="emit('toggleStatus', status)"
          />
          {{ STATUS_LABELS[status] }}
        </label>
      </section>
      <section>
        <h3>优先级</h3>
        <label
          v-for="priority in PRIORITY_OPTIONS"
          :key="priority"
          class="filter-item"
        >
          <input
            type="checkbox"
            :checked="selectedPriorities.includes(priority)"
            @change="emit('togglePriority', priority)"
          />
          {{ PRIORITY_LABELS[priority] }}
        </label>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 140px;
  flex-shrink: 0;
  border-right: 1px solid #e5e7eb;
  background: #fafafa;
  overflow: hidden;
  transition: width 0.2s ease;
}

.sidebar.collapsed {
  width: 0;
  border-right: none;
}

.sidebar-content {
  width: 140px;
  padding: 12px 10px;
  overflow-y: auto;
  height: 100%;
}

section + section {
  margin-top: 16px;
}

h3 {
  margin: 0 0 6px;
  font-size: 12px;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  font-size: 13px;
  cursor: pointer;
}
</style>
