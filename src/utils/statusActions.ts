/** 状态机合法边 → Inspector 动词按钮 */
import type { TodoStatus } from "@/api/types";

export interface StatusAction {
  label: string;
  target: TodoStatus;
}

export function statusActionsFor(status: TodoStatus): StatusAction[] {
  switch (status) {
    case "Todo":
      return [
        { label: "开始", target: "Doing" },
        { label: "完成", target: "Done" },
        { label: "归档", target: "Archived" },
      ];
    case "Doing":
      return [
        { label: "回待办", target: "Todo" },
        { label: "完成", target: "Done" },
        { label: "归档", target: "Archived" },
      ];
    case "Done":
      return [
        { label: "重开", target: "Todo" },
        { label: "归档", target: "Archived" },
      ];
    case "Archived":
      return [{ label: "重开", target: "Todo" }];
    default:
      return [];
  }
}
