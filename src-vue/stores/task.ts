import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Task, GlobalStat, AddTaskOptions } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export type TaskListType = 'active' | 'waiting' | 'stopped'
export type SortField = 'name' | 'size' | 'progress' | 'speed' | 'default'
export type SortOrder = 'asc' | 'desc'

export const useTaskStore = defineStore('task', () => {
  // State
  const tasks = ref<Task[]>([])
  const currentListType = ref<TaskListType>('active')
  const selectedGids = ref<string[]>([])
  const currentTask = ref<Task | null>(null)
  const detailVisible = ref(false)
  const globalStat = ref<GlobalStat | null>(null)
  const loading = ref(false)
  const searchQuery = ref('')
  const sortField = ref<SortField>('default')
  const sortOrder = ref<SortOrder>('asc')

  // Getters
  const filteredTasks = computed(() => {
    let result = [...tasks.value]

    // Search filter
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.trim().toLowerCase()
      result = result.filter(t => {
        // Match by file name
        const fileName = t.bittorrent?.info?.name
          || t.files?.[0]?.path?.split('/').pop()?.split('\\').pop()
          || ''
        if (fileName.toLowerCase().includes(query)) return true
        // Match by URL
        const uri = t.files?.[0]?.uris?.[0]?.uri || ''
        if (uri.toLowerCase().includes(query)) return true
        return false
      })
    }

    // Sort
    if (sortField.value !== 'default') {
      result.sort((a, b) => {
        let cmp = 0
        switch (sortField.value) {
          case 'name': {
            const nameA = (a.bittorrent?.info?.name || a.files?.[0]?.path || '').toLowerCase()
            const nameB = (b.bittorrent?.info?.name || b.files?.[0]?.path || '').toLowerCase()
            cmp = nameA.localeCompare(nameB)
            break
          }
          case 'size':
            cmp = parseInt(a.totalLength || '0') - parseInt(b.totalLength || '0')
            break
          case 'progress': {
            const pA = parseInt(a.totalLength || '0') > 0
              ? parseInt(a.completedLength || '0') / parseInt(a.totalLength || '1') : 0
            const pB = parseInt(b.totalLength || '0') > 0
              ? parseInt(b.completedLength || '0') / parseInt(b.totalLength || '1') : 0
            cmp = pA - pB
            break
          }
          case 'speed':
            cmp = parseInt(a.downloadSpeed || '0') - parseInt(b.downloadSpeed || '0')
            break
        }
        return sortOrder.value === 'desc' ? -cmp : cmp
      })
    }

    return result
  })

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

  async function removeTask(gid: string, deleteFiles = false) {
    try {
      const task = tasks.value.find(t => t.gid === gid)

      // Collect file paths before removing the task record
      if (deleteFiles && task) {
        const filePaths = task.files
          ?.map(f => f.path)
          .filter(p => p && p.length > 0) || []
        if (filePaths.length > 0) {
          try {
            await invoke('delete_task_files', { filePaths })
          } catch (e) {
            console.warn('Failed to delete some files:', e)
          }
        }
      }

      if (task && (task.status === 'complete' || task.status === 'error' || task.status === 'removed')) {
        // Completed/error/removed tasks must use removeDownloadResult
        await invoke('remove_task_record', { gid })
      } else {
        // Active/waiting/paused tasks use force remove then clean up the record
        await invoke('force_remove_task', { gid })
        // Also remove the download result record so it doesn't linger in stopped list
        try {
          await invoke('remove_task_record', { gid })
        } catch {
          // Record may not exist yet, ignore
        }
      }
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
    const gids = [...selectedGids.value]
    await Promise.all(gids.map(gid => {
      const task = tasks.value.find(t => t.gid === gid)
      const isBT = task?.bittorrent !== undefined
      return invoke(isBT ? 'force_pause_task' : 'pause_task', { gid }).catch(e =>
        console.error(`Failed to pause ${gid}:`, e)
      )
    }))
    await fetchTasks()
  }

  async function resumeSelectedTasks() {
    const gids = [...selectedGids.value]
    await Promise.all(gids.map(gid =>
      invoke('resume_task', { gid }).catch(e =>
        console.error(`Failed to resume ${gid}:`, e)
      )
    ))
    await fetchTasks()
  }

  async function removeSelectedTasks(deleteFiles = false) {
    const gids = [...selectedGids.value]
    for (const gid of gids) {
      await removeTask(gid, deleteFiles)
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

  async function retryTask(task: Task) {
    // Get the download URL from the task
    const uri = task.files?.[0]?.uris?.[0]?.uri
    if (!uri) throw new Error('No URI to retry')

    const options: Record<string, string> = {}
    if (task.dir) options.dir = task.dir

    // Remove the failed task record first
    try {
      await invoke('remove_task_record', { gid: task.gid })
    } catch {
      // Record may not exist, ignore
    }

    // Re-add the download
    await invoke('add_uri', { uris: [uri], options })
    await fetchTasks()
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
    searchQuery,
    sortField,
    sortOrder,
    // Getters
    filteredTasks,
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
    retryTask,
  }
})
