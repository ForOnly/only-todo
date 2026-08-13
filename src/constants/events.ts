/**
 * Tauri 后端发出的事件名，前端在 App.vue 中监听。
 * 修改事件名须同步更新 lib.rs 中的 emit 调用。
 */
export const TAURI_EVENTS = {
  /** 托盘菜单「新建任务」或程序化触发 — 打开 CreateTodoModal */
  CREATE_TODO: "create-todo",
  /** 通知点击或 show_main_window(todo_id) — 在列表中选中任务 */
  NAVIGATE_TO_TODO: "navigate-to-todo",
  /** 托盘菜单「设置」— 打开 SettingsModal */
  OPEN_SETTINGS: "open-settings",
  /** 设置已更新 — 助理刷新可见条数等 */
  SETTINGS_UPDATED: "settings-updated",
  /** 任务/提醒写成功 — 主窗与助理刷新列表与计数 */
  TODOS_CHANGED: "todos-changed",
  /** 助理会话变更 */
  FLOAT_SESSION_CHANGED: "float-session-changed",
} as const;

export type TauriEventName = (typeof TAURI_EVENTS)[keyof typeof TAURI_EVENTS];
