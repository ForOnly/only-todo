import type { WorkbenchView } from "@/api/types";

/** 库模式主列表（不含标签、归档、回收站） */
export const LIBRARY_MAIN_VIEWS: WorkbenchView[] = ["all", "done"];

/** 库模式底栏：归档 / 回收站 */
export const LIBRARY_FOOTER_VIEWS: WorkbenchView[] = ["archived", "trash"];

/** 打开主窗默认落点：焦点台或库内归类 */
export const DEFAULT_VIEW_OPTIONS: WorkbenchView[] = [
  "today",
  ...LIBRARY_MAIN_VIEWS,
  ...LIBRARY_FOOTER_VIEWS,
];

export function isLibraryView(view: WorkbenchView): boolean {
  return (
    view === "all" ||
    view === "done" ||
    view === "archived" ||
    view === "trash" ||
    view === "tag"
  );
}

export function coerceDefaultView(view: WorkbenchView): WorkbenchView {
  return DEFAULT_VIEW_OPTIONS.includes(view) ? view : "today";
}
