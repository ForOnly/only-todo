/** 将 ISO 时间格式化为列表/详情展示用字符串 */
export function formatDate(value: string): string {
  return new Date(value).toLocaleString();
}

/** 将 ISO 时间格式化为通知正文用字符串（月日 + 时分） */
export function formatDateTime(value: string): string {
  const date = new Date(value);
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
