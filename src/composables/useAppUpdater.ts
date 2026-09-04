import { computed, ref } from "vue";
import type { Composer } from "vue-i18n";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

import { confirm } from "@/composables/useAppConfirm";
import { MESSAGE_KEYS, useMessage } from "@/composables/useMessage";

const SKIPPED_VERSION_KEY = "only-todo:skipped-update-version";

export type UpdaterPhase = "idle" | "downloading" | "installing";

function resetUpdateUi(
  updating: { value: boolean },
  progress: { value: number },
  phase: { value: UpdaterPhase },
  statusMessage: { value: string },
): void {
  updating.value = false;
  progress.value = 0;
  phase.value = "idle";
  statusMessage.value = "";
}

export function useAppUpdater(options: {
  t: Composer["t"];
  flushBeforeInstall: () => Promise<boolean>;
}) {
  const { t, flushBeforeInstall } = options;
  const { info, error: showError } = useMessage();

  const updating = ref(false);
  const checking = ref(false);
  const progress = ref(0);
  const phase = ref<UpdaterPhase>("idle");
  const statusMessage = ref("");
  const indeterminate = ref(false);

  let flowInFlight = false;

  const showIndeterminate = computed(() => indeterminate.value && phase.value === "downloading");

  function isUpdaterEnabled(): boolean {
    return !import.meta.env.DEV;
  }

  function getSkippedVersion(): string | null {
    try {
      return localStorage.getItem(SKIPPED_VERSION_KEY);
    } catch {
      return null;
    }
  }

  function skipVersion(version: string): void {
    try {
      localStorage.setItem(SKIPPED_VERSION_KEY, version);
    } catch {
      // 忽略存储失败
    }
  }

  async function installUpdate(update: Update): Promise<void> {
    checking.value = false;
    updating.value = true;
    progress.value = 0;
    phase.value = "downloading";
    indeterminate.value = false;
    statusMessage.value = t("updater.downloading");

    let downloaded = 0;
    let total = 0;

    await update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? 0;
        progress.value = 0;
        indeterminate.value = total <= 0;
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        if (total > 0) {
          indeterminate.value = false;
          progress.value = Math.min(100, Math.round((downloaded / total) * 100));
        }
      } else if (event.event === "Finished") {
        indeterminate.value = false;
        phase.value = "installing";
        statusMessage.value = t("updater.installing");
        progress.value = 100;
      }
    });

    try {
      await relaunch();
    } catch (err) {
      resetUpdateUi(updating, progress, phase, statusMessage);
      indeterminate.value = false;
      throw err;
    }
  }

  async function runUpdateFlow(manual: boolean): Promise<void> {
    if (flowInFlight) return;

    if (!isUpdaterEnabled()) {
      if (manual) {
        info(t("updater.devOnly"), { key: MESSAGE_KEYS.updater });
      }
      return;
    }

    flowInFlight = true;
    checking.value = true;

    try {
      const update = await check();
      if (!update) {
        if (manual) {
          info(t("updater.alreadyLatest"), { key: MESSAGE_KEYS.updater });
        }
        return;
      }

      if (!manual && getSkippedVersion() === update.version) {
        return;
      }

      const notes = update.body?.trim();
      const message = notes
        ? t("updater.confirmMessageWithNotes", { version: update.version, notes })
        : t("updater.confirmMessage", { version: update.version });

      const accepted = await confirm({
        title: t("updater.confirmTitle", { version: update.version }),
        message,
        confirmLabel: t("updater.confirm"),
        cancelLabel: t("updater.later"),
      });

      if (!accepted) {
        skipVersion(update.version);
        return;
      }

      if (!(await flushBeforeInstall())) {
        showError(t("updater.flushFailed"), { key: MESSAGE_KEYS.updater });
        return;
      }

      await installUpdate(update);
    } catch (err) {
      resetUpdateUi(updating, progress, phase, statusMessage);
      indeterminate.value = false;

      const detail = err instanceof Error ? err.message : String(err);
      if (manual) {
        showError(t("updater.failed", { detail }), { key: MESSAGE_KEYS.updater });
      } else {
        console.warn("[updater] auto check failed:", detail);
      }
    } finally {
      checking.value = false;
      flowInFlight = false;
    }
  }

  return {
    updating,
    checking,
    progress,
    phase,
    statusMessage,
    indeterminate: showIndeterminate,
    runUpdateFlow,
  };
}
