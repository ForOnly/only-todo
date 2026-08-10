import type { CreateTodoFormModel } from "@/api/types";
import { DESCRIPTION_MAX_LENGTH, TITLE_MAX_LENGTH } from "@/api/types";

/** 调用 create_todo 前的客户端校验 */
export function validateCreateTodoForm(
  form: CreateTodoFormModel,
): string | null {
  const title = form.title.trim();
  if (!title) {
    return "标题不能为空";
  }
  if (title.length > TITLE_MAX_LENGTH) {
    return `标题不能超过 ${TITLE_MAX_LENGTH} 个字符`;
  }
  if (form.description.length > DESCRIPTION_MAX_LENGTH) {
    return `描述不能超过 ${DESCRIPTION_MAX_LENGTH} 个字符`;
  }
  return null;
}
