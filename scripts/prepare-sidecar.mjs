import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import os from 'node:os'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { execFileSync } from 'node:child_process'

import {
  expectedSidecarPath,
  runSidecarCheck,
  validateSidecar,
} from './check-sidecar.mjs'

export const DEFAULT_ARIA2_VERSION = process.env.MOTRIX_ARIA2_VERSION ?? '1.37.0'

export function getPrepareStrategy(targetTriple) {
  return targetTriple.includes('windows') ? 'windows-archive' : 'source-build'
}

export function getSourceArchiveUrl(version) {
  return `https://github.com/aria2/aria2/releases/download/release-${version}/aria2-${version}.tar.xz`
}

export function getWindowsArchiveUrl(version) {
  return `https://github.com/aria2/aria2/releases/download/release-${version}/aria2-${version}-win-64bit-build1.zip`
}

export function getConfigureArgs(targetTriple) {
  if (targetTriple.includes('apple-darwin')) {
    return ['--without-gnutls', '--with-appletls', '--without-libgcrypt']
  }

  return ['--without-gnutls', '--with-openssl', '--without-libgcrypt']
}

export function getRequiredBuildTools(targetTriple) {
  if (targetTriple.includes('apple-darwin')) {
    // macOS: Homebrew's libtool provides glibtool/glibtoolize; system has Apple libtool
    return ['autoreconf', 'make', 'pkg-config']
  }

  // Linux: the libtool package installs libtoolize (not libtool); autoreconf -i calls libtoolize
  return ['autoreconf', 'make', 'pkg-config', 'libtoolize']
}

export function formatMissingBuildToolsMessage(targetTriple, missingTools) {
  const tools = missingTools.join(', ')
  if (targetTriple.includes('apple-darwin')) {
    return `Missing build tools for aria2 source build: ${tools}. Install them first, for example: brew install autoconf automake libtool pkg-config cppunit`
  }

  return `Missing build tools for aria2 source build: ${tools}. Install them first, for example: sudo apt install autoconf automake autopoint libtool pkg-config build-essential libssl-dev libxml2-dev libcppunit-dev zlib1g-dev libsqlite3-dev libssh2-1-dev libc-ares-dev`
}

async function exists(pathname) {
  try {
    await fs.access(pathname)
    return true
  } catch {
    return false
  }
}

async function ensureDir(pathname) {
  await fs.mkdir(pathname, { recursive: true })
}

async function commandExists(command) {
  try {
    await runCommand(process.platform === 'win32' ? 'where' : 'which', [command], { stdio: 'ignore' })
    return true
  } catch {
    return false
  }
}

function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: 'inherit',
      shell: false,
      ...options,
    })

    child.on('error', reject)
    child.on('exit', (code) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`${command} ${args.join(' ')} failed with exit code ${code}`))
      }
    })
  })
}

async function tryReuseSystemAria2c(sidecarPath) {
  const candidates = process.platform === 'win32'
    ? ['aria2c.exe', 'aria2c']
    : ['aria2c']

  for (const candidate of candidates) {
    try {
      const sourcePath = process.platform === 'win32'
        ? execFileSync('where', [candidate], { encoding: 'utf8' }).split(/\r?\n/).find(Boolean)
        : execFileSync('which', [candidate], { encoding: 'utf8' }).trim()
      if (!sourcePath || !(await exists(sourcePath))) continue

      await fs.copyFile(sourcePath, sidecarPath)
      await fs.chmod(sidecarPath, 0o755)
      await validateSidecar(sidecarPath)
      // Verify the copied binary works from its new location — some system
      // aria2c installations are launchers that only work in their original directory
      await runCommand(sidecarPath, ['--version'], { stdio: 'ignore' })
      return true
    } catch {
      // fall through to bundled build/download
    }
  }

  return false
}

async function prepareFromSource({ repoRoot, targetTriple, version, sidecarPath }) {
  const requiredTools = getRequiredBuildTools(targetTriple)
  const missingTools = []
  for (const tool of requiredTools) {
    if (!(await commandExists(tool))) {
      missingTools.push(tool)
    }
  }
  if (missingTools.length > 0) {
    throw new Error(formatMissingBuildToolsMessage(targetTriple, missingTools))
  }

  const downloadDir = path.join(repoRoot, '.sidecar-cache')
  const archiveName = `aria2-${version}.tar.xz`
  const sourceDir = path.join(downloadDir, `aria2-${version}`)
  const archivePath = path.join(downloadDir, archiveName)

  await ensureDir(downloadDir)

  if (!(await exists(archivePath))) {
    await runCommand('curl', ['-fSL', getSourceArchiveUrl(version), '-o', archivePath], { cwd: repoRoot })
  }

  if (!(await exists(sourceDir))) {
    await runCommand('tar', ['-xf', archivePath], { cwd: downloadDir })
  }

  await runCommand('autoreconf', ['-i'], { cwd: sourceDir })
  await runCommand('./configure', getConfigureArgs(targetTriple), { cwd: sourceDir })
  await runCommand('make', [`-j${os.cpus().length}`], { cwd: sourceDir })

  const builtBinary = path.join(sourceDir, 'src', 'aria2c')
  await validateSidecar(builtBinary)
  await fs.copyFile(builtBinary, sidecarPath)
  await fs.chmod(sidecarPath, 0o755)
}

async function findAria2cInDir(dirPath) {
  const entries = await fs.readdir(dirPath, { withFileTypes: true })
  for (const entry of entries) {
    const fullPath = path.join(dirPath, entry.name)
    if (entry.isFile() && entry.name === 'aria2c.exe') {
      return fullPath
    }
    if (entry.isDirectory()) {
      const found = await findAria2cInDir(fullPath)
      if (found) return found
    }
  }
  return null
}

async function prepareFromWindowsArchive({ repoRoot, version, sidecarPath }) {
  const downloadDir = path.join(repoRoot, '.sidecar-cache')
  const archiveName = `aria2-${version}-win-64bit-build1.zip`
  const archivePath = path.join(downloadDir, archiveName)
  const extractDir = path.join(downloadDir, `aria2-${version}-win-64bit-build1`)

  await ensureDir(downloadDir)

  if (!(await exists(archivePath))) {
    await runCommand('curl', ['-fSL', getWindowsArchiveUrl(version), '-o', archivePath], { cwd: repoRoot })
  }

  let extractedBinary = path.join(extractDir, `aria2-${version}-win-64bit-build1`, 'aria2c.exe')
  if (!(await exists(extractedBinary))) {
    if (process.platform === 'win32') {
      await runCommand('powershell', [
        '-NoProfile',
        '-Command',
        `Expand-Archive -Path "${archivePath}" -DestinationPath "${extractDir}" -Force`,
      ], { cwd: repoRoot })
    } else {
      await runCommand('tar', ['-xf', archivePath, '-C', extractDir], { cwd: repoRoot })
    }

    // Search for aria2c.exe in the extracted tree (zip structure varies per release)
    const found = await findAria2cInDir(extractDir)
    if (found) extractedBinary = found
  }

  await validateSidecar(extractedBinary)
  await fs.copyFile(extractedBinary, sidecarPath)
  await fs.chmod(sidecarPath, 0o755)
}

export async function prepareSidecar({
  repoRoot = process.cwd(),
  targetTriple = process.env.TARGET_TRIPLE ?? process.env.TARGET ?? process.env.TAURI_ENV_TARGET_TRIPLE,
  version = DEFAULT_ARIA2_VERSION,
  logger = console,
} = {}) {
  if (!targetTriple) {
    const rustc = process.env.RUSTC ?? 'rustc'
    const output = execFileSync(rustc, ['-vV'], { encoding: 'utf8' })
    const hostLine = output
      .split('\n')
      .map((line) => line.trim())
      .find((line) => line.startsWith('host: '))

    if (!hostLine) {
      throw new Error('prepare-sidecar requires TARGET_TRIPLE, TARGET, TAURI_ENV_TARGET_TRIPLE, or a working rustc')
    }

    targetTriple = hostLine.slice('host: '.length)
  }

  const sidecarPath = expectedSidecarPath(repoRoot, targetTriple)
  await ensureDir(path.dirname(sidecarPath))

  try {
    await validateSidecar(sidecarPath)
    logger.log(`Using existing aria2 sidecar: ${sidecarPath}`)
    return sidecarPath
  } catch {
    // prepare a real sidecar below
  }

  const reused = await tryReuseSystemAria2c(sidecarPath)
  if (!reused) {
    const strategy = getPrepareStrategy(targetTriple)
    if (strategy === 'windows-archive') {
      await prepareFromWindowsArchive({ repoRoot, version, sidecarPath })
    } else {
      await prepareFromSource({ repoRoot, targetTriple, version, sidecarPath })
    }
  }

  await validateSidecar(sidecarPath)
  logger.log(`Prepared aria2 sidecar: ${sidecarPath}`)
  return sidecarPath
}

const modulePath = fileURLToPath(import.meta.url)
const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : ''

if (modulePath === invokedPath) {
  const repoRoot = process.cwd()

  prepareSidecar({ repoRoot })
    .then(() => runSidecarCheck({ cwd: repoRoot }))
    .catch((error) => {
      console.error(error.message)
      process.exit(1)
    })
}
