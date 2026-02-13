<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, h, inject, type Ref } from 'vue'
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
const showAddDialog = inject<Ref<boolean>>('showAddDialog')
let refreshInterval: number | null = null

// Sync with App.vue's showAddDialog (for deep-link/CLI URL handling)
if (showAddDialog) {
  watch(showAddDialog, (val) => {
    if (val) {
      addDialogVisible.value = true
      showAddDialog.value = false
    }
  })
}
let currentInterval = 1000
let lastSelectedIndex = -1

// DragSelect state
const tasksListRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const dragRect = ref({ x: 0, y: 0, w: 0, h: 0 })
let dragStart = { x: 0, y: 0 }
let dragScrollTop = 0

const pageTitle = computed(() => {
  const status = route.params.status as string
  if (status === 'stopped') return t('nav.stopped')
  if (status === 'waiting') return t('nav.waiting')
  if (status === 'seeding') return t('nav.seeding')
  return t('nav.downloads')
})

// Fetch tasks based on route
const fetchTasks = () => {
  const status = route.params.status as string
  if (status === 'stopped') {
    taskStore.fetchTasks('stopped')
  } else if (status === 'waiting') {
    taskStore.fetchTasks('waiting')
  } else if (status === 'seeding') {
    taskStore.fetchTasks('seeding')
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

// Shift+Click range selection
function handleTaskClick(e: MouseEvent, gid: string, index: number) {
  if (e.shiftKey && lastSelectedIndex >= 0) {
    const tasks = taskStore.filteredTasks
    const start = Math.min(lastSelectedIndex, index)
    const end = Math.max(lastSelectedIndex, index)
    for (let i = start; i <= end; i++) {
      taskStore.selectTask(tasks[i].gid)
    }
  } else {
    taskStore.toggleSelectTask(gid)
    lastSelectedIndex = index
  }
}

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

// DragSelect handlers
function onDragStart(e: MouseEvent) {
  // Only start drag on left button, not on interactive elements
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('button, input, .el-select, .el-checkbox, .el-button')) return

  const list = tasksListRef.value
  if (!list) return

  const listRect = list.getBoundingClientRect()
  dragStart = { x: e.clientX - listRect.left, y: e.clientY - listRect.top + list.scrollTop }
  dragScrollTop = list.scrollTop
  isDragging.value = false

  const onMove = (me: MouseEvent) => {
    const dx = me.clientX - (listRect.left + dragStart.x - dragScrollTop + list.scrollTop)
    const dy = me.clientY - (listRect.top + dragStart.y - dragScrollTop + list.scrollTop)
    if (!isDragging.value && Math.abs(dx) + Math.abs(dy) > 5) {
      isDragging.value = true
      if (!me.shiftKey) taskStore.clearSelection()
    }
    if (!isDragging.value) return

    const curX = me.clientX - listRect.left
    const curY = me.clientY - listRect.top + list.scrollTop
    const x = Math.min(dragStart.x, curX)
    const y = Math.min(dragStart.y, curY)
    const w = Math.abs(curX - dragStart.x)
    const h = Math.abs(curY - dragStart.y)
    dragRect.value = { x, y, w, h }

    // Hit-test task items
    const items = list.querySelectorAll('[data-gid]')
    items.forEach((item) => {
      const el = item as HTMLElement
      const gid = el.dataset.gid!
      const itemTop = el.offsetTop
      const itemBottom = itemTop + el.offsetHeight
      const overlaps = !(itemBottom < y || itemTop > y + h)
      if (overlaps) {
        taskStore.selectTask(gid)
      }
    })
  }

  const onUp = () => {
    isDragging.value = false
    dragRect.value = { x: 0, y: 0, w: 0, h: 0 }
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }

  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
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

async function retryTask(task: import('@/types').Task) {
  try {
    await taskStore.retryTask(task)
  } catch (e) {
    console.error('Failed to retry task:', e)
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

    <div class="tasks-list" ref="tasksListRef" @mousedown="onDragStart">
      <div
        v-if="isDragging"
        class="drag-select-rect"
        :style="{
          left: dragRect.x + 'px',
          top: dragRect.y + 'px',
          width: dragRect.w + 'px',
          height: dragRect.h + 'px',
        }"
      />
      <template v-if="taskStore.filteredTasks.length > 0">
        <TaskItem
          v-for="(task, index) in taskStore.filteredTasks"
          :key="task.gid"
          :task="task"
          :selected="taskStore.selectedGids.includes(task.gid)"
          :data-gid="task.gid"
          @click="(e: MouseEvent) => handleTaskClick(e, task.gid, index)"
          @select="taskStore.toggleSelectTask(task.gid)"
          @pause="taskStore.pauseTask(task.gid)"
          @resume="taskStore.resumeTask(task.gid)"
          @remove="confirmRemoveTask(task.gid)"
          @retry="retryTask(task)"
          @show-detail="taskStore.showTaskDetail(task)"
          @move-up="taskStore.moveTaskUp(task.gid)"
          @move-down="taskStore.moveTaskDown(task.gid)"
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
  position: relative;
  user-select: none;
}

.drag-select-rect {
  position: absolute;
  border: 1px solid var(--el-color-primary);
  background: rgba(64, 158, 255, 0.1);
  pointer-events: none;
  z-index: 10;
}
</style>
