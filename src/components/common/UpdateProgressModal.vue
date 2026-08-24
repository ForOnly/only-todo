<script setup lang="ts">
import type { UpdaterPhase } from "@/composables/useAppUpdater";

defineProps<{
  open: boolean;
  message: string;
  progress: number;
  phase: UpdaterPhase;
  indeterminate: boolean;
}>();
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="dialog" aria-modal="true">
      <div class="modal">
        <h2 class="title">{{ $t("updater.progressTitle") }}</h2>
        <p class="message" aria-live="polite">{{ message }}</p>
        <div
          class="progress-track"
          role="progressbar"
          :aria-busy="indeterminate"
          :aria-valuemin="0"
          :aria-valuemax="100"
          :aria-valuenow="indeterminate ? undefined : progress"
          :aria-label="$t('updater.progressTitle')"
        >
          <div
            class="progress-bar"
            :class="{ indeterminate }"
            :style="indeterminate ? undefined : { width: `${progress}%` }"
          />
        </div>
        <p class="percent">{{ indeterminate ? "—" : `${progress}%` }}</p>
        <p class="hint">{{ $t("updater.progressHint") }}</p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  backdrop-filter: blur(2px);
}

.modal {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  min-width: 360px;
  max-width: 90vw;
  padding: 24px;
  box-shadow: var(--shadow-md);
  color: var(--color-text);
}

.title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}

.message {
  margin: 0 0 16px;
  font-size: 14px;
  color: var(--color-muted);
}

.progress-track {
  height: 8px;
  border-radius: 999px;
  background: var(--color-border);
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: var(--color-accent, #3b82f6);
  transition: width 0.2s ease;
}

.progress-bar.indeterminate {
  width: 40%;
  animation: indeterminate 1.2s ease-in-out infinite;
}

@keyframes indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(350%);
  }
}

.percent {
  margin: 8px 0 0;
  font-size: 13px;
  text-align: right;
  color: var(--color-muted);
}

.hint {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--color-muted);
  line-height: 1.4;
}
</style>
