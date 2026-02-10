<script setup lang="ts">
import { computed } from 'vue'
import { useTaskStore } from '@/stores/task'
import { formatBytes, formatSpeed, formatDuration, calcProgress, calcRemainingTime, getTaskName, isBtTask } from '@/utils'
import TaskFiles from './TaskFiles.vue'
import TaskPeers from './TaskPeers.vue'
import TaskTrackers from './TaskTrackers.vue'
import TaskActivity from './TaskActivity.vue'

const taskStore = useTaskStore()

const task = computed(() => taskStore.currentTask)
const visible = computed(() => taskStore.detailVisible)

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
    active: 'Downloading',
    waiting: 'Waiting',
    paused: 'Paused',
    complete: 'Completed',
    error: 'Error',
    removed: 'Removed',
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

function close() {
  taskStore.hideTaskDetail()
}
</script>

<template>
  <el-drawer
    :model-value="visible"
    title="Task Details"
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
              <span class="stat-label">Size</span>
              <span class="stat-value">{{ completedSize }} / {{ totalSize }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Progress</span>
              <span class="stat-value">{{ progress }}%</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Download Speed</span>
              <span class="stat-value">{{ downloadSpeed }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Upload Speed</span>
              <span class="stat-value">{{ uploadSpeed }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Uploaded</span>
              <span class="stat-value">{{ uploadedSize }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Connections</span>
              <span class="stat-value">{{ connections }}</span>
            </div>
            <div class="stat-item" v-if="isBT">
              <span class="stat-label">Seeders</span>
              <span class="stat-value">{{ seeders }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">ETA</span>
              <span class="stat-value">{{ remainingTime }}</span>
            </div>
            <div class="stat-item full-width" v-if="taskUrl">
              <span class="stat-label">URL</span>
              <span class="stat-value path">{{ taskUrl }}</span>
            </div>
            <div class="stat-item full-width">
              <span class="stat-label">Save Path</span>
              <span class="stat-value path">{{ task.dir }}</span>
            </div>
            <div class="stat-item" v-if="btMode">
              <span class="stat-label">BT Mode</span>
              <span class="stat-value">{{ btMode }}</span>
            </div>
            <div class="stat-item" v-if="btCreationDate">
              <span class="stat-label">Created</span>
              <span class="stat-value">{{ btCreationDate }}</span>
            </div>
            <div class="stat-item full-width" v-if="btComment">
              <span class="stat-label">Comment</span>
              <span class="stat-value path">{{ btComment }}</span>
            </div>
          </div>
        </div>

        <!-- Tabs -->
        <el-tabs class="detail-tabs">
          <el-tab-pane label="Activity" v-if="task.status === 'active'">
            <TaskActivity :download-speed="task.downloadSpeed" :upload-speed="task.uploadSpeed" />
          </el-tab-pane>
          <el-tab-pane label="Files">
            <TaskFiles :files="task.files" />
          </el-tab-pane>
          <el-tab-pane v-if="isBT" label="Peers">
            <TaskPeers :gid="task.gid" />
          </el-tab-pane>
          <el-tab-pane v-if="isBT" label="Trackers">
            <TaskTrackers :task="task" />
          </el-tab-pane>
        </el-tabs>

        <!-- Error Message -->
        <div v-if="task.status === 'error' && task.errorMessage" class="detail-section error-section">
          <el-alert
            :title="'Error: ' + task.errorCode"
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
