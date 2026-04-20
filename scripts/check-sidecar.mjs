import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

export function expectedSidecarFileName(targetTriple) {
  const suffix = targetTriple.includes('windows') ? '.exe' : ''
  return `motrix-aria2c-${targetTriple}${suffix}`
}

export function expectedSidecarPath(repoRoot, targetTriple) {
  return path.join(repoRoot, 'src-tauri', 'binaries', expectedSidecarFileName(targetTriple))
}

export async function validateSidecar(sidecarPath) {
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

  const header = await fs.readFile(sidecarPath, { encoding: 'utf8' })
  if (header.includes('MOTRIX_SIDECAR_PLACEHOLDER')) {
    throw new Error(`Aria2 sidecar is still the repository placeholder: ${sidecarPath}`)
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
  const sidecarPath = expectedSidecarPath(cwd, targetTriple)

  await validateSidecar(sidecarPath)
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
