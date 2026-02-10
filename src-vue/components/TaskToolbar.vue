<script setup lang="ts">
import { computed } from 'vue'
import { useTaskStore } from '@/stores/task'

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
  await taskStore.removeSelectedTasks()
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
          {{ selectionCount }} selected
        </span>
        <span v-else>
          Select all
        </span>
      </el-checkbox>
    </div>

    <div class="toolbar-right" v-if="hasSelection">
      <el-button-group>
        <el-button size="small" @click="resumeSelected">
          <el-icon><VideoPlay /></el-icon>
          Resume
        </el-button>
        <el-button size="small" @click="pauseSelected">
          <el-icon><VideoPause /></el-icon>
          Pause
        </el-button>
        <el-button size="small" type="danger" @click="removeSelected">
          <el-icon><Delete /></el-icon>
          Remove
        </el-button>
      </el-button-group>
    </div>

    <div class="toolbar-right" v-else>
      <el-button-group>
        <el-button size="small" @click="taskStore.pauseAllTasks()" title="Pause All">
          <el-icon><VideoPause /></el-icon>
        </el-button>
        <el-button size="small" @click="taskStore.resumeAllTasks()" title="Resume All">
          <el-icon><VideoPlay /></el-icon>
        </el-button>
      </el-button-group>
      <span class="task-count">{{ totalCount }} tasks</span>
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
