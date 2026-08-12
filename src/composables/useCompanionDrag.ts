import { getCurrentWindow } from "@tauri-apps/api/window";
import { onUnmounted } from "vue";

import { companionDragEnded } from "@/api/window";
import type { CompanionSurface } from "@/api/types";

const DRAG_SETTLE_MS = 180;
const DRAG_SAFETY_MS = 3000;

/** native startDragging 后 WebView 常丢 pointerup；Moved 静止后再通知 Host */
export function useCompanionDrag(which: CompanionSurface) {
  let didDrag = false;
  let sessionId = 0;
  let settleTimer: ReturnType<typeof setTimeout> | null = null;
  let safetyTimer: ReturnType<typeof setTimeout> | null = null;
  let unlistenMoved: (() => void) | null = null;
  let movedReady: Promise<void> | null = null;

  function clearTimers() {
    if (settleTimer) {
      clearTimeout(settleTimer);
      settleTimer = null;
    }
    if (safetyTimer) {
      clearTimeout(safetyTimer);
      safetyTimer = null;
    }
  }

  function armSettle() {
    if (settleTimer) clearTimeout(settleTimer);
    settleTimer = setTimeout(() => {
      settleTimer = null;
      void finish();
    }, DRAG_SETTLE_MS);
  }

  async function ensureMovedListener() {
    if (movedReady) {
      await movedReady;
      return;
    }
    movedReady = (async () => {
      unlistenMoved = await getCurrentWindow().onMoved(() => {
        if (!didDrag) return;
        armSettle();
        if (safetyTimer) clearTimeout(safetyTimer);
        safetyTimer = setTimeout(() => {
          safetyTimer = null;
          void finish();
        }, DRAG_SAFETY_MS);
      });
    })();
    await movedReady;
  }

  async function finish() {
    if (!didDrag) return;
    const id = sessionId;
    clearTimers();
    try {
      const result = await companionDragEnded(which);
      if (id !== sessionId) return;
      if (result.stillDragging) {
        armSettle();
        return;
      }
      didDrag = false;
    } catch (err) {
      console.error("companionDragEnded failed", err);
      if (id === sessionId) {
        didDrag = false;
      }
    }
  }

  async function startDrag() {
    sessionId += 1;
    didDrag = true;
    clearTimers();
    safetyTimer = setTimeout(() => {
      safetyTimer = null;
      void finish();
    }, DRAG_SAFETY_MS);
    await ensureMovedListener();
    try {
      await getCurrentWindow().startDragging();
    } catch (err) {
      console.error("startDragging failed", err);
      didDrag = false;
      clearTimers();
    }
  }

  function onPointerUp() {
    if (didDrag) {
      void finish();
    }
  }

  onUnmounted(() => {
    clearTimers();
    unlistenMoved?.();
    unlistenMoved = null;
    movedReady = null;
  });

  return { startDrag, onPointerUp };
}
