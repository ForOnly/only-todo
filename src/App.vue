<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";

import type { CreateTodoDto } from "@/api/types";
import { getSettings, updateSettings } from "@/api/settings";
import SettingsModal from "@/components/settings/SettingsModal.vue";
import AppHeader from "@/components/layout/AppHeader.vue";
import FilterSidebar from "@/components/layout/FilterSidebar.vue";
import CreateTodoModal from "@/components/todo/CreateTodoModal.vue";
import TodoDetailDrawer from "@/components/todo/TodoDetailDrawer.vue";
import TodoList from "@/components/todo/TodoList.vue";
import { TAURI_EVENTS } from "@/constants/events";
import { useReminders, useTodoDetail } from "@/composables/useTodoDetail";
import { useTodos } from "@/composables/useTodos";
import { formatErrorMessage } from "@/utils/error";
import {
  loadFilterCollapsed,
  loadFilterPinned,
  saveFilterCollapsed,
  saveFilterPinned,
} from "@/utils/filterPrefs";

const {
  todos,
  total,
  page,
  pageSize,
  loading,
  error: listError,
  keyword,
  selectedStatuses,
  selectedPriorities,
  selectedId,
  fetchTodos,
  createTodo,
  selectTodo,
  toggleStatusFilter,
  togglePriorityFilter,
  search,
  goToPage,
} = useTodos();

const {
  reminders,
  loading: remindersLoading,
  error: remindersError,
  addReminder,
  removeReminder,
} = useReminders(selectedId);

const {
  detail,
  editTitle,
  editDescription,
  editPriority,
  saving,
  error: detailError,
  save,
  toggleComplete,
  remove,
} = useTodoDetail(selectedId, fetchTodos);

const settingsOpen = ref(false);
const createOpen = ref(false);
const notificationEnabled = ref(true);
const createModalRef = ref<InstanceType<typeof CreateTodoModal> | null>(null);
const reminderError = ref<string | null>(null);

const filterCollapsed = ref(loadFilterCollapsed());
const filterPinned = ref(loadFilterPinned());

const detailOpen = computed(() => selectedId.value !== null);

onMounted(async () => {
  await fetchTodos();

  if (filterPinned.value) {
    filterCollapsed.value = false;
  }

  try {
    const settings = await getSettings();
    notificationEnabled.value = settings.notificationEnabled;
  } catch {
    // 首次启动时 settings 表可能尚未写入种子数据，忽略即可
  }

  await listen<string>(TAURI_EVENTS.NAVIGATE_TO_TODO, (event) => {
    void selectTodo(event.payload);
  });

  await listen(TAURI_EVENTS.CREATE_TODO, () => {
    createOpen.value = true;
  });

  await listen(TAURI_EVENTS.OPEN_SETTINGS, () => {
    settingsOpen.value = true;
  });
});

function openCreateModal() {
  createOpen.value = true;
}

function closeDetail() {
  selectedId.value = null;
}

async function handleSave() {
  const ok = await save();
  if (ok) {
    closeDetail();
  }
}

async function handleRemove() {
  await remove();
  closeDetail();
}

function toggleFilter() {
  if (filterPinned.value) return;
  filterCollapsed.value = !filterCollapsed.value;
  saveFilterCollapsed(filterCollapsed.value);
}

function toggleFilterPin() {
  filterPinned.value = !filterPinned.value;
  saveFilterPinned(filterPinned.value);
  if (filterPinned.value) {
    filterCollapsed.value = false;
    saveFilterCollapsed(false);
  }
}

async function handleCreateSubmit(dto: CreateTodoDto) {
  try {
    await createTodo(dto);
    createOpen.value = false;
  } catch (err) {
    createModalRef.value?.setError(formatErrorMessage(err));
  }
}

async function handleAddReminder(datetime: string) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await addReminder(selectedId.value, datetime);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleRemoveReminder(id: string) {
  if (!selectedId.value) return;
  reminderError.value = null;
  try {
    await removeReminder(id, selectedId.value);
  } catch (err) {
    reminderError.value = formatErrorMessage(err);
  }
}

async function handleNotificationToggle(enabled: boolean) {
  notificationEnabled.value = enabled;
  await updateSettings({ notificationEnabled: enabled });
}
</script>

<template>
  <div class="app">
    <AppHeader
      v-model:keyword="keyword"
      :filter-collapsed="filterCollapsed"
      :filter-pinned="filterPinned"
      @search="search"
      @create="openCreateModal"
      @settings="settingsOpen = true"
      @toggle-filter="toggleFilter"
      @toggle-filter-pin="toggleFilterPin"
    />

    <div class="body">
      <FilterSidebar
        :collapsed="filterCollapsed"
        :selected-statuses="selectedStatuses"
        :selected-priorities="selectedPriorities"
        @toggle-status="toggleStatusFilter"
        @toggle-priority="togglePriorityFilter"
      />

      <main class="main-content">
        <TodoList
          :todos="todos"
          :selected-id="selectedId"
          :loading="loading"
          :total="total"
          :page="page"
          :page-size="pageSize"
          @select="selectTodo"
          @prev-page="goToPage(page - 1)"
          @next-page="goToPage(page + 1)"
        />
      </main>
    </div>

    <p v-if="listError" class="global-error">{{ listError }}</p>

    <TodoDetailDrawer
      :open="detailOpen"
      :detail="detail"
      v-model:edit-title="editTitle"
      v-model:edit-description="editDescription"
      v-model:edit-priority="editPriority"
      :saving="saving"
      :error="detailError"
      :reminders="reminders"
      :reminders-loading="remindersLoading"
      :reminders-error="remindersError ?? reminderError"
      @close="closeDetail"
      @save="handleSave"
      @toggle-complete="toggleComplete"
      @remove="handleRemove"
      @add-reminder="handleAddReminder"
      @remove-reminder="handleRemoveReminder"
    />

    <CreateTodoModal
      ref="createModalRef"
      :open="createOpen"
      @close="createOpen = false"
      @submit="handleCreateSubmit"
    />

    <SettingsModal
      :open="settingsOpen"
      :notification-enabled="notificationEnabled"
      @close="settingsOpen = false"
      @update:notification-enabled="handleNotificationToggle"
    />
  </div>
</template>

<style>
* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  height: 100%;
}

body {
  font-family: Inter, system-ui, sans-serif;
  color: #111827;
  background: #f3f4f6;
}
</style>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.main-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.global-error {
  margin: 0;
  padding: 8px 16px;
  background: #fef2f2;
  color: #dc2626;
}
</style>
