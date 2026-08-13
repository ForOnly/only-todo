<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import {
  closeContextMenu,
  runContextAction,
  useAppContextMenuState,
  type ContextMenuAction,
} from "@/composables/useAppContextMenu";

const { t } = useI18n();
const state = useAppContextMenuState();
const menuRef = ref<HTMLElement | null>(null);
const style = ref<Record<string, string>>({});

const labels: Record<ContextMenuAction, string> = {
  cut: "contextMenu.cut",
  copy: "contextMenu.copy",
  paste: "contextMenu.paste",
  selectAll: "contextMenu.selectAll",
};

const visibleItems = computed(() => state.items);

async function place() {
  await nextTick();
  const el = menuRef.value;
  if (!el) return;
  const w = el.offsetWidth;
  const h = el.offsetHeight;
  let x = state.x;
  let y = state.y;
  if (x + w > window.innerWidth - 4) x = Math.max(4, window.innerWidth - w - 4);
  if (y + h > window.innerHeight - 4) y = Math.max(4, window.innerHeight - h - 4);
  style.value = {
    position: "fixed",
    left: `${x}px`,
    top: `${y}px`,
    zIndex: "1200",
  };
}

function onDocPointer(event: PointerEvent) {
  if (!state.open) return;
  if (menuRef.value?.contains(event.target as Node)) return;
  closeContextMenu();
}

function onKey(event: KeyboardEvent) {
  if (event.key === "Escape" && state.open) {
    closeContextMenu();
  }
}

watch(
  () => state.open,
  (v) => {
    if (v) void place();
  },
);

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointer, true);
  window.addEventListener("keydown", onKey);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointer, true);
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <Teleport to="body">
    <ul v-if="state.open" ref="menuRef" class="ctx-menu" role="menu" :style="style">
      <li v-for="item in visibleItems" :key="item.id" role="none">
        <button
          type="button"
          class="ctx-item"
          role="menuitem"
          :disabled="!item.enabled"
          @click="runContextAction(item.id)"
        >
          {{ t(labels[item.id]) }}
        </button>
      </li>
    </ul>
  </Teleport>
</template>

<style scoped>
.ctx-menu {
  margin: 0;
  padding: 4px;
  list-style: none;
  min-width: 140px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  box-shadow: var(--shadow-md);
}

.ctx-item {
  display: block;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text);
  font: inherit;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.ctx-item:hover:not(:disabled) {
  background: var(--color-bg-accent);
}

.ctx-item:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
