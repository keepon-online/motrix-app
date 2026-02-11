<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { TaskFile } from '@/types'
import { formatBytes, calcProgress } from '@/utils'

const { t } = useI18n()

const props = defineProps<{
  files: TaskFile[]
}>()

const fileList = computed(() => {
  return props.files.map((file) => {
    const name = file.path.split('/').pop() || file.path.split('\\').pop() || 'Unknown'
    const size = formatBytes(file.length)
    const completed = formatBytes(file.completedLength)
    const progress = calcProgress(file.length, file.completedLength)
    const selected = file.selected === 'true'

    return {
      ...file,
      name,
      size,
      completed,
      progress,
      selected,
    }
  })
})
</script>

<template>
  <div class="task-files">
    <div v-if="fileList.length === 0" class="empty">
      <el-empty :description="t('detail.noFiles')" :image-size="60" />
    </div>
    <div v-else class="file-list">
      <div
        v-for="file in fileList"
        :key="file.index"
        class="file-item"
        :class="{ disabled: !file.selected }"
      >
        <div class="file-info">
          <el-icon class="file-icon">
            <Document />
          </el-icon>
          <div class="file-details">
            <span class="file-name" :title="file.name">{{ file.name }}</span>
            <span class="file-size">{{ file.completed }} / {{ file.size }}</span>
          </div>
        </div>
        <div class="file-progress">
          <el-progress
            :percentage="file.progress"
            :stroke-width="4"
            :show-text="false"
          />
          <span class="progress-text">{{ file.progress }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.task-files {
  max-height: 300px;
  overflow-y: auto;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-item {
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 6px;

  &.disabled {
    opacity: 0.5;
  }
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.file-icon {
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.file-name {
  font-size: 13px;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.file-progress {
  display: flex;
  align-items: center;
  gap: 8px;

  .el-progress {
    flex: 1;
  }

  .progress-text {
    font-size: 11px;
    color: var(--el-text-color-secondary);
    width: 36px;
    text-align: right;
  }
}
</style>
