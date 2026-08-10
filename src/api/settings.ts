import { invoke } from "@tauri-apps/api/core";

import type { SettingsDto, UpdateSettingsDto } from "@/api/types";

/** Tauri Command: get_settings */
export async function getSettings(): Promise<SettingsDto> {
  return invoke("get_settings");
}

/** Tauri Command: update_settings */
export async function updateSettings(dto: UpdateSettingsDto): Promise<SettingsDto> {
  return invoke("update_settings", { dto });
}
