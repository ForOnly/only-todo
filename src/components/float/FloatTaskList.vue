<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";

import FloatDetailPanel from "@/components/float/FloatDetailPanel.vue";
import type { TodoDto } from "@/api/types";
import { formatDueDate, isOverdue } from "@/utils/date";

const props = defineProps<{
  todos: TodoDto[];
  selectedId: string | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  openMain: [id: string];
  complete: [id: string];
  closePeek: [];
}>();

/** 与 .peek-wrap 过渡对齐，关闭后延迟卸载以播完收拢动画 */
const PEEK_CLOSE_MS = 200;
const mountedPeekId = ref<string | null>(props.selectedId);
let closeTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.selectedId,
  (id, prev) => {
    if (closeTimer) {
      clearTimeout(closeTimer);
      closeTimer = null;
    }
    if (id) {
      mountedPeekId.value = id;
      return;
    }
    const closing = prev ?? mountedPeekId.value;
    if (!closing) {
      mountedPeekId.value = null;
      return;
    }
    mountedPeekId.value = closing;
    closeTimer = setTimeout(() => {
      if (props.selectedId === null) {
        mountedPeekId.value = null;
      }
      closeTimer = null;
    }, PEEK_CLOSE_MS);
  },
);

onUnmounted(() => {
  if (closeTimer) {
    clearTimeout(closeTimer);
    closeTimer = null;
  }
});

const showLoadingHint = computed(() => props.loading && props.todos.length === 0);
const isRefreshing = computed(() => props.loading && props.todos.length > 0);
</script>

<template>
  <div class="task-list">
    <div v-if="showLoadingHint" class="hint">{{ $t("common.loading") }}</div>
    <ul v-else :class="{ refreshing: isRefreshing }">
      <li
        v-for="todo in todos"
        :key="todo.id"
        class="item"
        :class="{
          selected: selectedId === todo.id,
          overdue: isOverdue(todo.dueDate, todo.status),
        }"
        :data-priority="todo.priority"
      >
        <div class="row" @click="emit('select', todo.id)">
          <button
            type="button"
            class="check"
            :title="$t('companion.markComplete')"
            :aria-label="$t('companion.markComplete')"
            @click.stop="emit('complete', todo.id)"
          />
          <div class="content">
            <span class="name">{{ todo.title }}</span>
            <span class="meta">
              {{ $t(`status.${todo.status}`) }}
              <template v-if="todo.dueDate">
                ·
                <span :class="{ 'due-overdue': isOverdue(todo.dueDate, todo.status) }">
                  {{ formatDueDate(todo.dueDate) }}
                </span>
              </template>
            </span>
          </div>
        </div>

        <div class="peek-wrap" :class="{ open: selectedId === todo.id }">
          <div class="peek-inner">
            <FloatDetailPanel
              v-if="mountedPeekId === todo.id"
              :detail="todo"
              @close="emit('closePeek')"
              @open-main="emit('openMain', todo.id)"
            />
          </div>
        </div>
      </li>
      <li v-if="todos.length === 0" class="hint empty">
        <p>{{ $t("companion.emptyList") }}</p>
        <p class="caption">{{ $t("companion.editHint") }}</p>
      </li>
    </ul>
    <p v-if="todos.length > 0 && !selectedId" class="list-caption">
      {{ $t("companion.editHint") }}
    </p>
  </div>
</template>

<style scoped>
.task-list {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

ul {
  list-style: none;
  margin: 0;
  padding: 4px 8px 0;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  transition: opacity 0.18s ease;
}

ul.refreshing {
  opacity: 0.7;
}

.item {
  border-radius: var(--radius-md);
  transition: background 0.18s ease;
}

.item:hover {
  background: color-mix(in srgb, var(--color-bg-accent) 70%, transparent);
}

.item.selected {
  background: var(--color-accent-soft);
}

.row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px 10px;
  cursor: pointer;
  border-left: 3px solid var(--color-priority-low);
  border-radius: var(--radius-md);
}

.item[data-priority="Urgent"] .row {
  border-left-color: var(--color-priority-urgent);
}

.item[data-priority="High"] .row {
  border-left-color: var(--color-priority-high);
}

.item[data-priority="Medium"] .row {
  border-left-color: var(--color-priority-medium);
}

.item[data-priority="Low"] .row {
  border-left-color: var(--color-priority-low);
}

.check {
  width: 18px;
  height: 18px;
  margin-top: 1px;
  flex-shrink: 0;
  border: 1.5px solid var(--color-border-strong);
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  padding: 0;
  position: relative;
  transition:
    border-color 0.18s ease,
    background 0.18s ease,
    transform 0.18s ease;
}

.check:hover {
  border-color: var(--color-success);
  background: color-mix(in srgb, var(--color-success) 16%, transparent);
  transform: scale(1.06);
}

.check:hover::after {
  content: "";
  position: absolute;
  inset: 4px;
  border-radius: 50%;
  background: var(--color-success);
  opacity: 0.85;
}

.content {
  min-width: 0;
  flex: 1;
}

.name {
  display: block;
  font-size: 13px;
  font-weight: 500;
  line-height: 1.35;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text);
}

.meta {
  display: block;
  font-size: 11px;
  color: var(--color-muted);
  margin-top: 2px;
}

.due-overdue,
.item.overdue .name {
  color: var(--color-overdue);
}

.peek-wrap {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.2s ease;
}

.peek-wrap.open {
  grid-template-rows: 1fr;
}

.peek-inner {
  overflow: hidden;
  min-height: 0;
  opacity: 0;
  transition: opacity 0.18s ease;
}

.peek-wrap.open .peek-inner {
  opacity: 1;
}

.hint {
  padding: 28px 16px;
  color: var(--color-muted);
  font-size: 13px;
  text-align: center;
  list-style: none;
}

.hint.empty p {
  margin: 0;
}

.hint .caption,
.list-caption {
  margin: 8px 0 0;
  font-size: 11px;
  color: var(--color-muted);
  opacity: 0.85;
}

.list-caption {
  margin: 0;
  padding: 6px 12px 10px;
  text-align: center;
  flex-shrink: 0;
}
</style>
