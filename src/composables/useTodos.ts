import { ref, type Ref } from "vue";

import * as todoApi from "@/api/todos";
import type {
  CreateTodoDto,
  ListTodoQuery,
  SortBy,
  SortOrder,
  TodoDto,
  WorkbenchView,
} from "@/api/types";
import { formatErrorMessage } from "@/utils/error";
import { dueDateRangeForFilter } from "@/utils/date";
import { sortTodos } from "@/utils/listSort";

/** 今日视图双查询各自上限（合并去重后可能少于 2×limit） */
const TODAY_FETCH_LIMIT = 100;

export interface UseTodosReturn {
  todos: Ref<TodoDto[]>;
  total: Ref<number>;
  page: Ref<number>;
  pageSize: Ref<number>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  keyword: Ref<string>;
  view: Ref<WorkbenchView>;
  activeTag: Ref<string | null>;
  sortBy: Ref<SortBy>;
  sortOrder: Ref<SortOrder>;
  selectedId: Ref<string | null>;
  fetchTodos: () => Promise<void>;
  createTodo: (dto: CreateTodoDto) => Promise<TodoDto>;
  selectTodo: (id: string | null) => Promise<void>;
  setView: (next: WorkbenchView, tag?: string | null) => void;
  setSort: (sortBy: SortBy, sortOrder?: SortOrder) => void;
  search: () => Promise<void>;
  goToPage: (nextPage: number) => Promise<void>;
}

function mergeById(lists: TodoDto[][]): TodoDto[] {
  const map = new Map<string, TodoDto>();
  for (const list of lists) {
    for (const item of list) {
      map.set(item.id, item);
    }
  }
  return [...map.values()];
}

export function useTodos(): UseTodosReturn {
  const todos = ref<TodoDto[]>([]);
  const total = ref(0);
  const page = ref(1);
  const pageSize = ref(50);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const keyword = ref("");
  const view = ref<WorkbenchView>("today");
  const activeTag = ref<string | null>(null);
  const sortBy = ref<SortBy>("priority");
  const sortOrder = ref<SortOrder>("desc");
  const selectedId = ref<string | null>(null);
  let listGen = 0;

  function baseQuery(): ListTodoQuery {
    const query: ListTodoQuery = {
      page: page.value,
      pageSize: pageSize.value,
      sortBy: sortBy.value,
      sortOrder: sortOrder.value,
      includeArchived: false,
      includeDeleted: false,
    };
    if (keyword.value.trim()) {
      query.keyword = keyword.value.trim();
    }
    return query;
  }

  async function fetchTodos(): Promise<void> {
    const gen = ++listGen;
    loading.value = true;
    error.value = null;
    try {
      let items: TodoDto[] = [];
      let resultTotal = 0;
      let resultPage = page.value;

      if (view.value === "today") {
        const dueRange = dueDateRangeForFilter("today");
        const dueQuery: ListTodoQuery = {
          ...baseQuery(),
          status: ["Todo", "Doing"],
          dueDateAfter: dueRange.after,
          dueDateBefore: dueRange.before,
          page: 1,
          pageSize: TODAY_FETCH_LIMIT,
        };
        const doingQuery: ListTodoQuery = {
          ...baseQuery(),
          status: ["Doing"],
          page: 1,
          pageSize: TODAY_FETCH_LIMIT,
        };
        const [dueResult, doingResult] = await Promise.all([
          todoApi.listTodos(dueQuery),
          todoApi.listTodos(doingQuery),
        ]);
        if (gen !== listGen) return;
        items = sortTodos(
          mergeById([dueResult.items, doingResult.items]),
          sortBy.value,
          sortOrder.value,
        );
        resultTotal = items.length;
        resultPage = 1;
      } else if (view.value === "overdue") {
        const dueRange = dueDateRangeForFilter("overdue");
        const result = await todoApi.listTodos({
          ...baseQuery(),
          status: ["Todo", "Doing"],
          dueDateBefore: dueRange.before,
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      } else if (view.value === "doing") {
        const result = await todoApi.listTodos({
          ...baseQuery(),
          status: ["Doing"],
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      } else if (view.value === "all") {
        const result = await todoApi.listTodos({
          ...baseQuery(),
          status: ["Todo", "Doing"],
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      } else if (view.value === "done") {
        const result = await todoApi.listTodos({
          ...baseQuery(),
          status: ["Done"],
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      } else if (view.value === "archived") {
        const result = await todoApi.listTodos({
          ...baseQuery(),
          status: ["Archived"],
          includeArchived: true,
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      } else if (view.value === "trash") {
        const result = await todoApi.listTodos({
          ...baseQuery(),
          includeDeleted: true,
          includeArchived: true,
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      } else {
        const result = await todoApi.listTodos({
          ...baseQuery(),
          status: ["Todo", "Doing"],
          tags: activeTag.value ? [activeTag.value] : undefined,
        });
        if (gen !== listGen) return;
        items = result.items;
        resultTotal = result.total;
        resultPage = result.page;
      }

      todos.value = items;
      total.value = resultTotal;
      page.value = resultPage;
    } catch (err) {
      if (gen !== listGen) return;
      error.value = formatErrorMessage(err);
    } finally {
      if (gen === listGen) {
        loading.value = false;
      }
    }
  }

  async function createTodo(dto: CreateTodoDto): Promise<TodoDto> {
    const todo = await todoApi.createTodo(dto);
    selectedId.value = todo.id;
    await fetchTodos();
    return todo;
  }

  async function selectTodo(id: string | null): Promise<void> {
    selectedId.value = id;
  }

  function setView(next: WorkbenchView, tag: string | null = null): void {
    view.value = next;
    activeTag.value = next === "tag" ? tag : null;
    page.value = 1;
    void fetchTodos();
  }

  function setSort(nextSortBy: SortBy, nextOrder: SortOrder = sortOrder.value): void {
    sortBy.value = nextSortBy;
    sortOrder.value = nextOrder;
    page.value = 1;
    void fetchTodos();
  }

  async function search(): Promise<void> {
    page.value = 1;
    await fetchTodos();
  }

  async function goToPage(nextPage: number): Promise<void> {
    if (nextPage < 1) return;
    const maxPage = Math.max(1, Math.ceil(total.value / pageSize.value));
    if (nextPage > maxPage) return;
    page.value = nextPage;
    await fetchTodos();
  }

  return {
    todos,
    total,
    page,
    pageSize,
    loading,
    error,
    keyword,
    view,
    activeTag,
    sortBy,
    sortOrder,
    selectedId,
    fetchTodos,
    createTodo,
    selectTodo,
    setView,
    setSort,
    search,
    goToPage,
  };
}
