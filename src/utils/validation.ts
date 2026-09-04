import type { CreateTodoFormModel } from "@/api/types";
import {
  DESCRIPTION_MAX_LENGTH,
  MAX_TAG_LEN,
  MAX_TAGS,
  TITLE_MAX_LENGTH,
} from "@/api/types";
import { i18n } from "@/i18n";

/** 创建表单标签解析（逗号/空白切分、去重） */
export function parseCreateTags(text: string): string[] {
  const seen = new Set<string>();
  const tags: string[] = [];
  for (const part of text.split(/[,，\s]+/)) {
    const tag = part.trim();
    if (!tag) continue;
    const key = tag.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    tags.push(tag);
  }
  return tags;
}

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
  const tags = parseCreateTags(form.tagsText);
  if (tags.length > MAX_TAGS) {
    return i18n.global.t("validation.tooManyTags", { n: MAX_TAGS });
  }
  for (const tag of tags) {
    if (tag.length > MAX_TAG_LEN) {
      return i18n.global.t("validation.tagTooLong", { n: MAX_TAG_LEN });
    }
  }
  return null;
}
