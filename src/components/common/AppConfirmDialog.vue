<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/common/AppButton.vue";
import AppModal from "@/components/common/AppModal.vue";
import { resolveConfirm, useAppConfirmState } from "@/composables/useAppConfirm";

const { t } = useI18n();
const state = useAppConfirmState();

const confirmLabel = computed(() => state.confirmLabel || t("common.confirm"));
const cancelLabel = computed(() => state.cancelLabel || t("common.cancel"));
</script>

<template>
  <AppModal :open="state.open" :title="state.title" @close="resolveConfirm(false)">
    <p class="message">{{ state.message }}</p>
    <div class="app-modal-actions">
      <AppButton variant="ghost" @click="resolveConfirm(false)">{{ cancelLabel }}</AppButton>
      <AppButton :variant="state.danger ? 'danger' : 'primary'" @click="resolveConfirm(true)">
        {{ confirmLabel }}
      </AppButton>
    </div>
  </AppModal>
</template>

<style scoped>
.message {
  margin: 0 0 16px;
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-text);
}
</style>
