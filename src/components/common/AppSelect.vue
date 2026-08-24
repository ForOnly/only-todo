<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";

export type AppSelectOption = {
  value: string;
  label: string;
};

const props = withDefaults(
  defineProps<{
    modelValue: string;
    options: AppSelectOption[];
    disabled?: boolean;
    compact?: boolean;
    /** float 窗设 false，避免 Teleport 打断 hover */
    teleport?: boolean;
    /** 无匹配选项时显示（如下拉动作入口） */
    placeholder?: string;
  }>(),
  {
    disabled: false,
    compact: false,
    teleport: true,
    placeholder: undefined,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const open = ref(false);
const triggerRef = ref<HTMLButtonElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const highlight = ref(0);
const menuStyle = ref<Record<string, string>>({});

const selectedLabel = computed(() => {
  const hit = props.options.find((o) => o.value === props.modelValue);
  if (hit) return hit.label;
  return props.placeholder ?? props.modelValue;
});

function syncHighlight() {
  const idx = props.options.findIndex((o) => o.value === props.modelValue);
  highlight.value = idx >= 0 ? idx : 0;
}

async function placeMenu() {
  await nextTick();
  const el = triggerRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const menuH = menuRef.value?.offsetHeight ?? 200;
  const spaceBelow = window.innerHeight - rect.bottom;
  const openUp = spaceBelow < menuH + 8 && rect.top > spaceBelow;
  const top = openUp ? Math.max(8, rect.top - menuH - 4) : rect.bottom + 4;
  const width = Math.max(rect.width, props.compact ? 120 : 160);
  let left = rect.left;
  if (left + width > window.innerWidth - 8) {
    left = Math.max(8, window.innerWidth - width - 8);
  }
  menuStyle.value = {
    position: "fixed",
    top: `${top}px`,
    left: `${left}px`,
    minWidth: `${width}px`,
    zIndex: "1100",
  };
}

async function toggle() {
  if (props.disabled) return;
  if (open.value) {
    open.value = false;
    return;
  }
  syncHighlight();
  open.value = true;
  await placeMenu();
}

function select(value: string) {
  emit("update:modelValue", value);
  open.value = false;
}

function onDocPointer(event: PointerEvent) {
  if (!open.value) return;
  const t = event.target as Node;
  if (triggerRef.value?.contains(t) || menuRef.value?.contains(t)) return;
  open.value = false;
}

function onKeydown(event: KeyboardEvent) {
  if (!open.value) return;
  if (event.key === "Escape") {
    // capture + stopImmediatePropagation：避免父 AppModal 同帧也响应 Esc
    event.preventDefault();
    event.stopImmediatePropagation();
    open.value = false;
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    highlight.value = Math.min(props.options.length - 1, highlight.value + 1);
    return;
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    highlight.value = Math.max(0, highlight.value - 1);
    return;
  }
  if (event.key === "Enter") {
    event.preventDefault();
    const opt = props.options[highlight.value];
    if (opt) select(opt.value);
  }
}

function onScrollClose(event: Event) {
  if (!open.value) return;
  const t = event.target;
  // 菜单自身滚动不关闭
  if (t instanceof Node && menuRef.value?.contains(t)) return;
  open.value = false;
}

function bindOpenListeners() {
  document.addEventListener("scroll", onScrollClose, true);
}

function unbindOpenListeners() {
  document.removeEventListener("scroll", onScrollClose, true);
}

watch(open, (v) => {
  if (v) {
    bindOpenListeners();
    void placeMenu();
  } else {
    unbindOpenListeners();
  }
});

function onResize() {
  if (open.value) void placeMenu();
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointer, true);
  window.addEventListener("keydown", onKeydown, true);
  window.addEventListener("resize", onResize);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointer, true);
  window.removeEventListener("keydown", onKeydown, true);
  window.removeEventListener("resize", onResize);
  unbindOpenListeners();
});
</script>

<template>
  <div class="app-select" :class="{ compact, disabled, open }">
    <button
      ref="triggerRef"
      type="button"
      class="trigger"
      :disabled="disabled"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <span class="label">{{ selectedLabel }}</span>
      <span class="chevron" aria-hidden="true">▾</span>
    </button>
    <Teleport to="body" :disabled="teleport === false">
      <ul v-if="open" ref="menuRef" class="menu" role="listbox" :style="menuStyle">
        <li
          v-for="(opt, i) in options"
          :key="opt.value"
          role="option"
          class="option"
          :class="{ active: opt.value === modelValue, highlight: i === highlight }"
          :aria-selected="opt.value === modelValue"
          @mouseenter="highlight = i"
          @click="select(opt.value)"
        >
          {{ opt.label }}
        </li>
      </ul>
    </Teleport>
  </div>
</template>

<style scoped>
.app-select {
  display: inline-flex;
  min-width: 0;
  width: 100%;
}

.trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  font: inherit;
  font-size: 14px;
  color: var(--color-text);
  background: var(--color-surface);
  cursor: pointer;
  text-align: left;
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.compact .trigger {
  padding: 6px 8px;
  font-size: 13px;
}

.trigger:focus-visible {
  outline: none;
  border-color: var(--color-accent);
  box-shadow: var(--focus-ring);
}

.trigger:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chevron {
  color: var(--color-muted);
  font-size: 11px;
}

.menu {
  margin: 0;
  padding: 4px;
  list-style: none;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  box-shadow: var(--shadow-md);
  max-height: min(280px, 50vh);
  overflow-y: auto;
}

.option {
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  font-size: 14px;
  color: var(--color-text);
  cursor: pointer;
}

.option.highlight,
.option:hover {
  background: var(--color-bg-accent);
}

.option.active {
  font-weight: 600;
  color: var(--color-accent);
}
</style>
