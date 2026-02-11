import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Task, GlobalStat, AddTaskOptions } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export type TaskListType = 'active' | 'waiting' | 'stopped'

export const useTaskStore = defineStore('task', () => {
  // State
  const tasks = ref<Task[]>([])
  const currentListType = ref<TaskListType>('active')
  const selectedGids = ref<string[]>([])
  const currentTask = ref<Task | null>(null)
  const detailVisible = ref(false)
  const globalStat = ref<GlobalStat | null>(null)
  const loading = ref(false)

  // Getters
  const activeTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'active' || t.status === 'waiting')
  )

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'complete')
  )

  const downloadSpeed = computed(() => {
    return tasks.value.reduce((sum, t) => sum + parseInt(t.downloadSpeed || '0'), 0)
  })

  const uploadSpeed = computed(() => {
    return tasks.value.reduce((sum, t) => sum + parseInt(t.uploadSpeed || '0'), 0)
  })

  // Actions
  async function fetchTasks(type?: TaskListType) {
    const listType = type ?? currentListType.value

    try {
      const result = await invoke<Task[]>('get_task_list', { taskType: listType })
      tasks.value = result
      currentListType.value = listType

      // Update selected gids (remove non-existent)
      const gids = new Set(result.map((t) => t.gid))
      selectedGids.value = selectedGids.value.filter((gid) => gids.has(gid))
    } catch (error) {
      console.error('Failed to fetch tasks:', error)
    }
  }

  async function fetchGlobalStat() {
    try {
      globalStat.value = await invoke<GlobalStat>('get_global_stat')
    } catch (error) {
      console.error('Failed to fetch global stat:', error)
    }
  }

  async function addUri(uris: string[], options?: AddTaskOptions) {
    try {
      await invoke('add_uri', { uris, options })
      await fetchTasks()
    } catch (error) {
      console.error('Failed to add URI:', error)
      throw error
    }
  }

  async function addTorrent(torrent: string, options?: AddTaskOptions) {
    try {
      await invoke('add_torrent', { torrent, options })
      await fetchTasks()
    } catch (error) {
      console.error('Failed to add torrent:', error)
      throw error
    }
  }

  async function pauseTask(gid: string) {
    try {
      // Check if BT task, use forcePause for BT
      const task = tasks.value.find(t => t.gid === gid)
      const isBT = task?.bittorrent !== undefined
      await invoke(isBT ? 'force_pause_task' : 'pause_task', { gid })
      await fetchTasks()
    } catch (error) {
      console.error('Failed to pause task:', error)
      throw error
    }
  }

  async function resumeTask(gid: string) {
    try {
      await invoke('resume_task', { gid })
      await fetchTasks()
    } catch (error) {
      console.error('Failed to resume task:', error)
      throw error
    }
  }

  async function removeTask(gid: string) {
    try {
      await invoke('remove_task', { gid })
      if (currentTask.value?.gid === gid) {
        currentTask.value = null
        detailVisible.value = false
      }
      await fetchTasks()
    } catch (error) {
      console.error('Failed to remove task:', error)
      throw error
    }
  }

  async function toggleTask(task: Task) {
    if (task.status === 'active') {
      await pauseTask(task.gid)
    } else if (task.status === 'paused' || task.status === 'waiting') {
      await resumeTask(task.gid)
    }
  }

  async function fetchTaskInfo(gid: string) {
    try {
      currentTask.value = await invoke<Task>('get_task_info', { gid })
    } catch (error) {
      console.error('Failed to fetch task info:', error)
    }
  }

  function showTaskDetail(task: Task) {
    currentTask.value = task
    detailVisible.value = true
  }

  function hideTaskDetail() {
    detailVisible.value = false
  }

  function selectTask(gid: string) {
    if (!selectedGids.value.includes(gid)) {
      selectedGids.value.push(gid)
    }
  }

  function deselectTask(gid: string) {
    selectedGids.value = selectedGids.value.filter((g) => g !== gid)
  }

  function toggleSelectTask(gid: string) {
    if (selectedGids.value.includes(gid)) {
      deselectTask(gid)
    } else {
      selectTask(gid)
    }
  }

  function selectAllTasks() {
    selectedGids.value = tasks.value.map((t) => t.gid)
  }

  function clearSelection() {
    selectedGids.value = []
  }

  // Batch operations
  async function pauseSelectedTasks() {
    for (const gid of selectedGids.value) {
      await pauseTask(gid)
    }
  }

  async function resumeSelectedTasks() {
    for (const gid of selectedGids.value) {
      await resumeTask(gid)
    }
  }

  async function removeSelectedTasks() {
    for (const gid of selectedGids.value) {
      await removeTask(gid)
    }
    clearSelection()
  }

  async function pauseAllTasks() {
    try {
      await invoke('pause_all_tasks')
      await fetchTasks()
    } catch (error) {
      console.error('Failed to pause all tasks:', error)
      throw error
    }
  }

  async function resumeAllTasks() {
    try {
      await invoke('resume_all_tasks')
      await fetchTasks()
    } catch (error) {
      console.error('Failed to resume all tasks:', error)
      throw error
    }
  }

  async function removeTaskRecord(gid: string) {
    try {
      await invoke('remove_task_record', { gid })
      await fetchTasks()
    } catch (error) {
      console.error('Failed to remove task record:', error)
      throw error
    }
  }

  async function purgeTaskRecords() {
    try {
      await invoke('purge_task_records')
      await fetchTasks('stopped')
    } catch (error) {
      console.error('Failed to purge task records:', error)
      throw error
    }
  }

  return {
    // State
    tasks,
    currentListType,
    selectedGids,
    currentTask,
    detailVisible,
    globalStat,
    loading,
    // Getters
    activeTasks,
    completedTasks,
    downloadSpeed,
    uploadSpeed,
    // Actions
    fetchTasks,
    fetchGlobalStat,
    addUri,
    addTorrent,
    pauseTask,
    resumeTask,
    removeTask,
    toggleTask,
    fetchTaskInfo,
    showTaskDetail,
    hideTaskDetail,
    selectTask,
    deselectTask,
    toggleSelectTask,
    selectAllTasks,
    clearSelection,
    pauseSelectedTasks,
    resumeSelectedTasks,
    removeSelectedTasks,
    pauseAllTasks,
    resumeAllTasks,
    removeTaskRecord,
    purgeTaskRecords,
  }
})
