import type { CreateTodoFormModel } from "@/api/types";
import { DESCRIPTION_MAX_LENGTH, TITLE_MAX_LENGTH } from "@/api/types";
import { i18n } from "@/i18n";

/** 调用 create_todo 前的客户端校验 */
export function validateCreateTodoForm(form: CreateTodoFormModel): string | null {
  const title = form.title.trim();
  if (!title) {
    return i18n.global.t("validation.titleRequired");
  }
  if (title.length > TITLE_MAX_LENGTH) {
    return i18n.global.t("validation.titleTooLong", { n: TITLE_MAX_LENGTH });
  }
  if (form.description.length > DESCRIPTION_MAX_LENGTH) {
    return i18n.global.t("validation.descriptionTooLong", { n: DESCRIPTION_MAX_LENGTH });
  }
  return null;
}
