import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'

const sidecarModule = await import('../scripts/check-sidecar.mjs')

test('expectedSidecarPath uses app-private binary name', () => {
  const actual = sidecarModule.expectedSidecarPath('/repo', 'x86_64-unknown-linux-gnu')
  assert.equal(
    actual,
    path.join('/repo', 'src-tauri', 'binaries', 'motrix-aria2c-x86_64-unknown-linux-gnu'),
  )
})

test('validateSidecar rejects missing and empty files', async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'motrix-sidecar-test-'))
  const missingPath = path.join(tempDir, 'missing')
  await assert.rejects(
    sidecarModule.validateSidecar(missingPath),
    /not found/,
  )

  const emptyPath = path.join(tempDir, 'empty')
  await fs.writeFile(emptyPath, '')
  await assert.rejects(
    sidecarModule.validateSidecar(emptyPath),
    /empty/,
  )

  const placeholderPath = path.join(tempDir, 'placeholder')
  await fs.writeFile(placeholderPath, '#!/bin/sh\n# MOTRIX_SIDECAR_PLACEHOLDER\n')
  await assert.rejects(
    sidecarModule.validateSidecar(placeholderPath),
    /placeholder/,
  )
})

test('validateSidecar accepts a non-empty executable', async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'motrix-sidecar-test-'))
  const sidecarPath = path.join(tempDir, 'motrix-aria2c')
  await fs.writeFile(sidecarPath, '#!/bin/sh\nexit 0\n')

  await assert.doesNotReject(sidecarModule.validateSidecar(sidecarPath))
})
