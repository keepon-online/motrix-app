<script setup lang="ts">
import { computed, watch, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useTaskStore } from '@/stores/task'
import { formatBytes, formatSpeed, formatDuration, calcProgress, calcRemainingTime, getTaskName, isBtTask } from '@/utils'
import TaskFiles from './TaskFiles.vue'
import TaskPeers from './TaskPeers.vue'
import TaskTrackers from './TaskTrackers.vue'
import TaskActivity from './TaskActivity.vue'

const { t } = useI18n()
const taskStore = useTaskStore()

const task = computed(() => taskStore.currentTask)
const visible = computed(() => taskStore.detailVisible)

// Auto-refresh task detail when drawer is open
const refreshTimer = ref<ReturnType<typeof setInterval> | null>(null)

watch(visible, (val) => {
  if (val && task.value) {
    // Start polling
    refreshTimer.value = setInterval(() => {
      if (task.value && (task.value.status === 'active' || task.value.status === 'waiting' || task.value.status === 'paused')) {
        taskStore.fetchTaskInfo(task.value.gid)
      }
    }, 1000)
  } else {
    // Stop polling
    if (refreshTimer.value) {
      clearInterval(refreshTimer.value)
      refreshTimer.value = null
    }
  }
})

onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
})

const taskName = computed(() => task.value ? getTaskName(task.value) : '')
const progress = computed(() => task.value ? calcProgress(task.value.totalLength, task.value.completedLength) : 0)
const totalSize = computed(() => task.value ? formatBytes(task.value.totalLength) : '0 B')
const completedSize = computed(() => task.value ? formatBytes(task.value.completedLength) : '0 B')
const uploadedSize = computed(() => task.value ? formatBytes(task.value.uploadLength) : '0 B')
const downloadSpeed = computed(() => task.value ? formatSpeed(task.value.downloadSpeed) : '0 B/s')
const uploadSpeed = computed(() => task.value ? formatSpeed(task.value.uploadSpeed) : '0 B/s')
const connections = computed(() => task.value?.connections || '0')
const seeders = computed(() => task.value?.numSeeders || '0')

const remainingTime = computed(() => {
  if (!task.value) return '--'
  const seconds = calcRemainingTime(
    task.value.totalLength,
    task.value.completedLength,
    task.value.downloadSpeed
  )
  return formatDuration(seconds)
})

const isBT = computed(() => task.value ? isBtTask(task.value) : false)

const taskUrl = computed(() => {
  if (!task.value) return ''
  if (isBT.value) return ''
  const file = task.value.files?.[0]
  if (file?.uris?.[0]?.uri) return file.uris[0].uri
  return ''
})

const btComment = computed(() => task.value?.bittorrent?.comment || '')
const btCreationDate = computed(() => {
  if (!task.value?.bittorrent?.creationDate) return ''
  return new Date(task.value.bittorrent.creationDate * 1000).toLocaleString()
})
const btMode = computed(() => task.value?.bittorrent?.mode || '')

const statusText = computed(() => {
  if (!task.value) return ''
  const statusMap: Record<string, string> = {
    active: t('task.downloading'),
    waiting: t('task.waiting'),
    paused: t('task.paused'),
    complete: t('task.completed'),
    error: t('task.error'),
    removed: t('task.removed'),
  }
  return statusMap[task.value.status] || task.value.status
})

const statusType = computed((): 'info' | 'success' | 'warning' | 'primary' | 'danger' => {
  if (!task.value) return 'info'
  const typeMap: Record<string, 'info' | 'success' | 'warning' | 'primary' | 'danger'> = {
    active: 'primary',
    waiting: 'warning',
    paused: 'info',
    complete: 'success',
    error: 'danger',
  }
  return typeMap[task.value.status] || 'info'
})

const isActive = computed(() => task.value?.status === 'active')
const isPaused = computed(() => task.value?.status === 'paused' || task.value?.status === 'waiting')
const isComplete = computed(() => task.value?.status === 'complete')

const firstFilePath = computed(() => {
  if (task.value?.files?.[0]?.path) return task.value.files[0].path
  return ''
})

async function handlePause() {
  if (!task.value) return
  try {
    await taskStore.pauseTask(task.value.gid)
  } catch {
    ElMessage.error(t('detail.pauseFailed'))
  }
}

async function handleResume() {
  if (!task.value) return
  try {
    await taskStore.resumeTask(task.value.gid)
  } catch {
    ElMessage.error(t('detail.resumeFailed'))
  }
}

async function handleRemove() {
  if (!task.value) return
  try {
    await ElMessageBox.confirm(t('task.removeConfirm'), {
      confirmButtonText: t('task.remove'),
      cancelButtonText: t('dialog.cancel'),
      type: 'warning',
    })
    await taskStore.removeTask(task.value.gid)
  } catch {
    // cancelled or error
  }
}

async function handleOpenFile() {
  if (!firstFilePath.value) return
  try {
    await invoke('open_file', { path: firstFilePath.value })
  } catch {
    ElMessage.error(t('task.failedOpenFile'))
  }
}

async function handleShowInFolder() {
  const path = firstFilePath.value || task.value?.dir
  if (!path) return
  try {
    await invoke('show_in_folder', { path })
  } catch {
    ElMessage.error(t('task.failedOpenFolder'))
  }
}

async function handleCopyLink() {
  if (!taskUrl.value) return
  try {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
    await writeText(taskUrl.value)
    ElMessage.success(t('task.linkCopied'))
  } catch {
    ElMessage.error(t('task.failedCopyLink'))
  }
}

function close() {
  taskStore.hideTaskDetail()
}
</script>

<template>
  <el-drawer
    :model-value="visible"
    :title="t('detail.title')"
    direction="rtl"
    size="480px"
    @close="close"
  >
    <template v-if="task">
      <div class="task-detail">
        <!-- Header -->
        <div class="detail-header">
          <h3 class="task-name">{{ taskName }}</h3>
          <el-tag :type="statusType" size="small">{{ statusText }}</el-tag>
        </div>

        <!-- Actions -->
        <div class="detail-actions">
          <el-button v-if="isActive" @click="handlePause" size="small">
            <el-icon><VideoPause /></el-icon>
            {{ t('task.pause') }}
          </el-button>
          <el-button v-if="isPaused" type="primary" @click="handleResume" size="small">
            <el-icon><VideoPlay /></el-icon>
            {{ t('task.resume') }}
          </el-button>
          <el-button v-if="isComplete && firstFilePath" @click="handleOpenFile" size="small">
            <el-icon><Document /></el-icon>
            {{ t('task.openFile') }}
          </el-button>
          <el-button v-if="firstFilePath || task.dir" @click="handleShowInFolder" size="small">
            <el-icon><FolderOpened /></el-icon>
            {{ t('task.showInFolder') }}
          </el-button>
          <el-button v-if="taskUrl" @click="handleCopyLink" size="small">
            <el-icon><CopyDocument /></el-icon>
            {{ t('task.copyLink') }}
          </el-button>
          <el-button type="danger" @click="handleRemove" size="small">
            <el-icon><Delete /></el-icon>
            {{ t('task.remove') }}
          </el-button>
        </div>

        <!-- Progress -->
        <div class="detail-section">
          <el-progress
            :percentage="progress"
            :status="task.status === 'complete' ? 'success' : task.status === 'error' ? 'exception' : undefined"
            :stroke-width="8"
          />
        </div>

        <!-- Stats -->
        <div class="detail-section">
          <div class="stats-grid">
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.size') }}</span>
              <span class="stat-value">{{ completedSize }} / {{ totalSize }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.progress') }}</span>
              <span class="stat-value">{{ progress }}%</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.downloadSpeed') }}</span>
              <span class="stat-value">{{ downloadSpeed }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.uploadSpeed') }}</span>
              <span class="stat-value">{{ uploadSpeed }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.uploaded') }}</span>
              <span class="stat-value">{{ uploadedSize }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.connections') }}</span>
              <span class="stat-value">{{ connections }}</span>
            </div>
            <div class="stat-item" v-if="isBT">
              <span class="stat-label">{{ t('detail.seeders') }}</span>
              <span class="stat-value">{{ seeders }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('detail.eta') }}</span>
              <span class="stat-value">{{ remainingTime }}</span>
            </div>
            <div class="stat-item full-width" v-if="taskUrl">
              <span class="stat-label">{{ t('detail.url') }}</span>
              <span class="stat-value path">{{ taskUrl }}</span>
            </div>
            <div class="stat-item full-width">
              <span class="stat-label">{{ t('detail.savePath') }}</span>
              <span class="stat-value path">{{ task.dir }}</span>
            </div>
            <div class="stat-item" v-if="btMode">
              <span class="stat-label">{{ t('detail.btMode') }}</span>
              <span class="stat-value">{{ btMode }}</span>
            </div>
            <div class="stat-item" v-if="btCreationDate">
              <span class="stat-label">{{ t('detail.created') }}</span>
              <span class="stat-value">{{ btCreationDate }}</span>
            </div>
            <div class="stat-item full-width" v-if="btComment">
              <span class="stat-label">{{ t('detail.comment') }}</span>
              <span class="stat-value path">{{ btComment }}</span>
            </div>
          </div>
        </div>

        <!-- Tabs -->
        <el-tabs class="detail-tabs">
          <el-tab-pane :label="t('detail.activity')" v-if="task.status === 'active'">
            <TaskActivity :download-speed="task.downloadSpeed" :upload-speed="task.uploadSpeed" />
          </el-tab-pane>
          <el-tab-pane :label="t('detail.files')">
            <TaskFiles :files="task.files" />
          </el-tab-pane>
          <el-tab-pane v-if="isBT" :label="t('detail.peers')">
            <TaskPeers :gid="task.gid" />
          </el-tab-pane>
          <el-tab-pane v-if="isBT" :label="t('detail.trackers')">
            <TaskTrackers :task="task" />
          </el-tab-pane>
        </el-tabs>

        <!-- Error Message -->
        <div v-if="task.status === 'error' && task.errorMessage" class="detail-section error-section">
          <el-alert
            :title="t('detail.error') + ': ' + task.errorCode"
            :description="task.errorMessage"
            type="error"
            :closable="false"
            show-icon
          />
        </div>
      </div>
    </template>
  </el-drawer>
</template>

<style lang="scss" scoped>
.task-detail {
  padding: 0 4px;
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;

  .task-name {
    flex: 1;
    font-size: 16px;
    font-weight: 600;
    color: var(--el-text-color-primary);
    margin: 0;
    word-break: break-all;
  }
}

.detail-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.detail-section {
  margin-bottom: 20px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;

  &.full-width {
    grid-column: 1 / -1;
  }

  .stat-label {
    font-size: 12px;
    color: var(--el-text-color-secondary);
  }

  .stat-value {
    font-size: 14px;
    color: var(--el-text-color-primary);
    font-weight: 500;

    &.path {
      font-size: 12px;
      word-break: break-all;
      font-weight: normal;
    }
  }
}

.detail-tabs {
  margin-top: 16px;
}

.error-section {
  margin-top: 16px;
}
</style>
