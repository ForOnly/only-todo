const FILTER_COLLAPSED_KEY = "only-todo.filter.collapsed";
const FILTER_PINNED_KEY = "only-todo.filter.pinned";

export function loadFilterCollapsed(): boolean {
  const value = localStorage.getItem(FILTER_COLLAPSED_KEY);
  if (value === null) return false; // 默认展开
  return value === "true";
}

export function loadFilterPinned(): boolean {
  return localStorage.getItem(FILTER_PINNED_KEY) === "true";
}

export function saveFilterCollapsed(collapsed: boolean): void {
  localStorage.setItem(FILTER_COLLAPSED_KEY, String(collapsed));
}

export function saveFilterPinned(pinned: boolean): void {
  localStorage.setItem(FILTER_PINNED_KEY, String(pinned));
}
