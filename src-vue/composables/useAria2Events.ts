import { onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useTaskStore } from '@/stores/task'
import { useAppStore } from '@/stores/app'
import { ElNotification } from 'element-plus'
import { getTaskName } from '@/utils'

export interface Aria2Event {
  eventType: 'download_start' | 'download_pause' | 'download_stop' | 'download_complete' | 'download_error' | 'bt_download_complete'
  gid: string
}

export function useAria2Events() {
  const { t } = useI18n()
  const taskStore = useTaskStore()
  const appStore = useAppStore()
  let unlisten: UnlistenFn | null = null

  async function sendSystemNotification(title: string, body: string) {
    try {
      const { sendNotification, isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification')
      let granted = await isPermissionGranted()
      if (!granted) {
        const permission = await requestPermission()
        granted = permission === 'granted'
      }
      if (granted) {
        sendNotification({ title, body })
      }
    } catch {
      // Notification plugin may not be available
    }
  }

  async function setupEventListener() {
    try {
      unlisten = await listen<Aria2Event>('aria2-event', (event) => {
        handleAria2Event(event.payload)
      })
    } catch (error) {
      console.error('Failed to setup aria2 event listener:', error)
    }
  }

  async function handleAria2Event(event: Aria2Event) {
    switch (event.eventType) {
      case 'download_start':
        taskStore.fetchTasks('active')
        invoke('prevent_sleep').catch(() => {})
        break

      case 'download_pause':
        taskStore.fetchTasks('active')
        break

      case 'download_stop':
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        checkAndAllowSleep()
        break

      case 'download_complete':
      case 'bt_download_complete':
        if (appStore.config?.notifyOnComplete) {
          try {
            const info = await invoke<Record<string, unknown>>('get_task_info', { gid: event.gid })
            const name = getTaskName(info as { files?: { path?: string }[]; bittorrent?: { info?: { name?: string } } })
            sendSystemNotification(t('task.completed'), name)
            ElNotification({ title: t('task.completed'), message: name, type: 'success', duration: 5000 })
          } catch {
            sendSystemNotification(t('task.completed'), event.gid)
            ElNotification({ title: t('task.completed'), message: event.gid, type: 'success', duration: 5000 })
          }
        }
        // Auto-pause seeding if keepSeeding is disabled (BT tasks only)
        if (event.eventType === 'bt_download_complete' && appStore.config?.keepSeeding === false) {
          invoke('force_pause_task', { gid: event.gid }).catch((e) => {
            console.warn('Failed to auto-pause seeding task:', e)
          })
        }
        taskStore.fetchTasks('active')
        await taskStore.fetchTasks('stopped')
        taskStore.fetchGlobalStat()
        // Auto clear completed task record if configured (delay to let UI show it)
        if (appStore.config?.autoClearCompleted) {
          setTimeout(() => {
            taskStore.removeTaskRecord(event.gid).catch(() => {})
          }, 3000)
        }
        checkAndAllowSleep()
        // Bounce dock on macOS
        invoke('bounce_dock').catch(() => {})
        break

      case 'download_error':
        try {
          const info = await invoke<Record<string, unknown>>('get_task_info', { gid: event.gid })
          const name = getTaskName(info as { files?: { path?: string }[]; bittorrent?: { info?: { name?: string } } })
          const errorMsg = info.errorMessage ? `: ${info.errorMessage}` : ''
          sendSystemNotification(t('task.error'), `${name}${errorMsg}`)
          ElNotification({ title: t('task.error'), message: `${name}${errorMsg}`, type: 'error', duration: 8000 })
        } catch {
          sendSystemNotification(t('task.error'), event.gid)
          ElNotification({ title: t('task.error'), message: event.gid, type: 'error', duration: 8000 })
        }
        taskStore.fetchTasks('active')
        taskStore.fetchTasks('stopped')
        checkAndAllowSleep()
        break
    }
  }

  async function checkAndAllowSleep() {
    try {
      const stat = await invoke<{ numActive: string }>('get_global_stat')
      if (parseInt(stat.numActive || '0') === 0) {
        invoke('allow_sleep').catch(() => {})
      }
    } catch {
      // ignore
    }
  }

  let unlistenConnection: UnlistenFn | null = null
  let unlistenEngineDead: UnlistenFn | null = null
  let autoRestartAttempts = 0
  const MAX_AUTO_RESTART = 3
  const RESTART_COOLDOWN = 10_000 // 10s between attempts
  let lastRestartTime = 0

  async function setupConnectionListener() {
    try {
      unlistenConnection = await listen<string>('aria2-connection', (event) => {
        if (event.payload === 'connected') {
          appStore.engineReady = true
          appStore.engineError = ''
          autoRestartAttempts = 0 // Reset on successful connection
          // Refresh all task lists after reconnection
          taskStore.fetchTasks()
          taskStore.fetchGlobalStat()
        } else if (event.payload === 'disconnected') {
          appStore.engineReady = false
        }
      })
    } catch {
      // ignore
    }
  }

  async function setupEngineDeadListener() {
    try {
      unlistenEngineDead = await listen<string>('aria2-engine-dead', async (event) => {
        appStore.engineReady = false
        appStore.engineError = event.payload || 'engine_dead'

        // Check cooldown and retry limit
        const now = Date.now()
        if (now - lastRestartTime < RESTART_COOLDOWN) {
          return // Too soon, skip
        }

        if (autoRestartAttempts >= MAX_AUTO_RESTART) {
          ElNotification({
            title: t('engine.disconnected'),
            message: t('engine.restartFailed'),
            type: 'error',
            duration: 0, // Persistent until dismissed
          })
          return
        }

        autoRestartAttempts++
        lastRestartTime = now

        ElNotification({
          title: t('engine.disconnected'),
          message: t('engine.reconnecting'),
          type: 'warning',
          duration: 5000,
        })

        try {
          await invoke('restart_engine')
        } catch (e) {
          console.error('Auto-restart engine failed:', e)
        }
      })
    } catch {
      // ignore
    }
  }

  onMounted(() => {
    setupEventListener()
    setupConnectionListener()
    setupEngineDeadListener()
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
    }
    if (unlistenConnection) {
      unlistenConnection()
    }
    if (unlistenEngineDead) {
      unlistenEngineDead()
    }
  })

  return {
    handleAria2Event,
  }
}
