import type { ListDefaultSort, SortBy, SortOrder } from "@/api/types";

const SORT_BY: SortBy[] = [
  "priority",
  "createdAt",
  "updatedAt",
  "completedAt",
  "dueDate",
  "title",
];

/** 解析 settings.listDefaultSort（后端种子为 snake_case JSON） */
export function parseListDefaultSort(raw: string | null | undefined): ListDefaultSort {
  const fallback: ListDefaultSort = { sortBy: "priority", sortOrder: "desc" };
  if (!raw?.trim()) return fallback;
  try {
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    const sortByRaw = (parsed.sortBy ?? parsed.sort_by) as string | undefined;
    const sortOrderRaw = (parsed.sortOrder ?? parsed.sort_order) as string | undefined;
    const sortBy = SORT_BY.includes(sortByRaw as SortBy) ? (sortByRaw as SortBy) : fallback.sortBy;
    const sortOrder: SortOrder =
      sortOrderRaw === "asc" || sortOrderRaw === "desc" ? sortOrderRaw : fallback.sortOrder;
    return { sortBy, sortOrder };
  } catch {
    return fallback;
  }
}

export function serializeListDefaultSort(value: ListDefaultSort): string {
  return JSON.stringify({
    sort_by: value.sortBy,
    sort_order: value.sortOrder,
  });
}

/** 客户端合并列表（如今日视图）排序键；标签由调用方 $t(`sort.${value}`) */
export function sortKeysForView(view: string): SortBy[] {
  const base: SortBy[] = ["priority", "dueDate", "updatedAt", "createdAt", "title"];
  if (view === "done") {
    return ["completedAt", ...base.filter((o) => o !== "dueDate")];
  }
  return base;
}
