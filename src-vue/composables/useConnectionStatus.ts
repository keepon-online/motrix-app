import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type ConnectionState = 'connected' | 'disconnected' | 'reconnecting' | 'terminated' | 'failed'

// Module-level shared state — all consumers see the same values
const status = ref<ConnectionState>('connected')
const lastDisconnected = ref<Date | null>(null)
const engineReady = ref(false)
let listenersSetup = false
let unlisten: UnlistenFn | null = null
let unlistenReady: UnlistenFn | null = null

async function setupListeners() {
  if (listenersSetup) return
  listenersSetup = true

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

  try {
    unlistenReady = await listen('aria2-ready', () => {
      engineReady.value = true
    })
  } catch (error) {
    console.error('Failed to setup ready listener:', error)
  }
}

// Setup immediately on module import (before any component mounts)
setupListeners()

const isConnected = computed(() => status.value === 'connected')
const isDisconnected = computed(() => status.value === 'disconnected' || status.value === 'terminated')

export function useConnectionStatus() {
  return {
    status,
    isConnected,
    isDisconnected,
    lastDisconnected,
    engineReady,
  }
}
