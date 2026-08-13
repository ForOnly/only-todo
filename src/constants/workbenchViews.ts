import type { WorkbenchView } from "@/api/types";

/** 侧栏主归类（不含标签、归档、回收站） */
export const MAIN_WORKBENCH_VIEWS: WorkbenchView[] = [
  "today",
  "doing",
  "overdue",
  "done",
  "all",
];

/** 侧栏底栏：归档 / 回收站 */
export const FOOTER_WORKBENCH_VIEWS: WorkbenchView[] = ["archived", "trash"];

/** 可作为启动默认归类的侧栏项（不含标签） */
export const DEFAULT_VIEW_OPTIONS: WorkbenchView[] = [
  ...MAIN_WORKBENCH_VIEWS,
  ...FOOTER_WORKBENCH_VIEWS,
];
