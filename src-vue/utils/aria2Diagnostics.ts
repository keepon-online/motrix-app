export type Aria2ConnectionState = 'starting' | 'connected' | 'disconnected' | 'reconnecting' | 'terminated'

export type Aria2PanelStatus =
  | 'connected'
  | 'starting'
  | 'restarting'
  | 'reconnecting'
  | 'disconnected'
  | 'terminated'

export interface Aria2DiagnosticsState {
  connectionState: Aria2ConnectionState
  engineReady: boolean
  isRestarting: boolean
  lastError: string | null
  lastErrorAt: number | null
  lastSuccessAt: number | null
}

export function createAria2DiagnosticsState(): Aria2DiagnosticsState {
  return {
    connectionState: 'starting',
    engineReady: false,
    isRestarting: false,
    lastError: null,
    lastErrorAt: null,
    lastSuccessAt: null,
  }
}

export function normalizeAria2Error(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message || error.toString()
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message.trim()) return message
  }
  return String(error)
}

export function markRestartStarted(state: Aria2DiagnosticsState): Aria2DiagnosticsState {
  return {
    ...state,
    connectionState: 'reconnecting',
    engineReady: false,
    isRestarting: true,
  }
}

export function markRestartFailed(
  state: Aria2DiagnosticsState,
  error: unknown,
  at = Date.now(),
): Aria2DiagnosticsState {
  return recordAria2Failure(state, error, at)
}

export function recordAria2Failure(
  state: Aria2DiagnosticsState,
  error: unknown,
  at = Date.now(),
): Aria2DiagnosticsState {
  return {
    ...state,
    connectionState: 'terminated',
    engineReady: false,
    isRestarting: false,
    lastError: normalizeAria2Error(error),
    lastErrorAt: at,
  }
}

export function markRestartSucceeded(
  state: Aria2DiagnosticsState,
  at = Date.now(),
): Aria2DiagnosticsState {
  return {
    ...state,
    connectionState: 'connected',
    engineReady: true,
    isRestarting: false,
    lastError: null,
    lastErrorAt: null,
    lastSuccessAt: at,
  }
}

export function applyEngineReady(
  state: Aria2DiagnosticsState,
  at = Date.now(),
): Aria2DiagnosticsState {
  return {
    ...state,
    connectionState: 'connected',
    engineReady: true,
    isRestarting: false,
    lastError: null,
    lastErrorAt: null,
    lastSuccessAt: at,
  }
}

export function applyConnectionState(
  state: Aria2DiagnosticsState,
  connectionState: Aria2ConnectionState,
  at = Date.now(),
): Aria2DiagnosticsState {
  switch (connectionState) {
    case 'connected':
      return {
        ...state,
        connectionState,
        engineReady: true,
        isRestarting: false,
        lastError: null,
        lastErrorAt: null,
        lastSuccessAt: at,
      }
    case 'starting':
      return {
        ...state,
        connectionState,
        engineReady: false,
        isRestarting: false,
      }
    case 'disconnected':
      return {
        ...state,
        connectionState,
        engineReady: false,
        isRestarting: false,
      }
    case 'reconnecting':
      return {
        ...state,
        connectionState,
        engineReady: false,
        isRestarting: false,
      }
    case 'terminated':
      return {
        ...state,
        connectionState,
        engineReady: false,
        isRestarting: false,
      }
  }
}

export function getAria2PanelStatus(state: Aria2DiagnosticsState): Aria2PanelStatus {
  if (state.connectionState === 'starting') return 'starting'
  if (state.isRestarting) return 'restarting'
  if (state.connectionState === 'reconnecting') return 'reconnecting'
  if (state.connectionState === 'disconnected') return 'disconnected'
  if (state.connectionState === 'terminated') return 'terminated'
  if (!state.engineReady) return 'starting'
  return 'connected'
}
