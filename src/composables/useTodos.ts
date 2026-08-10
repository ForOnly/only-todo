/**
 * 任务列表状态：筛选、分页、CRUD 编排。
 * 通过 api/todos.ts 调用 Tauri Command。
 */
import { ref, type Ref } from "vue";

import * as todoApi from "@/api/todos";
import type {
  CreateTodoDto,
  ListTodoQuery,
  Priority,
  TodoDto,
  TodoStatus,
} from "@/api/types";
import { formatErrorMessage } from "@/utils/error";

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
  selectedId: Ref<string | null>;
  fetchTodos: () => Promise<void>;
  createTodo: (dto: CreateTodoDto) => Promise<TodoDto>;
  selectTodo: (id: string | null) => Promise<void>;
  toggleStatusFilter: (status: TodoStatus) => void;
  togglePriorityFilter: (priority: Priority) => void;
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
  const selectedId = ref<string | null>(null);

  async function fetchTodos(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const query: ListTodoQuery = {
        page: page.value,
        pageSize: pageSize.value,
        sortBy: "priority",
        sortOrder: "desc",
      };
      if (keyword.value.trim()) {
        query.keyword = keyword.value.trim();
      }
      if (selectedStatuses.value.length > 0) {
        query.status = [...selectedStatuses.value];
      }
      if (selectedPriorities.value.length > 0) {
        query.priority = [...selectedPriorities.value];
      }
      const result = await todoApi.listTodos(query);
      todos.value = result.items;
      total.value = result.total;
      page.value = result.page;
      pageSize.value = result.pageSize;
    } catch (err) {
      error.value = formatErrorMessage(err);
    } finally {
      loading.value = false;
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
    selectedId,
    fetchTodos,
    createTodo,
    selectTodo,
    toggleStatusFilter,
    togglePriorityFilter,
    search,
    goToPage,
  };
}
