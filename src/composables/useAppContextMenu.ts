import { reactive } from "vue";

export type ContextMenuAction = "cut" | "copy" | "paste" | "selectAll";

export type ContextMenuItem = {
  id: ContextMenuAction;
  enabled: boolean;
};

type ContextMenuState = {
  open: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
  target: HTMLElement | null;
};

const state = reactive<ContextMenuState>({
  open: false,
  x: 0,
  y: 0,
  items: [],
  target: null,
});

function isEditable(el: Element | null): el is HTMLInputElement | HTMLTextAreaElement {
  if (!el) return false;
  if (el instanceof HTMLTextAreaElement) return !el.disabled && !el.readOnly;
  if (el instanceof HTMLInputElement) {
    if (el.disabled || el.readOnly) return false;
    const t = el.type;
    return (
      t === "text" ||
      t === "search" ||
      t === "password" ||
      t === "email" ||
      t === "url" ||
      t === "tel" ||
      t === "number" ||
      t === ""
    );
  }
  return (el as HTMLElement).isContentEditable;
}

function hasTextSelection(el: HTMLInputElement | HTMLTextAreaElement | null): boolean {
  if (el && "selectionStart" in el) {
    const start = el.selectionStart ?? 0;
    const end = el.selectionEnd ?? 0;
    return end > start;
  }
  const sel = window.getSelection();
  return !!sel && sel.toString().length > 0;
}

/** 根据右键目标构建菜单；无可操作项时返回 false */
export function openContextMenu(event: MouseEvent): boolean {
  const target = event.target;
  if (!(target instanceof Element)) return false;

  const editable = target.closest("input, textarea, [contenteditable='true']");
  const editableEl = isEditable(editable) ? editable : null;
  const selectionText = window.getSelection()?.toString() ?? "";

  const items: ContextMenuItem[] = [];

  if (editableEl) {
    const selected = hasTextSelection(editableEl);
    items.push(
      { id: "cut", enabled: selected },
      { id: "copy", enabled: selected },
      { id: "paste", enabled: true },
      { id: "selectAll", enabled: true },
    );
  } else if (selectionText.length > 0) {
    items.push({ id: "copy", enabled: true });
  } else {
    return false;
  }

  state.open = true;
  state.x = event.clientX;
  state.y = event.clientY;
  state.items = items;
  state.target = editableEl ?? (target as HTMLElement);
  return true;
}

export function closeContextMenu(): void {
  state.open = false;
  state.target = null;
  state.items = [];
}

async function writeClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // ignore
  }
}

async function readClipboard(): Promise<string> {
  try {
    return await navigator.clipboard.readText();
  } catch {
    return "";
  }
}

export async function runContextAction(id: ContextMenuAction): Promise<void> {
  const el = state.target;
  const sel = window.getSelection();

  if (id === "copy") {
    if (el && (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) {
      const start = el.selectionStart ?? 0;
      const end = el.selectionEnd ?? 0;
      if (end > start) await writeClipboard(el.value.slice(start, end));
    } else if (sel && sel.toString()) {
      await writeClipboard(sel.toString());
    }
  } else if (id === "cut") {
    if (el && (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) {
      const start = el.selectionStart ?? 0;
      const end = el.selectionEnd ?? 0;
      if (end > start) {
        await writeClipboard(el.value.slice(start, end));
        const next = el.value.slice(0, start) + el.value.slice(end);
        el.value = next;
        el.dispatchEvent(new Event("input", { bubbles: true }));
        el.setSelectionRange(start, start);
      }
    }
  } else if (id === "paste") {
    if (el && (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) {
      const text = await readClipboard();
      const start = el.selectionStart ?? el.value.length;
      const end = el.selectionEnd ?? el.value.length;
      let next = el.value.slice(0, start) + text + el.value.slice(end);
      const max = el.maxLength;
      if (max > 0 && next.length > max) {
        next = next.slice(0, max);
      }
      el.value = next;
      const caret = Math.min(start + text.length, next.length);
      el.dispatchEvent(new Event("input", { bubbles: true }));
      el.setSelectionRange(caret, caret);
      el.focus();
    }
  } else if (id === "selectAll") {
    if (el && (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement)) {
      el.focus();
      el.select();
    } else {
      document.execCommand("selectAll");
    }
  }

  closeContextMenu();
}

export function useAppContextMenuState() {
  return state;
}
