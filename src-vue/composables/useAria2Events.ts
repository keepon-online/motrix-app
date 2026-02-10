import { onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useTaskStore } from '@/stores/task'
import { ElNotification } from 'element-plus'

export interface Aria2Event {
  eventType: 'download_start' | 'download_pause' | 'download_stop' | 'download_complete' | 'download_error' | 'bt_download_complete'
  gid: string
}

export function useAria2Events() {
  const { t } = useI18n()
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
        taskStore.fetchTasks('active')
        break

      case 'download_pause':
        taskStore.fetchTasks('active')
        break

      case 'download_stop':
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        break

      case 'download_complete':
      case 'bt_download_complete':
        ElNotification({
          title: t('task.completed'),
          message: `${t('task.completed')} - ${event.gid}`,
          type: 'success',
          duration: 5000,
        })
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        taskStore.fetchGlobalStat()
        break

      case 'download_error':
        ElNotification({
          title: t('task.error'),
          message: `${t('task.error')} - ${event.gid}`,
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
