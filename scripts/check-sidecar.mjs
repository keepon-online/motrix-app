import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

import {
  getSidecarValidation,
  loadSidecarManifest,
} from './sidecar-manifest.mjs'

export function expectedSidecarFileName(targetTriple) {
  const suffix = targetTriple.includes('windows') ? '.exe' : ''
  return `motrix-aria2c-${targetTriple}${suffix}`
}

export function expectedSidecarPath(repoRoot, targetTriple) {
  return path.join(repoRoot, 'src-tauri', 'binaries', expectedSidecarFileName(targetTriple))
}

async function readPrefix(sidecarPath, length) {
  const file = await fs.open(sidecarPath, 'r')
  try {
    const buffer = Buffer.alloc(length)
    const { bytesRead } = await file.read(buffer, 0, length, 0)
    return buffer.subarray(0, bytesRead)
  } finally {
    await file.close()
  }
}

function isWindowsTarget(targetTriple) {
  return targetTriple?.includes('windows') ?? false
}

function isLinuxTarget(targetTriple) {
  return targetTriple?.includes('linux') ?? false
}

export function parseNeededLibraries(output) {
  return output
    .split(/\r?\n/)
    .map((line) => line.match(/^\s*NEEDED\s+(\S+)/)?.[1])
    .filter(Boolean)
}

function validateLinuxPortability(sidecarPath) {
  let output
  try {
    output = execFileSync('objdump', ['-p', sidecarPath], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    })
  } catch (error) {
    throw new Error(
      `Unable to inspect Linux aria2 sidecar dependencies with objdump: ${error.message}`
    )
  }

  const neededLibraries = parseNeededLibraries(output)
  if (neededLibraries.length > 0) {
    throw new Error(
      `Linux aria2 sidecar is dynamically linked (${neededLibraries.join(', ')}): ${sidecarPath}. Build a static sidecar for release portability.`
    )
  }
}

export async function validateSidecar(sidecarPath, {
  targetTriple,
  requirePortableLinux = false,
  validation = {},
} = {}) {
  let stat
  try {
    stat = await fs.stat(sidecarPath)
  } catch {
    throw new Error(`Aria2 sidecar not found: ${sidecarPath}`)
  }

  if (!stat.isFile()) {
    throw new Error(`Aria2 sidecar is not a file: ${sidecarPath}`)
  }

  if (stat.size === 0) {
    throw new Error(`Aria2 sidecar is empty: ${sidecarPath}`)
  }

  const prefix = await readPrefix(sidecarPath, 4096)
  const prefixText = prefix.toString('utf8')
  if (prefixText.includes('MOTRIX_SIDECAR_PLACEHOLDER')) {
    throw new Error(`Aria2 sidecar is still the repository placeholder: ${sidecarPath}`)
  }

  // Validate by target triple, not the host OS. CI may prepare a Windows
  // sidecar on Linux, and host-based checks let ELF binaries pass as .exe.
  if (validation.format === 'pe' || isWindowsTarget(targetTriple)) {
    if (prefix[0] !== 0x4d || prefix[1] !== 0x5a) {
      throw new Error(
        `Aria2 sidecar is not a valid Windows executable (expected MZ header): ${sidecarPath}`
      )
    }
  }

  if (validation.format === 'elf' || isLinuxTarget(targetTriple)) {
    if (
      prefix[0] !== 0x7f ||
      prefix[1] !== 0x45 ||
      prefix[2] !== 0x4c ||
      prefix[3] !== 0x46
    ) {
      throw new Error(
        `Aria2 sidecar is not a valid Linux executable (expected ELF header): ${sidecarPath}`
      )
    }

    if (requirePortableLinux || validation.portableLinux) {
      validateLinuxPortability(sidecarPath)
    }
  }
}

function detectTargetTriple(env = process.env) {
  if (env.TARGET_TRIPLE) return env.TARGET_TRIPLE
  if (env.TARGET) return env.TARGET

  const rustc = env.RUSTC ?? 'rustc'
  const output = execFileSync(rustc, ['-vV'], { encoding: 'utf8' })
  const hostLine = output
    .split('\n')
    .map((line) => line.trim())
    .find((line) => line.startsWith('host: '))

  if (!hostLine) {
    throw new Error('Unable to detect Rust host target triple from rustc -vV')
  }

  return hostLine.slice('host: '.length)
}

export async function runSidecarCheck({
  env = process.env,
  cwd = process.cwd(),
  logger = console,
} = {}) {
  const targetTriple = detectTargetTriple(env)
  const manifest = loadSidecarManifest(cwd)
  const validation = getSidecarValidation(manifest, targetTriple)
  const sidecarPath = expectedSidecarPath(cwd, targetTriple)

  await validateSidecar(sidecarPath, {
    targetTriple,
    validation,
  })
  if (validation.runVersion) {
    execFileSync(sidecarPath, ['--version'], { stdio: 'ignore' })
  }
  logger.log(`Using aria2 sidecar: ${sidecarPath}`)
}

const modulePath = fileURLToPath(import.meta.url)
const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : ''

if (modulePath === invokedPath) {
  runSidecarCheck().catch((error) => {
    console.error(error.message)
    process.exit(1)
  })
}
