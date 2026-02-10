import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from '@/stores/task'
import { ElNotification } from 'element-plus'

export interface Aria2Event {
  eventType: 'download_start' | 'download_pause' | 'download_stop' | 'download_complete' | 'download_error' | 'bt_download_complete'
  gid: string
}

export function useAria2Events() {
  const taskStore = useTaskStore()
  let unlisten: UnlistenFn | null = null

  async function setupEventListener() {
    try {
      unlisten = await listen<Aria2Event>('aria2-event', (event) => {
        handleAria2Event(event.payload)
      })
    } catch (error) {
      console.error('Failed to setup aria2 event listener:', error)
    }
  }

  function handleAria2Event(event: Aria2Event) {
    console.log('Aria2 event received:', event)

    switch (event.eventType) {
      case 'download_start':
        // Refresh task list to show new task
        taskStore.fetchTasks('active')
        break

      case 'download_pause':
        // Update task status
        taskStore.fetchTasks('active')
        break

      case 'download_stop':
        // Task was stopped/removed
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        break

      case 'download_complete':
      case 'bt_download_complete':
        // Show success notification
        ElNotification({
          title: 'Download Complete',
          message: `Task ${event.gid} has finished downloading`,
          type: 'success',
          duration: 5000,
        })
        // Refresh both lists
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        taskStore.fetchGlobalStat()
        break

      case 'download_error':
        // Show error notification
        ElNotification({
          title: 'Download Error',
          message: `Task ${event.gid} encountered an error`,
          type: 'error',
          duration: 8000,
        })
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        break
    }
  }

  onMounted(() => {
    setupEventListener()
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
    }
  })

  return {
    handleAria2Event,
  }
}
