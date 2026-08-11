import { ref, type Ref } from "vue";

import * as todoApi from "@/api/todos";
import type {
  CreateTodoDto,
  DueDateFilter,
  ListTodoQuery,
  Priority,
  TodoDto,
  TodoStatus,
} from "@/api/types";
import { formatErrorMessage } from "@/utils/error";
import { dueDateRangeForFilter } from "@/utils/date";

export interface UseTodosReturn {
  todos: Ref<TodoDto[]>;
  total: Ref<number>;
  page: Ref<number>;
  pageSize: Ref<number>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  keyword: Ref<string>;
  selectedStatuses: Ref<TodoStatus[]>;
  selectedPriorities: Ref<Priority[]>;
  selectedTags: Ref<string[]>;
  dueDateFilter: Ref<DueDateFilter>;
  selectedId: Ref<string | null>;
  fetchTodos: () => Promise<void>;
  createTodo: (dto: CreateTodoDto) => Promise<TodoDto>;
  selectTodo: (id: string | null) => Promise<void>;
  toggleStatusFilter: (status: TodoStatus) => void;
  togglePriorityFilter: (priority: Priority) => void;
  toggleTagFilter: (tag: string) => void;
  setDueDateFilter: (filter: DueDateFilter) => void;
  search: () => Promise<void>;
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
  const selectedStatuses = ref<TodoStatus[]>([]);
  const selectedPriorities = ref<Priority[]>([]);
  const selectedTags = ref<string[]>([]);
  const dueDateFilter = ref<DueDateFilter>("all");
  const selectedId = ref<string | null>(null);
  let listGen = 0;

  async function fetchTodos(): Promise<void> {
    const gen = ++listGen;
    loading.value = true;
    error.value = null;
    try {
      const query: ListTodoQuery = {
        page: page.value,
        pageSize: pageSize.value,
        sortBy: "priority",
        sortOrder: "desc",
        includeArchived: false,
      };
      if (keyword.value.trim()) {
        query.keyword = keyword.value.trim();
      }
      if (selectedStatuses.value.length > 0) {
        query.status = [...selectedStatuses.value];
        query.includeArchived = selectedStatuses.value.includes("Archived");
      } else if (dueDateFilter.value === "overdue" || dueDateFilter.value === "today") {
        // 与角标计数一致：仅活跃 Todo/Doing
        query.status = ["Todo", "Doing"];
      }
      if (selectedPriorities.value.length > 0) {
        query.priority = [...selectedPriorities.value];
      }
      if (selectedTags.value.length > 0) {
        query.tags = [...selectedTags.value];
      }
      const dueRange = dueDateRangeForFilter(dueDateFilter.value);
      if (dueRange.after) query.dueDateAfter = dueRange.after;
      if (dueRange.before) query.dueDateBefore = dueRange.before;

      const result = await todoApi.listTodos(query);
      if (gen !== listGen) return;
      todos.value = result.items;
      total.value = result.total;
      page.value = result.page;
      pageSize.value = result.pageSize;
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

  function toggleStatusFilter(status: TodoStatus): void {
    const index = selectedStatuses.value.indexOf(status);
    if (index >= 0) {
      selectedStatuses.value.splice(index, 1);
    } else {
      selectedStatuses.value.push(status);
    }
    page.value = 1;
    void fetchTodos();
  }

  function togglePriorityFilter(priority: Priority): void {
    const index = selectedPriorities.value.indexOf(priority);
    if (index >= 0) {
      selectedPriorities.value.splice(index, 1);
    } else {
      selectedPriorities.value.push(priority);
    }
    page.value = 1;
    void fetchTodos();
  }

  function toggleTagFilter(tag: string): void {
    const index = selectedTags.value.indexOf(tag);
    if (index >= 0) {
      selectedTags.value.splice(index, 1);
    } else {
      selectedTags.value.push(tag);
    }
    page.value = 1;
    void fetchTodos();
  }

  function setDueDateFilter(filter: DueDateFilter): void {
    dueDateFilter.value = filter;
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
    selectedStatuses,
    selectedPriorities,
    selectedTags,
    dueDateFilter,
    selectedId,
    fetchTodos,
    createTodo,
    selectTodo,
    toggleStatusFilter,
    togglePriorityFilter,
    toggleTagFilter,
    setDueDateFilter,
    search,
    goToPage,
  };
}
