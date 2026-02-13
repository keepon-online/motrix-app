<script setup lang="ts">
import type { Task } from '@/types'
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { ElMessage } from 'element-plus'
import { formatBytes, formatSpeed, formatDuration, calcProgress, calcRemainingTime, getTaskName } from '@/utils'

const { t } = useI18n()

// Context menu state
const contextMenuVisible = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)

function showContextMenu(e: MouseEvent) {
  // Clamp position to prevent overflow
  const menuWidth = 200
  const menuHeight = 300
  contextMenuX.value = Math.min(e.clientX, window.innerWidth - menuWidth)
  contextMenuY.value = Math.min(e.clientY, window.innerHeight - menuHeight)
  contextMenuVisible.value = true
}

function hideContextMenu() {
  contextMenuVisible.value = false
}

function onContextMenuAction(action: string) {
  hideContextMenu()
  switch (action) {
    case 'pause': emit('pause'); break
    case 'resume': emit('resume'); break
    case 'retry': emit('retry'); break
    case 'openFile': openFile(); break
    case 'showInFolder': showInFolder(); break
    case 'copyLink': copyLink(); break
    case 'showDetail': emit('showDetail'); break
    case 'remove': emit('remove'); break
  }
}

function onClickOutside() {
  if (contextMenuVisible.value) {
    hideContextMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', onClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside)
})

const props = defineProps<{
  task: Task
  selected?: boolean
}>()

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void
  (e: 'select'): void
  (e: 'pause'): void
  (e: 'resume'): void
  (e: 'remove'): void
  (e: 'showDetail'): void
  (e: 'retry'): void
  (e: 'moveUp'): void
  (e: 'moveDown'): void
}>()

const taskName = computed(() => getTaskName(props.task))
const progress = computed(() => calcProgress(props.task.totalLength, props.task.completedLength))
const totalSize = computed(() => formatBytes(props.task.totalLength))
const completedSize = computed(() => formatBytes(props.task.completedLength))
const downloadSpeed = computed(() => formatSpeed(props.task.downloadSpeed))
const uploadSpeed = computed(() => formatSpeed(props.task.uploadSpeed))
const remainingTime = computed(() => {
  const seconds = calcRemainingTime(
    props.task.totalLength,
    props.task.completedLength,
    props.task.downloadSpeed
  )
  return formatDuration(seconds)
})

const statusClass = computed(() => {
  return `status-${props.task.status}`
})

const statusText = computed(() => {
  if (isSeedingTask.value) return t('task.seeding')
  const statusMap: Record<string, string> = {
    active: t('task.downloading'),
    waiting: t('task.waiting'),
    paused: t('task.paused'),
    complete: t('task.completed'),
    error: t('task.error'),
    removed: t('task.removed'),
  }
  if (props.task.status === 'error' && props.task.errorMessage) {
    return `${t('task.error')}: ${props.task.errorMessage}`
  }
  return statusMap[props.task.status] || props.task.status
})

const isActive = computed(() => props.task.status === 'active')
const isPaused = computed(() => props.task.status === 'paused' || props.task.status === 'waiting')
const isWaiting = computed(() => props.task.status === 'waiting' || props.task.status === 'paused')
const isComplete = computed(() => props.task.status === 'complete')
const isError = computed(() => props.task.status === 'error')
const isSeedingTask = computed(() => {
  return props.task.status === 'active'
    && props.task.totalLength !== '0'
    && props.task.completedLength === props.task.totalLength
})

const firstFilePath = computed(() => {
  if (props.task.files?.[0]?.path) return props.task.files[0].path
  return ''
})

const taskUrl = computed(() => {
  const file = props.task.files?.[0]
  if (file?.uris?.[0]?.uri) return file.uris[0].uri
  return ''
})

async function openFile() {
  if (!firstFilePath.value) return
  try {
    await invoke('open_file', { path: firstFilePath.value })
  } catch (error) {
    ElMessage.error(t('task.failedOpenFile'))
  }
}

async function showInFolder() {
  const path = firstFilePath.value || props.task.dir
  if (!path) return
  try {
    await invoke('show_in_folder', { path })
  } catch (error) {
    ElMessage.error(t('task.failedOpenFolder'))
  }
}

async function copyLink() {
  if (!taskUrl.value) return
  try {
    await writeText(taskUrl.value)
    ElMessage.success(t('task.linkCopied'))
  } catch (error) {
    ElMessage.error(t('task.failedCopyLink'))
  }
}
</script>

<template>
  <div
    class="task-item"
    :class="[statusClass, { selected }]"
    @click="emit('click', $event)"
    @dblclick="emit('showDetail')"
    @contextmenu.prevent="showContextMenu"
  >
    <div class="task-checkbox" @click.stop="emit('select')">
      <el-checkbox :model-value="selected" />
    </div>

    <div class="task-info">
      <div class="task-name" :title="taskName">{{ taskName }}</div>
      <div class="task-meta">
        <span class="task-size">{{ completedSize }} / {{ totalSize }}</span>
        <span v-if="isSeedingTask" class="task-speed">
          <el-icon><Upload /></el-icon> {{ uploadSpeed }}
        </span>
        <span v-else-if="isActive" class="task-speed">
          <el-icon><Download /></el-icon> {{ downloadSpeed }}
          <el-icon style="margin-left: 8px"><Upload /></el-icon> {{ uploadSpeed }}
        </span>
        <span v-if="isActive && !isSeedingTask" class="task-eta">{{ t('detail.eta') }}: {{ remainingTime }}</span>
        <span v-if="isSeedingTask" class="task-status">{{ statusText }}</span>
        <span v-if="!isActive" class="task-status" :title="statusText">{{ statusText }}</span>
      </div>
      <el-progress
        :percentage="progress"
        :status="isComplete ? 'success' : task.status === 'error' ? 'exception' : undefined"
        :stroke-width="4"
        :show-text="false"
      />
    </div>

    <div class="task-actions" @click.stop>
      <el-button
        v-if="isComplete && firstFilePath"
        circle
        size="small"
        @click="openFile"
        :title="t('task.openFile')"
      >
        <el-icon><Document /></el-icon>
      </el-button>
      <el-button
        v-if="firstFilePath || task.dir"
        circle
        size="small"
        @click="showInFolder"
        :title="t('task.showInFolder')"
      >
        <el-icon><FolderOpened /></el-icon>
      </el-button>
      <el-button
        v-if="taskUrl"
        circle
        size="small"
        @click="copyLink"
        :title="t('task.copyLink')"
      >
        <el-icon><CopyDocument /></el-icon>
      </el-button>
      <el-button
        v-if="isActive"
        circle
        size="small"
        @click="emit('pause')"
        :title="t('task.pause')"
      >
        <el-icon><VideoPause /></el-icon>
      </el-button>
      <el-button
        v-if="isPaused"
        circle
        size="small"
        type="primary"
        @click="emit('resume')"
        :title="t('task.resume')"
      >
        <el-icon><VideoPlay /></el-icon>
      </el-button>
      <el-button
        v-if="isWaiting"
        circle
        size="small"
        @click="emit('moveUp')"
        :title="t('task.moveUp')"
      >
        <el-icon><Top /></el-icon>
      </el-button>
      <el-button
        v-if="isWaiting"
        circle
        size="small"
        @click="emit('moveDown')"
        :title="t('task.moveDown')"
      >
        <el-icon><Bottom /></el-icon>
      </el-button>
      <el-button
        circle
        size="small"
        type="danger"
        @click="emit('remove')"
        :title="t('task.remove')"
      >
        <el-icon><Delete /></el-icon>
      </el-button>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenuVisible"
        class="context-menu"
        :style="{ left: contextMenuX + 'px', top: contextMenuY + 'px' }"
        @contextmenu.prevent
      >
        <div v-if="isActive" class="context-menu-item" @click="onContextMenuAction('pause')">
          <el-icon><VideoPause /></el-icon>
          <span>{{ t('task.pause') }}</span>
        </div>
        <div v-if="isPaused" class="context-menu-item" @click="onContextMenuAction('resume')">
          <el-icon><VideoPlay /></el-icon>
          <span>{{ t('task.resume') }}</span>
        </div>
        <div v-if="isError && taskUrl" class="context-menu-item" @click="onContextMenuAction('retry')">
          <el-icon><RefreshRight /></el-icon>
          <span>{{ t('task.retry') }}</span>
        </div>
        <div class="context-menu-divider" v-if="isActive || isPaused || isError" />
        <div v-if="isComplete && firstFilePath" class="context-menu-item" @click="onContextMenuAction('openFile')">
          <el-icon><Document /></el-icon>
          <span>{{ t('task.openFile') }}</span>
        </div>
        <div v-if="firstFilePath || task.dir" class="context-menu-item" @click="onContextMenuAction('showInFolder')">
          <el-icon><FolderOpened /></el-icon>
          <span>{{ t('task.showInFolder') }}</span>
        </div>
        <div v-if="taskUrl" class="context-menu-item" @click="onContextMenuAction('copyLink')">
          <el-icon><CopyDocument /></el-icon>
          <span>{{ t('task.copyLink') }}</span>
        </div>
        <div class="context-menu-divider" />
        <div class="context-menu-item" @click="onContextMenuAction('showDetail')">
          <el-icon><InfoFilled /></el-icon>
          <span>{{ t('detail.title') }}</span>
        </div>
        <div class="context-menu-divider" />
        <div class="context-menu-item danger" @click="onContextMenuAction('remove')">
          <el-icon><Delete /></el-icon>
          <span>{{ t('task.remove') }}</span>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style lang="scss" scoped>
.task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--el-bg-color);
  border-radius: 8px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: var(--el-fill-color-light);
  }

  &.selected {
    background: var(--el-color-primary-light-9);
  }

  &.status-error {
    border-left: 3px solid var(--el-color-danger);
  }

  &.status-complete {
    border-left: 3px solid var(--el-color-success);
  }
}

.task-checkbox {
  flex-shrink: 0;
}

.task-info {
  flex: 1;
  min-width: 0;
}

.task-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.task-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 8px;

  .task-speed,
  .task-eta {
    display: flex;
    align-items: center;
    gap: 4px;
  }
}

.task-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
</style>

<style lang="scss">
.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 180px;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color-light);
  border-radius: 6px;
  padding: 4px 0;
  box-shadow: var(--el-box-shadow-light);

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    font-size: 13px;
    color: var(--el-text-color-regular);
    cursor: pointer;
    transition: background 0.15s;

    &:hover {
      background: var(--el-fill-color-light);
      color: var(--el-text-color-primary);
    }

    &.danger {
      color: var(--el-color-danger);

      &:hover {
        background: var(--el-color-danger-light-9);
      }
    }
  }

  .context-menu-divider {
    height: 1px;
    background: var(--el-border-color-lighter);
    margin: 4px 0;
  }
}
</style>
