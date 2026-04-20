import { computed, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import {
  applyConnectionState,
  applyEngineReady,
  createAria2DiagnosticsState,
  getAria2PanelStatus,
  markRestartFailed as reduceRestartFailed,
  markRestartStarted as reduceRestartStarted,
  markRestartSucceeded as reduceRestartSucceeded,
  recordAria2Failure,
} from '@/utils/aria2Diagnostics'

const diagnostics = ref(createAria2DiagnosticsState())
let listenersSetup = false

async function setupListeners() {
  if (listenersSetup) return
  listenersSetup = true

  try {
    await listen<string>('aria2-connection', (event) => {
      const state = event.payload
      if (
        state === 'connected'
        || state === 'disconnected'
        || state === 'reconnecting'
        || state === 'terminated'
      ) {
        diagnostics.value = applyConnectionState(diagnostics.value, state)
      }
    })
  } catch (error) {
    console.error('Failed to setup aria2 diagnostics connection listener:', error)
  }

  try {
    await listen('aria2-ready', () => {
      diagnostics.value = applyEngineReady(diagnostics.value)
    })
  } catch (error) {
    console.error('Failed to setup aria2 diagnostics ready listener:', error)
  }

  try {
    await listen<string>('aria2-error', (event) => {
      diagnostics.value = recordAria2Failure(diagnostics.value, event.payload)
    })
  } catch (error) {
    console.error('Failed to setup aria2 diagnostics error listener:', error)
  }
}

setupListeners()

export function useAria2Diagnostics() {
  function markRestartStarted() {
    diagnostics.value = reduceRestartStarted(diagnostics.value)
  }

  function markRestartFailed(error: unknown): string {
    diagnostics.value = reduceRestartFailed(diagnostics.value, error)
    return diagnostics.value.lastError ?? String(error)
  }

  function markRestartSucceeded() {
    diagnostics.value = reduceRestartSucceeded(diagnostics.value)
  }

  return {
    diagnostics,
    status: computed(() => diagnostics.value.connectionState),
    engineReady: computed(() => diagnostics.value.engineReady),
    panelStatus: computed(() => getAria2PanelStatus(diagnostics.value)),
    markRestartStarted,
    markRestartFailed,
    markRestartSucceeded,
  }
}
