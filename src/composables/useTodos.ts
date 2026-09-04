import { ref, type Ref } from "vue";

import * as todoApi from "@/api/todos";
import type { CreateTodoDto, SortBy, SortOrder, TodoDto, WorkbenchView } from "@/api/types";
import { formatErrorMessage } from "@/utils/error";

export interface UseTodosReturn {
  todos: Ref<TodoDto[]>;
  total: Ref<number>;
  page: Ref<number>;
  pageSize: Ref<number>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  keyword: Ref<string>;
  searchActive: Ref<boolean>;
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
  clearSearch: () => Promise<void>;
  goToPage: (nextPage: number) => Promise<void>;
}

export function useTodos(): UseTodosReturn {
  const todos = ref<TodoDto[]>([]);
  const total = ref(0);
  const page = ref(1);
  const pageSize = ref(50);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const keyword = ref("");
  const searchActive = ref(false);
  const view = ref<WorkbenchView>("all");
  const activeTag = ref<string | null>(null);
  const sortBy = ref<SortBy>("priority");
  const sortOrder = ref<SortOrder>("desc");
  const selectedId = ref<string | null>(null);
  let listGen = 0;

  async function fetchTodos(): Promise<void> {
    const gen = ++listGen;
    if (todos.value.length === 0) {
      loading.value = true;
    }
    error.value = null;
    try {
      if (searchActive.value && keyword.value.trim()) {
        const result = await todoApi.listTodos({
          keyword: keyword.value.trim(),
          status: ["Todo", "Doing", "Done"],
          sortBy: sortBy.value,
          sortOrder: sortOrder.value,
          page: page.value,
          pageSize: pageSize.value,
        });
        if (gen !== listGen) return;
        todos.value = result.items;
        total.value = result.total;
        page.value = result.page;
        return;
      }

      const result = await todoApi.listWorkbenchTodos({
        view: view.value,
        tag: view.value === "tag" ? (activeTag.value ?? undefined) : undefined,
        sortBy: sortBy.value,
        sortOrder: sortOrder.value,
        page: page.value,
        pageSize: pageSize.value,
      });
      if (gen !== listGen) return;
      todos.value = result.items;
      total.value = result.total;
      page.value = result.page;
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
    searchActive.value = false;
    keyword.value = "";
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
    searchActive.value = Boolean(keyword.value.trim());
    await fetchTodos();
  }

  async function clearSearch(): Promise<void> {
    keyword.value = "";
    searchActive.value = false;
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
    searchActive,
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
    clearSearch,
    goToPage,
  };
}
