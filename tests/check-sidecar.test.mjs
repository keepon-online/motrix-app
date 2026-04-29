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

test('validateSidecar validates Windows targets even when host is not Windows', async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'motrix-sidecar-test-'))
  const sidecarPath = path.join(tempDir, 'motrix-aria2c-x86_64-pc-windows-gnu.exe')
  await fs.writeFile(sidecarPath, Buffer.from([0x7f, 0x45, 0x4c, 0x46]))

  await assert.rejects(
    sidecarModule.validateSidecar(sidecarPath, {
      targetTriple: 'x86_64-pc-windows-gnu',
    }),
    /valid Windows executable/,
  )
})

test('validateSidecar rejects non-Mach-O binaries for macOS targets', async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'motrix-sidecar-test-'))
  const sidecarPath = path.join(tempDir, 'motrix-aria2c-x86_64-apple-darwin')
  await fs.writeFile(sidecarPath, Buffer.from([0x7f, 0x45, 0x4c, 0x46]))

  await assert.rejects(
    sidecarModule.validateSidecar(sidecarPath, {
      targetTriple: 'x86_64-apple-darwin',
    }),
    /valid macOS executable/,
  )
})

test('validateSidecar rejects Mach-O binaries with the wrong macOS architecture', async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'motrix-sidecar-test-'))
  const sidecarPath = path.join(tempDir, 'motrix-aria2c-x86_64-apple-darwin')
  const machO = Buffer.alloc(32)
  machO.writeUInt32LE(0xfeedfacf, 0)
  machO.writeUInt32LE(0x0100000c, 4)
  await fs.writeFile(sidecarPath, machO)

  await assert.rejects(
    sidecarModule.validateSidecar(sidecarPath, {
      targetTriple: 'x86_64-apple-darwin',
    }),
    /expected x86_64/,
  )
})

test('validateSidecar accepts Mach-O binaries matching the macOS target architecture', async () => {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'motrix-sidecar-test-'))
  const sidecarPath = path.join(tempDir, 'motrix-aria2c-x86_64-apple-darwin')
  const machO = Buffer.alloc(32)
  machO.writeUInt32LE(0xfeedfacf, 0)
  machO.writeUInt32LE(0x01000007, 4)
  await fs.writeFile(sidecarPath, machO)

  await assert.doesNotReject(
    sidecarModule.validateSidecar(sidecarPath, {
      targetTriple: 'x86_64-apple-darwin',
    }),
  )
})

test('parseNeededLibraries extracts ELF NEEDED entries', () => {
  const output = `
Dynamic Section:
  NEEDED               libssl.so.3
  NEEDED               libcrypto.so.3
`

  assert.deepEqual(
    sidecarModule.parseNeededLibraries(output),
    ['libssl.so.3', 'libcrypto.so.3'],
  )
})

test('validateSidecar rejects dynamically linked Linux sidecars when portability is required', async (t) => {
  if (process.platform !== 'linux') {
    t.skip('Linux dynamic-link validation is only meaningful on Linux hosts')
    return
  }

  await assert.rejects(
    sidecarModule.validateSidecar('/bin/ls', {
      targetTriple: 'x86_64-unknown-linux-gnu',
      requirePortableLinux: true,
    }),
    /dynamically linked/,
  )
})
