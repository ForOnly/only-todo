import { invoke } from "@tauri-apps/api/core";

import type { CreateReminderDto, ReminderDto, UpdateReminderDto } from "@/api/types";

/** Tauri Command: create_reminder */
export async function createReminder(dto: CreateReminderDto): Promise<ReminderDto> {
  return invoke("create_reminder", { dto });
}

/** Tauri Command: update_reminder */
export async function updateReminder(dto: UpdateReminderDto): Promise<ReminderDto> {
  return invoke("update_reminder", { dto });
}

/** Tauri Command: delete_reminder */
export async function deleteReminder(id: string): Promise<void> {
  return invoke("delete_reminder", { id });
}

/** Tauri Command: list_reminders */
export async function listReminders(todoId: string): Promise<ReminderDto[]> {
  return invoke("list_reminders", { todoId });
}

/** Tauri Command: snooze_reminder */
export async function snoozeReminder(id: string, minutes: number): Promise<ReminderDto> {
  return invoke("snooze_reminder", { id, minutes });
}
