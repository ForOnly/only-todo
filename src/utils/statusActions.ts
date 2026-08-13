/** 状态机合法边 — 权威定义在 Rust domain/status.rs，经 get_allowed_transitions 下发 */
import { ref, watch, type Ref } from "vue";

import { getAllowedTransitions } from "@/api/todos";
import type { StatusActionDto, TodoStatus } from "@/api/types";

export type StatusAction = StatusActionDto;

/** 供 Inspector / 伴侣详情：随 status 拉取后端动词列表 */
export function useStatusActions(status: Ref<TodoStatus | null | undefined>): {
  actions: Ref<StatusActionDto[]>;
} {
  const actions = ref<StatusActionDto[]>([]);

  watch(
    status,
    async (value) => {
      if (!value) {
        actions.value = [];
        return;
      }
      try {
        actions.value = await getAllowedTransitions(value);
      } catch {
        actions.value = [];
      }
    },
    { immediate: true },
  );

  return { actions };
}
