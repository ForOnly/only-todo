import { invoke } from "@tauri-apps/api/core";

import type { EventDto, ListEventsQuery } from "@/api/types";

/** Tauri Command: list_events_cmd */
export async function listEvents(query: ListEventsQuery = {}): Promise<EventDto[]> {
  return invoke("list_events_cmd", { query });
}
