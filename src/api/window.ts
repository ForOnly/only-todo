import { invoke } from "@tauri-apps/api/core";

import type { DockEdge, FloatDisplayMode, FloatWindowState } from "@/api/types";

/** Tauri Command: show_floating_window */
export async function showFloatingWindow(): Promise<void> {
  return invoke("show_floating_window");
}

/** Tauri Command: hide_floating_window */
export async function hideFloatingWindow(): Promise<void> {
  return invoke("hide_floating_window");
}

/** Tauri Command: toggle_floating_window */
export async function toggleFloatingWindow(): Promise<void> {
  return invoke("toggle_floating_window");
}

/** Tauri Command: get_active_todo_count */
export async function getActiveTodoCount(): Promise<number> {
  return invoke("get_active_todo_count");
}

/** Tauri Command: get_float_window_state */
export async function getFloatWindowState(): Promise<FloatWindowState> {
  return invoke("get_float_window_state");
}

/** Tauri Command: set_float_display_mode */
export async function setFloatDisplayMode(mode: FloatDisplayMode): Promise<FloatWindowState> {
  return invoke("set_float_display_mode", { mode });
}

/** Tauri Command: dock_float_window */
export async function dockFloatWindow(edge?: DockEdge): Promise<FloatWindowState> {
  return invoke("dock_float_window", { edge: edge ?? null });
}

/** Tauri Command: undock_float_window */
export async function undockFloatWindow(): Promise<FloatWindowState> {
  return invoke("undock_float_window");
}

/** Tauri Command: try_dock_on_edge — 拖拽结束后检测边缘吸附 */
export async function tryDockOnEdge(): Promise<FloatWindowState> {
  return invoke("try_dock_on_edge");
}

/** Tauri Command: peek_docked_float — clientY 为相对窗口客户区的 Y */
export async function peekDockedFloat(clientY?: number): Promise<FloatWindowState> {
  return invoke("peek_docked_float", { clientY: clientY ?? null });
}

/** Tauri Command: unpeek_docked_float — 挂边移开收回竖线 */
export async function unpeekDockedFloat(): Promise<FloatWindowState> {
  return invoke("unpeek_docked_float");
}

/** OS 级主按键是否仍按下（native drag 后 WebView 常收不到 pointerup） */
export async function isPrimaryMouseDown(): Promise<boolean> {
  return invoke("is_primary_mouse_down");
}
