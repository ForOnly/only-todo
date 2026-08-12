import type { ListDefaultSort, SortBy, SortOrder, TodoDto } from "@/api/types";

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

function priorityRank(priority: TodoDto["priority"]): number {
  switch (priority) {
    case "Urgent":
      return 4;
    case "High":
      return 3;
    case "Medium":
      return 2;
    default:
      return 1;
  }
}

/** 客户端合并列表（如今日视图）排序 */
export function sortTodos(items: TodoDto[], sortBy: SortBy, sortOrder: SortOrder): TodoDto[] {
  const dir = sortOrder === "desc" ? -1 : 1;
  return [...items].sort((a, b) => {
    let cmp = 0;
    switch (sortBy) {
      case "priority":
        cmp = priorityRank(a.priority) - priorityRank(b.priority);
        break;
      case "dueDate": {
        const da = a.dueDate ? new Date(a.dueDate).getTime() : Number.POSITIVE_INFINITY;
        const db = b.dueDate ? new Date(b.dueDate).getTime() : Number.POSITIVE_INFINITY;
        cmp = da - db;
        break;
      }
      case "updatedAt":
        cmp = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime();
        break;
      case "createdAt":
        cmp = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
        break;
      case "completedAt": {
        const ca = a.completedAt ? new Date(a.completedAt).getTime() : 0;
        const cb = b.completedAt ? new Date(b.completedAt).getTime() : 0;
        cmp = ca - cb;
        break;
      }
      case "title":
        cmp = a.title.localeCompare(b.title, "zh-CN");
        break;
    }
    if (cmp === 0) {
      cmp = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
    }
    return cmp * dir;
  });
}

/** 各视图可用的排序选项 */
export function sortOptionsForView(view: string): { value: SortBy; label: string }[] {
  const base = [
    { value: "priority" as SortBy, label: "优先级" },
    { value: "dueDate" as SortBy, label: "截止日期" },
    { value: "updatedAt" as SortBy, label: "最近更新" },
    { value: "createdAt" as SortBy, label: "创建时间" },
    { value: "title" as SortBy, label: "标题" },
  ];
  if (view === "done") {
    return [
      { value: "completedAt", label: "完成时间" },
      ...base.filter((o) => o.value !== "dueDate"),
    ];
  }
  return base;
}
