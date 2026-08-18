import { invoke } from "@tauri-apps/api/core";

import type { AppearanceDto, SettingsDto, UpdateSettingsDto } from "@/api/types";

/** Tauri Command: get_settings */
export async function getSettings(): Promise<SettingsDto> {
  return invoke("get_settings");
}

/** 只读 resolved 浅/深；给 Canvas 等非 CSS。不要用来写 DOM 主题。 */
export async function getAppearance(): Promise<AppearanceDto> {
  return invoke("get_appearance");
}

/** Tauri Command: update_settings */
export async function updateSettings(dto: UpdateSettingsDto): Promise<SettingsDto> {
  return invoke("update_settings", { dto });
}
