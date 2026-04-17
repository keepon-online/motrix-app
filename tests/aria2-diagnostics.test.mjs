import test from 'node:test'
import assert from 'node:assert/strict'
import { build } from 'esbuild'
import path from 'node:path'

async function importTypeScriptModule(relativePath) {
  const entryPoint = path.resolve(process.cwd(), relativePath)
  const result = await build({
    entryPoints: [entryPoint],
    absWorkingDir: process.cwd(),
    bundle: true,
    format: 'esm',
    platform: 'node',
    write: false,
    sourcemap: 'inline',
  })

  const output = result.outputFiles[0].text
  const moduleUrl = `data:text/javascript;base64,${Buffer.from(output).toString('base64')}`
  return import(moduleUrl)
}

test('aria2 diagnostics keeps the latest restart failure until recovery succeeds', async () => {
  const diagnosticsModule = await importTypeScriptModule('src-vue/utils/aria2Diagnostics.ts')
  const {
    createAria2DiagnosticsState,
    markRestartStarted,
    markRestartFailed,
    applyConnectionState,
  } = diagnosticsModule

  let state = createAria2DiagnosticsState()
  state = markRestartStarted(state)
  state = markRestartFailed(
    state,
    'WebSocket error: IO error: connection refused',
    1713340800000,
  )

  assert.equal(state.isRestarting, false)
  assert.equal(state.lastError, 'WebSocket error: IO error: connection refused')
  assert.equal(state.lastErrorAt, 1713340800000)

  state = applyConnectionState(state, 'reconnecting', 1713340860000)
  assert.equal(state.connectionState, 'reconnecting')
  assert.equal(state.lastError, 'WebSocket error: IO error: connection refused')

  state = applyConnectionState(state, 'connected', 1713340920000)
  assert.equal(state.connectionState, 'connected')
  assert.equal(state.lastError, null)
  assert.equal(state.lastSuccessAt, 1713340920000)
})
