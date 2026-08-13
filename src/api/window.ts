import { invoke } from "@tauri-apps/api/core";

import type {
  BodyView,
  CompanionDragEndResult,
  CompanionSession,
  CompanionSurface,
} from "@/api/types";

/** Tauri Command: show_floating_window */
export async function showFloatingWindow(): Promise<void> {
  return invoke("show_floating_window");
}

/** Tauri Command: hide_floating_window */
export async function hideFloatingWindow(): Promise<void> {
  return invoke("hide_floating_window");
}

/** Tauri Command: get_companion_session（只读快照，不广播） */
export async function getCompanionSession(): Promise<CompanionSession> {
  return invoke("get_companion_session");
}

/** Tauri Command: companion_refresh_session */
export async function companionRefreshSession(): Promise<CompanionSession> {
  return invoke("companion_refresh_session");
}

/** Tauri Command: companion_click_chrome */
export async function companionClickChrome(): Promise<CompanionSession> {
  return invoke("companion_click_chrome");
}

/** Tauri Command: companion_minimize */
export async function companionMinimize(): Promise<CompanionSession> {
  return invoke("companion_minimize");
}

/** Tauri Command: companion_collapse_to_strip（历史名；矩阵：回球或收条） */
export async function companionCollapseToStrip(): Promise<CompanionSession> {
  return invoke("companion_collapse_to_strip");
}

/** Tauri Command: companion_drag_ended */
export async function companionDragEnded(which: CompanionSurface): Promise<CompanionDragEndResult> {
  return invoke("companion_drag_ended", { which });
}

/** Tauri Command: companion_pointer_cluster */
export async function companionPointerCluster(
  surface: CompanionSurface,
  opts?: { inside?: boolean; focused?: boolean },
): Promise<CompanionSession> {
  return invoke("companion_pointer_cluster", {
    surface,
    inside: opts?.inside ?? null,
    focused: opts?.focused ?? null,
  });
}

/** Tauri Command: companion_open_view */
export async function companionOpenView(view: BodyView): Promise<CompanionSession> {
  return invoke("companion_open_view", { view });
}
