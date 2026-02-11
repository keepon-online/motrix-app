<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaskStore } from '@/stores/task'

const { t } = useI18n()
const taskStore = useTaskStore()

const hasSelection = computed(() => taskStore.selectedGids.length > 0)
const selectionCount = computed(() => taskStore.selectedGids.length)
const totalCount = computed(() => taskStore.tasks.length)
const isAllSelected = computed(() =>
  totalCount.value > 0 && selectionCount.value === totalCount.value
)
const isIndeterminate = computed(() =>
  selectionCount.value > 0 && selectionCount.value < totalCount.value
)

const emit = defineEmits<{
  removeSelected: []
}>()

function toggleSelectAll() {
  if (isAllSelected.value) {
    taskStore.clearSelection()
  } else {
    taskStore.selectAllTasks()
  }
}

async function pauseSelected() {
  await taskStore.pauseSelectedTasks()
}

async function resumeSelected() {
  await taskStore.resumeSelectedTasks()
}

async function removeSelected() {
  emit('removeSelected')
}
</script>

<template>
  <div class="task-toolbar">
    <div class="toolbar-left">
      <el-checkbox
        :model-value="isAllSelected"
        :indeterminate="isIndeterminate"
        @change="toggleSelectAll"
      >
        <span v-if="hasSelection">
          {{ t('task.selected', { count: selectionCount }) }}
        </span>
        <span v-else>
          {{ t('task.selectAll') }}
        </span>
      </el-checkbox>
    </div>

    <div class="toolbar-right" v-if="hasSelection">
      <el-button-group>
        <el-button size="small" @click="resumeSelected">
          <el-icon><VideoPlay /></el-icon>
          {{ t('task.resume') }}
        </el-button>
        <el-button size="small" @click="pauseSelected">
          <el-icon><VideoPause /></el-icon>
          {{ t('task.pause') }}
        </el-button>
        <el-button size="small" type="danger" @click="removeSelected">
          <el-icon><Delete /></el-icon>
          {{ t('task.remove') }}
        </el-button>
      </el-button-group>
    </div>

    <div class="toolbar-right" v-else>
      <el-button-group>
        <el-button size="small" @click="taskStore.pauseAllTasks()" :title="t('task.pauseAll')">
          <el-icon><VideoPause /></el-icon>
        </el-button>
        <el-button size="small" @click="taskStore.resumeAllTasks()" :title="t('task.resumeAll')">
          <el-icon><VideoPlay /></el-icon>
        </el-button>
      </el-button-group>
      <span class="task-count">{{ t('task.tasks', { count: totalCount }) }}</span>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.task-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  margin-bottom: 12px;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-count {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
</style>
