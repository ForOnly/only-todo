<script setup lang="ts">
const props = defineProps<{
  label: string;
  /** Inspector 可删标签（展示 × ） */
  removable?: boolean;
  /** 列表 +N 更多变体（不展示 # 前缀） */
  more?: boolean;
  /** 作为可点击按钮用于“标签建议” */
  clickable?: boolean;
}>();

const emit = defineEmits<{
  remove: [label: string];
  click: [label: string];
}>();

function onRemove() {
  if (!props.removable) return;
  emit("remove", props.label);
}

function onClick() {
  if (!props.clickable) return;
  emit("click", props.label);
}
</script>

<template>
  <button
    v-if="clickable"
    type="button"
    class="tag-chip tag-chip--button"
    @click="onClick"
    :title="label"
  >
    <span v-if="!more" class="tag-prefix">#</span>
    <span class="tag-label">{{ label }}</span>
  </button>

  <span v-else class="tag-chip" :class="{ 'tag-chip--more': more }" :title="label">
    <span v-if="!more" class="tag-prefix">#</span>
    <span class="tag-label">{{ label }}</span>
    <button
      v-if="removable"
      type="button"
      class="tag-x"
      aria-label="remove tag"
      @click.stop="onRemove"
    >
      ×
    </button>
  </span>
</template>

<style scoped>
.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--color-tag-border);
  background: var(--color-tag-bg);
  color: var(--color-tag-text);
  font-size: 11px;
  font-weight: 500;
  line-height: 1.4;

  max-width: 100%;
}

.tag-chip--button {
  cursor: pointer;
  border-radius: var(--radius-pill);
  background: var(--color-surface);
  color: var(--color-text-secondary);
  transition: background var(--transition-fast);
}

.tag-chip--button:hover {
  background: var(--color-surface-muted);
}

.tag-chip--more {
  background: transparent;
  border-color: transparent;
  color: var(--color-muted);
  padding: 2px 6px;
}

.tag-prefix {
  color: var(--color-tag-prefix);
}

.tag-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-x {
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--color-muted);
  padding: 0;
  font-size: 14px;
  line-height: 1;
}

.tag-x:hover {
  color: var(--color-accent);
}
</style>
