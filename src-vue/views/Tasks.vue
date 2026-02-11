<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, h } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useTaskStore } from '@/stores/task'
import { ElMessageBox, ElCheckbox } from 'element-plus'
import { Search, SortUp, SortDown } from '@element-plus/icons-vue'
import TaskItem from '@/components/TaskItem.vue'
import TaskToolbar from '@/components/TaskToolbar.vue'
import TaskDetail from '@/components/TaskDetail.vue'
import AddTaskDialog from '@/components/AddTaskDialog.vue'

const { t } = useI18n()
const route = useRoute()
const taskStore = useTaskStore()

const addDialogVisible = ref(false)
let refreshInterval: number | null = null
let currentInterval = 1000

const pageTitle = computed(() => {
  const status = route.params.status as string
  if (status === 'stopped') return t('nav.completed')
  if (status === 'waiting') return t('nav.waiting')
  return t('nav.downloads')
})

// Fetch tasks based on route
const fetchTasks = () => {
  const status = route.params.status as string
  if (status === 'stopped') {
    taskStore.fetchTasks('stopped')
  } else if (status === 'waiting') {
    taskStore.fetchTasks('waiting')
  } else {
    taskStore.fetchTasks('active')
  }
}

// Dynamic polling interval based on active task count
function getPollingInterval(): number {
  const activeTasks = taskStore.tasks.filter(t => t.status === 'active').length
  if (activeTasks > 5) return 500
  if (activeTasks > 0) return 1000
  return 3000
}

function setupInterval() {
  if (refreshInterval) clearInterval(refreshInterval)
  currentInterval = getPollingInterval()
  refreshInterval = window.setInterval(() => {
    fetchTasks()
    taskStore.fetchGlobalStat()
    // Check if interval needs adjusting
    const newInterval = getPollingInterval()
    if (newInterval !== currentInterval) {
      setupInterval()
    }
  }, currentInterval)
}

onMounted(() => {
  fetchTasks()
  taskStore.fetchGlobalStat()
  setupInterval()
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

// Watch route changes
watch(() => route.params.status, () => {
  taskStore.clearSelection()
  fetchTasks()
})

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  // Ctrl/Cmd + A: Select all
  if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
    e.preventDefault()
    taskStore.selectAllTasks()
  }
  // Escape: Clear selection
  if (e.key === 'Escape') {
    taskStore.clearSelection()
    taskStore.hideTaskDetail()
  }
  // Delete: Remove selected
  if (e.key === 'Delete' && taskStore.selectedGids.length > 0) {
    confirmRemoveSelected()
  }
}

// Confirm remove single task
async function confirmRemoveTask(gid: string) {
  const deleteFiles = ref(false)
  try {
    await ElMessageBox({
      title: t('task.remove'),
      message: () => h('div', null, [
        h('p', null, t('task.removeConfirm')),
        h(ElCheckbox, {
          modelValue: deleteFiles.value,
          'onUpdate:modelValue': (val: string | number | boolean) => { deleteFiles.value = !!val },
        }, () => t('task.removeWithFiles')),
      ]),
      confirmButtonText: t('task.remove'),
      cancelButtonText: t('dialog.cancel'),
      showCancelButton: true,
      type: 'warning',
    })
    await taskStore.removeTask(gid, deleteFiles.value)
  } catch {
    // User cancelled
  }
}

// Confirm remove selected tasks
async function confirmRemoveSelected() {
  const count = taskStore.selectedGids.length
  if (count === 0) return
  const deleteFiles = ref(false)
  try {
    await ElMessageBox({
      title: t('task.remove'),
      message: () => h('div', null, [
        h('p', null, t('task.removeConfirmBatch', { count })),
        h(ElCheckbox, {
          modelValue: deleteFiles.value,
          'onUpdate:modelValue': (val: string | number | boolean) => { deleteFiles.value = !!val },
        }, () => t('task.removeWithFiles')),
      ]),
      confirmButtonText: t('task.remove'),
      cancelButtonText: t('dialog.cancel'),
      showCancelButton: true,
      type: 'warning',
    })
    await taskStore.removeSelectedTasks(deleteFiles.value)
  } catch {
    // User cancelled
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="tasks-view">
    <header class="tasks-header">
      <h2 class="tasks-title">
        {{ pageTitle }}
      </h2>
      <div class="tasks-actions">
        <el-input
          v-model="taskStore.searchQuery"
          :placeholder="t('task.search')"
          clearable
          size="small"
          style="width: 200px; margin-right: 8px"
          :prefix-icon="Search"
        />
        <el-select
          v-model="taskStore.sortField"
          size="small"
          style="width: 120px; margin-right: 8px"
        >
          <el-option :label="t('task.sortDefault')" value="default" />
          <el-option :label="t('task.sortName')" value="name" />
          <el-option :label="t('task.sortSize')" value="size" />
          <el-option :label="t('task.sortProgress')" value="progress" />
          <el-option :label="t('task.sortSpeed')" value="speed" />
        </el-select>
        <el-button
          v-if="taskStore.sortField !== 'default'"
          size="small"
          circle
          @click="taskStore.sortOrder = taskStore.sortOrder === 'asc' ? 'desc' : 'asc'"
          style="margin-right: 8px"
        >
          <el-icon>
            <SortUp v-if="taskStore.sortOrder === 'asc'" />
            <SortDown v-else />
          </el-icon>
        </el-button>
        <el-button
          v-if="route.params.status === 'stopped' && taskStore.tasks.length > 0"
          @click="taskStore.purgeTaskRecords()"
        >
          <el-icon><Delete /></el-icon>
          {{ t('task.clearRecords') }}
        </el-button>
        <el-button type="primary" @click="addDialogVisible = true">
          <el-icon><Plus /></el-icon>
          {{ t('task.add') }}
        </el-button>
      </div>
    </header>

    <TaskToolbar v-if="taskStore.filteredTasks.length > 0" @remove-selected="confirmRemoveSelected" />

    <div class="tasks-list">
      <template v-if="taskStore.filteredTasks.length > 0">
        <TaskItem
          v-for="task in taskStore.filteredTasks"
          :key="task.gid"
          :task="task"
          :selected="taskStore.selectedGids.includes(task.gid)"
          @click="taskStore.toggleSelectTask(task.gid)"
          @select="taskStore.toggleSelectTask(task.gid)"
          @pause="taskStore.pauseTask(task.gid)"
          @resume="taskStore.resumeTask(task.gid)"
          @remove="confirmRemoveTask(task.gid)"
          @show-detail="taskStore.showTaskDetail(task)"
        />
      </template>
      <el-empty v-else :description="t('task.noTasks')" :image-size="120">
        <el-button type="primary" @click="addDialogVisible = true">
          {{ t('task.addFirst') }}
        </el-button>
      </el-empty>
    </div>

    <AddTaskDialog v-model="addDialogVisible" />
    <TaskDetail />
  </div>
</template>

<style lang="scss" scoped>
.tasks-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tasks-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.tasks-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0;
}

.tasks-list {
  flex: 1;
  overflow-y: auto;
}
</style>
