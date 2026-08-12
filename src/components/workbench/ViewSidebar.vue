<script setup lang="ts">
import type { WorkbenchView } from "@/api/types";

const props = defineProps<{
  view: WorkbenchView;
  activeTag: string | null;
  allTags: string[];
  overdueCount?: number;
}>();

const emit = defineEmits<{
  selectView: [view: WorkbenchView, tag?: string | null];
}>();

const primaryViews: { id: WorkbenchView; label: string }[] = [
  { id: "today", label: "今日" },
  { id: "overdue", label: "逾期" },
  { id: "doing", label: "进行中" },
  { id: "all", label: "全部" },
  { id: "done", label: "已完成" },
  { id: "archived", label: "归档" },
  { id: "trash", label: "回收站" },
];

function isActive(id: WorkbenchView): boolean {
  return props.view === id;
}

function isTagActive(tag: string): boolean {
  return props.view === "tag" && props.activeTag === tag;
}
</script>

<template>
  <aside class="view-sidebar" aria-label="视图">
    <nav class="nav">
      <button
        v-for="item in primaryViews"
        :key="item.id"
        type="button"
        class="nav-item"
        :class="{ active: isActive(item.id) }"
        @click="emit('selectView', item.id)"
      >
        <span>{{ item.label }}</span>
        <span
          v-if="item.id === 'overdue' && (overdueCount ?? 0) > 0"
          class="badge"
          :title="`${overdueCount} 条逾期`"
        >
          {{ overdueCount }}
        </span>
      </button>
    </nav>

    <div v-if="allTags.length" class="tags-block">
      <h3 class="tags-title">标签</h3>
      <button
        v-for="tag in allTags"
        :key="tag"
        type="button"
        class="nav-item tag"
        :class="{ active: isTagActive(tag) }"
        @click="emit('selectView', 'tag', tag)"
      >
        {{ tag }}
      </button>
    </div>
  </aside>
</template>

<style scoped>
.view-sidebar {
  width: 168px;
  flex-shrink: 0;
  border-right: 1px solid #e2e8f0;
  background: #f8fafc;
  padding: 12px 8px;
  overflow-y: auto;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #334155;
  font: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.nav-item:hover {
  background: #e2e8f0;
}

.nav-item.active {
  background: #0f172a;
  color: #f8fafc;
  font-weight: 600;
}

.badge {
  min-width: 18px;
  padding: 1px 6px;
  border-radius: 999px;
  background: #dc2626;
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  text-align: center;
}

.nav-item.active .badge {
  background: #fca5a5;
  color: #7f1d1d;
}

.tags-block {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #e2e8f0;
}

.tags-title {
  margin: 0 0 6px 10px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #94a3b8;
}

.nav-item.tag {
  color: #475569;
}
</style>
