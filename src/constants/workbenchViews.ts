import type { WorkbenchView } from "@/api/types";

/** 侧栏主列表（不含标签、归档、回收站） */
export const SIDEBAR_MAIN_VIEWS: WorkbenchView[] = ["all", "done"];

/** 侧栏底栏：归档 / 回收站 */
export const SIDEBAR_FOOTER_VIEWS: WorkbenchView[] = ["archived", "trash"];

/** 打开主窗默认落点候选 */
export const DEFAULT_VIEW_OPTIONS: WorkbenchView[] = [
  ...SIDEBAR_MAIN_VIEWS,
  ...SIDEBAR_FOOTER_VIEWS,
];

export function coerceDefaultView(view: WorkbenchView): WorkbenchView {
  return DEFAULT_VIEW_OPTIONS.includes(view) ? view : "all";
}
