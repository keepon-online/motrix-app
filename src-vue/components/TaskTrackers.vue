<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Task } from '@/types'

const { t } = useI18n()

const props = defineProps<{
  task: Task
}>()

const trackers = computed(() => {
  if (!props.task.bittorrent?.announceList) return []

  const list: string[] = []
  for (const tier of props.task.bittorrent.announceList) {
    for (const tracker of tier) {
      if (tracker && !list.includes(tracker)) {
        list.push(tracker)
      }
    }
  }
  return list
})

function getTrackerHost(url: string): string {
  try {
    const parsed = new URL(url)
    return parsed.host
  } catch {
    return url
  }
}

function getTrackerProtocol(url: string): string {
  try {
    const parsed = new URL(url)
    return parsed.protocol.replace(':', '').toUpperCase()
  } catch {
    return 'Unknown'
  }
}
</script>

<template>
  <div class="task-trackers">
    <div v-if="trackers.length === 0" class="empty">
      <el-empty :description="t('detail.noTrackers')" :image-size="60" />
    </div>
    <div v-else class="tracker-list">
      <div
        v-for="(tracker, index) in trackers"
        :key="index"
        class="tracker-item"
      >
        <el-tag size="small" type="info" class="tracker-protocol">
          {{ getTrackerProtocol(tracker) }}
        </el-tag>
        <span class="tracker-url" :title="tracker">
          {{ getTrackerHost(tracker) }}
        </span>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.task-trackers {
  max-height: 300px;
  overflow-y: auto;
}

.tracker-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.tracker-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
}

.tracker-protocol {
  flex-shrink: 0;
}

.tracker-url {
  flex: 1;
  font-size: 12px;
  color: var(--el-text-color-regular);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
