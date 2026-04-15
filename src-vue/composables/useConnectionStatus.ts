import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type ConnectionState = 'connected' | 'disconnected' | 'reconnecting' | 'terminated' | 'failed'

export function useConnectionStatus() {
  const status = ref<ConnectionState>('connected')
  const lastDisconnected = ref<Date | null>(null)
  let unlisten: UnlistenFn | null = null

  async function setupListener() {
    try {
      unlisten = await listen<string>('aria2-connection', (event) => {
        const state = event.payload
        switch (state) {
          case 'connected':
            status.value = 'connected'
            lastDisconnected.value = null
            break
          case 'disconnected':
            status.value = 'disconnected'
            lastDisconnected.value = new Date()
            break
          case 'terminated':
            status.value = 'terminated'
            lastDisconnected.value = new Date()
            break
          default:
            break
        }
      })
    } catch (error) {
      console.error('Failed to setup connection listener:', error)
    }
  }

  onMounted(() => {
    setupListener()
  })

  onUnmounted(() => {
    if (unlisten) {
      unlisten()
    }
  })

  const isConnected = computed(() => status.value === 'connected')
  const isDisconnected = computed(() => status.value === 'disconnected' || status.value === 'terminated')

  return {
    status,
    isConnected,
    isDisconnected,
    lastDisconnected,
  }
}
