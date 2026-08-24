/** 状态机合法边 — 权威定义在 Rust domain/status.rs，经 get_allowed_transitions 下发 */
import { ref, watch, type Ref } from "vue";
import type { Composer } from "vue-i18n";

import { getAllowedTransitions } from "@/api/todos";
import type { StatusActionDto, TodoStatus } from "@/api/types";

export type StatusAction = StatusActionDto;

/** 忽略后端中文 label，按 from→to 映射到 i18n */
export function statusActionLabel(from: TodoStatus, to: TodoStatus, t: Composer["t"]): string {
  if (to === "Doing") return t("statusAction.start");
  if (to === "Done") return t("statusAction.complete");
  if (to === "Archived") return t("statusAction.archive");
  if (to === "Todo") {
    if (from === "Doing") return t("statusAction.backToTodo");
    return t("statusAction.reopen");
  }
  return to;
}

/** 供 Inspector / 助理详情：随 status 拉取后端动词列表 */
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
