import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'

type ConnectionState = 'connected' | 'disconnected' | 'reconnecting' | 'terminated' | 'failed'

// Module-level shared state — all consumers see the same values
const status = ref<ConnectionState>('connected')
const engineReady = ref(false)
let listenersSetup = false

async function setupListeners() {
  if (listenersSetup) return
  listenersSetup = true

  try {
    await listen<string>('aria2-connection', (event) => {
      const state = event.payload
      switch (state) {
        case 'connected':
          status.value = 'connected'
          break
        case 'disconnected':
          status.value = 'disconnected'
          engineReady.value = false
          break
        case 'reconnecting':
          status.value = 'reconnecting'
          break
        case 'terminated':
          status.value = 'terminated'
          engineReady.value = false
          break
        default:
          break
      }
    })
  } catch (error) {
    console.error('Failed to setup connection listener:', error)
  }

  try {
    await listen('aria2-ready', () => {
      engineReady.value = true
    })
  } catch (error) {
    console.error('Failed to setup ready listener:', error)
  }
}

// Setup immediately on module import (before any component mounts)
setupListeners()

const isDisconnected = computed(() => status.value === 'disconnected' || status.value === 'terminated')

export function useConnectionStatus() {
  return {
    status,
    isDisconnected,
    engineReady,
  }
}
