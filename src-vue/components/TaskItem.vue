<script setup lang="ts">
import type { Task } from '@/types'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { ElMessage } from 'element-plus'
import { formatBytes, formatSpeed, formatDuration, calcProgress, calcRemainingTime, getTaskName } from '@/utils'

const { t } = useI18n()

const props = defineProps<{
  task: Task
  selected?: boolean
}>()

const emit = defineEmits<{
  (e: 'click'): void
  (e: 'select'): void
  (e: 'pause'): void
  (e: 'resume'): void
  (e: 'remove'): void
  (e: 'showDetail'): void
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
  const statusMap: Record<string, string> = {
    active: t('task.downloading'),
    waiting: t('task.waiting'),
    paused: t('task.paused'),
    complete: t('task.completed'),
    error: t('task.error'),
    removed: t('task.removed'),
  }
  return statusMap[props.task.status] || props.task.status
})

const isActive = computed(() => props.task.status === 'active')
const isPaused = computed(() => props.task.status === 'paused' || props.task.status === 'waiting')
const isComplete = computed(() => props.task.status === 'complete')

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
    @click="emit('click')"
    @dblclick="emit('showDetail')"
  >
    <div class="task-checkbox" @click.stop="emit('select')">
      <el-checkbox :model-value="selected" />
    </div>

    <div class="task-info">
      <div class="task-name" :title="taskName">{{ taskName }}</div>
      <div class="task-meta">
        <span class="task-size">{{ completedSize }} / {{ totalSize }}</span>
        <span v-if="isActive" class="task-speed">
          <el-icon><Download /></el-icon> {{ downloadSpeed }}
          <el-icon style="margin-left: 8px"><Upload /></el-icon> {{ uploadSpeed }}
        </span>
        <span v-if="isActive" class="task-eta">ETA: {{ remainingTime }}</span>
        <span v-if="!isActive" class="task-status">{{ statusText }}</span>
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
        circle
        size="small"
        type="danger"
        @click="emit('remove')"
        :title="t('task.remove')"
      >
        <el-icon><Delete /></el-icon>
      </el-button>
    </div>
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
