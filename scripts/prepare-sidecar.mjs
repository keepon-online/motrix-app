import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import os from 'node:os'
import { createHash } from 'node:crypto'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { execFileSync } from 'node:child_process'

import {
  expectedSidecarPath,
  runSidecarCheck,
  validateSidecar,
} from './check-sidecar.mjs'
import {
  getSidecarArtifactByName,
  getSidecarTarget,
  loadSidecarManifest,
} from './sidecar-manifest.mjs'

const modulePath = fileURLToPath(import.meta.url)
const defaultRepoRoot = path.resolve(path.dirname(modulePath), '..')
const defaultManifest = loadSidecarManifest(defaultRepoRoot)

export const DEFAULT_ARIA2_VERSION = defaultManifest.aria2Version

function getTargetConfig(targetTriple, manifest = defaultManifest) {
  return getSidecarTarget(manifest, targetTriple)
}

export function getPrepareStrategy(targetTriple, manifest = defaultManifest) {
  return getTargetConfig(targetTriple, manifest).strategy
}

export function getSidecarReleaseTag(version) {
  return `aria2-sidecar-${version}`
}

export function getPrebuiltArtifactFileName(targetTriple) {
  const suffix = targetTriple.includes('windows') ? '.exe' : ''
  return `motrix-aria2c-${targetTriple}${suffix}`
}

export function getPrebuiltSidecarUrl(version, fileName) {
  return `https://github.com/keepon-online/motrix-app/releases/download/${getSidecarReleaseTag(version)}/${fileName}`
}

export function getSourceArchiveUrl(version) {
  return `https://github.com/aria2/aria2/releases/download/release-${version}/aria2-${version}.tar.xz`
}

export function getWindowsArchiveUrl(version) {
  return `https://github.com/aria2/aria2/releases/download/release-${version}/aria2-${version}-win-64bit-build1.zip`
}

export function getConfigureArgs(targetTriple, manifest = defaultManifest) {
  return getTargetConfig(targetTriple, manifest).configureArgs ?? []
}

export function getConfigureEnv(targetTriple, manifest = defaultManifest) {
  return getTargetConfig(targetTriple, manifest).configureEnv ?? {}
}

export function getRequiredBuildTools(targetTriple, manifest = defaultManifest) {
  return getTargetConfig(targetTriple, manifest).requiredBuildTools ?? []
}

export function formatMissingBuildToolsMessage(targetTriple, missingTools) {
  const tools = missingTools.join(', ')
  if (targetTriple.includes('apple-darwin')) {
    return `Missing build tools for aria2 source build: ${tools}. Install them first, for example: brew install autoconf automake libtool pkg-config cppunit`
  }

  return `Missing build tools for aria2 source build: ${tools}. Install them first, for example: sudo apt install autoconf automake autopoint libtool pkg-config build-essential binutils libssl-dev libexpat1-dev libcppunit-dev zlib1g-dev libsqlite3-dev libssh2-1-dev libgpg-error-dev`
}

export function formatMissingStaticLibrariesMessage(targetTriple, missingLibraries) {
  const libraries = missingLibraries.join(', ')
  if (targetTriple.includes('linux')) {
    return `Missing static libraries for portable Linux aria2 sidecar: ${libraries}. Install them first, for example: sudo apt install build-essential libssl-dev libexpat1-dev zlib1g-dev libsqlite3-dev libssh2-1-dev libgpg-error-dev`
  }

  return `Missing static libraries for aria2 sidecar: ${libraries}`
}

export function formatMissingArchiveToolMessage(command) {
  if (command === 'unzip') {
    return 'Missing archive tool for Windows sidecar extraction: unzip. Install it first, for example: sudo apt install unzip'
  }

  return `Missing archive extraction tool: ${command}`
}

export function formatMissingContainerToolMessage(command) {
  if (command === 'docker') {
    return 'Missing container runtime for portable Linux sidecar build: docker. Install Docker or disable MOTRIX_SIDECAR_USE_DOCKER.'
  }

  return `Missing container runtime for sidecar build: ${command}`
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

async function anyPathExists(paths, pathExists = exists) {
  for (const candidate of paths) {
    if (await pathExists(candidate)) {
      return true
    }
  }
  return false
}

export function getRequiredStaticLibraries(targetTriple, manifest = defaultManifest) {
  return getTargetConfig(targetTriple, manifest).requiredStaticLibraries ?? []
}

export function getSourceBuildDir(
  repoRoot,
  targetTriple,
  artifact,
  buildContext = envFlagEnabled(process.env.MOTRIX_SIDECAR_IN_DOCKER) ? 'docker' : 'native',
) {
  return path.join(
    repoRoot,
    '.sidecar-cache',
    'build',
    targetTriple,
    buildContext,
    artifact.extractDir,
  )
}

async function resolveCommandOutput(command, args) {
  return execFileSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'ignore'],
  })
}

async function resolveStaticLibraryPath(
  library,
  pathExists = exists,
  resolveCommand = resolveCommandOutput,
) {
  if (await anyPathExists(library.paths ?? [], pathExists)) {
    return true
  }

  for (const commandSpec of library.resolveCommands ?? []) {
    const [command, ...args] = commandSpec
    let output
    try {
      output = await resolveCommand(command, args)
    } catch {
      continue
    }

    const candidate = String(output ?? '').trim()
    if (!candidate) continue
    if (await pathExists(candidate)) {
      return true
    }
  }

  return false
}

export async function findMissingStaticLibraries(
  targetTriple,
  manifest = defaultManifest,
  {
    pathExists = exists,
    resolveCommand = resolveCommandOutput,
  } = {},
) {
  const missing = []
  for (const library of getRequiredStaticLibraries(targetTriple, manifest)) {
    if (!(await resolveStaticLibraryPath(library, pathExists, resolveCommand))) {
      missing.push(library.name)
    }
  }
  return missing
}

function hostPlatformForTarget(target) {
  if (target.platform === 'windows') return 'win32'
  if (target.platform === 'macos') return 'darwin'
  if (target.platform === 'linux') return 'linux'
  return target.platform
}

function envFlagEnabled(value) {
  return ['1', 'true', 'yes'].includes(String(value ?? '').toLowerCase())
}

function envFlagDisabled(value) {
  return ['0', 'false', 'no'].includes(String(value ?? '').toLowerCase())
}

function sourceBuildRequested(env = process.env) {
  return envFlagEnabled(env.MOTRIX_SIDECAR_BUILD_FROM_SOURCE)
}

function fallbackRequested(env = process.env) {
  return envFlagEnabled(env.MOTRIX_SIDECAR_REGENERATE) || sourceBuildRequested(env)
}

export function shouldReuseSystemAria2c(
  targetTriple,
  env = process.env,
  hostPlatform = process.platform,
  manifest = defaultManifest,
) {
  if (!envFlagEnabled(env.MOTRIX_REUSE_SYSTEM_ARIA2C)) {
    return false
  }

  const expectedPlatform = hostPlatformForTarget(getTargetConfig(targetTriple, manifest))
  return expectedPlatform !== null && expectedPlatform === hostPlatform
}

export function shouldUseDockerizedBuild(
  targetTriple,
  env = process.env,
  hostPlatform = process.platform,
  manifest = defaultManifest,
) {
  const target = getTargetConfig(targetTriple, manifest)
  if (!target.dockerBuild) return false
  if (envFlagDisabled(env.MOTRIX_SIDECAR_USE_DOCKER)) return false
  if (!envFlagEnabled(env.MOTRIX_SIDECAR_USE_DOCKER)) return false
  if (envFlagEnabled(env.MOTRIX_SIDECAR_IN_DOCKER)) return false
  return target.platform === 'linux' && hostPlatform === 'linux'
}

function getFallbackStrategy(targetTriple, env = process.env, manifest = defaultManifest) {
  const target = getTargetConfig(targetTriple, manifest)
  if (!fallbackRequested(env)) {
    return null
  }

  if (sourceBuildRequested(env) && target.fallbackStrategy !== 'source-build') {
    throw new Error(
      `MOTRIX_SIDECAR_BUILD_FROM_SOURCE=1 is not supported for target: ${targetTriple}`
    )
  }

  if (!target.fallbackStrategy) {
    throw new Error(
      `No fallback sidecar preparation strategy is configured for target: ${targetTriple}`
    )
  }

  return target.fallbackStrategy
}

export function resolvePrepareStrategy(
  targetTriple,
  env = process.env,
  manifest = defaultManifest,
) {
  return getFallbackStrategy(targetTriple, env, manifest) ?? getPrepareStrategy(targetTriple, manifest)
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

async function fileSha256(filePath) {
  const buffer = await fs.readFile(filePath)
  return createHash('sha256').update(buffer).digest('hex')
}

async function validateArtifactChecksum(archivePath, artifact) {
  if (!artifact.sha256) {
    return
  }

  const actual = await fileSha256(archivePath)
  if (actual !== artifact.sha256) {
    throw new Error(
      `Checksum mismatch for ${archivePath}: expected ${artifact.sha256}, got ${actual}`
    )
  }
}

async function ensureArtifactArchive({ repoRoot, downloadDir, artifact }) {
  const fileName = artifact.archiveName
    ?? artifact.fileName
    ?? artifact.binaryName
    ?? path.basename(new URL(artifact.url).pathname)
  const archivePath = path.join(downloadDir, fileName)
  if (!(await exists(archivePath))) {
    await runCommand('curl', ['-fSL', artifact.url, '-o', archivePath], { cwd: repoRoot })
  }

  await validateArtifactChecksum(archivePath, artifact)
  return archivePath
}

async function prepareFromReleaseDownload({ repoRoot, targetTriple, target, artifact, sidecarPath }) {
  const downloadDir = path.join(repoRoot, '.sidecar-cache')
  await ensureDir(downloadDir)

  const downloadedPath = await ensureArtifactArchive({ repoRoot, downloadDir, artifact })
  await validateSidecar(downloadedPath, {
    targetTriple,
    validation: target.validation,
  })
  await fs.copyFile(downloadedPath, sidecarPath)
  await fs.chmod(sidecarPath, 0o755)
}

async function tryReuseSystemAria2c(sidecarPath, targetTriple, target) {
  const candidates = targetTriple.includes('windows')
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
      await validateSidecar(sidecarPath, {
        targetTriple,
        validation: target.validation,
      })
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

async function prepareFromSource({ repoRoot, manifest, targetTriple, target, artifact, sidecarPath }) {
  if (shouldUseDockerizedBuild(targetTriple, process.env, process.platform, manifest)) {
    if (!(await commandExists('docker'))) {
      throw new Error(formatMissingContainerToolMessage('docker'))
    }

    const dockerCommand = getLinuxDockerBuildCommand(repoRoot, targetTriple, manifest)
    await runCommand(dockerCommand.command, dockerCommand.args, { cwd: repoRoot })
    await validateSidecar(sidecarPath, {
      targetTriple,
      validation: target.validation,
    })
    return
  }

  const requiredTools = getRequiredBuildTools(targetTriple, manifest)
  const missingTools = []
  for (const tool of requiredTools) {
    if (!(await commandExists(tool))) {
      missingTools.push(tool)
    }
  }
  if (missingTools.length > 0) {
    throw new Error(formatMissingBuildToolsMessage(targetTriple, missingTools))
  }

  const missingStaticLibraries = await findMissingStaticLibraries(targetTriple, manifest)
  if (missingStaticLibraries.length > 0) {
    throw new Error(formatMissingStaticLibrariesMessage(targetTriple, missingStaticLibraries))
  }

  const downloadDir = path.join(repoRoot, '.sidecar-cache')
  const sourceDir = getSourceBuildDir(repoRoot, targetTriple, artifact)
  const sourceParentDir = path.dirname(sourceDir)

  await ensureDir(downloadDir)
  await ensureDir(sourceParentDir)

  const archivePath = await ensureArtifactArchive({ repoRoot, downloadDir, artifact })
  await fs.rm(sourceDir, { recursive: true, force: true })
  await runCommand('tar', ['-xf', archivePath, '-C', sourceParentDir], { cwd: repoRoot })

  await runCommand('autoreconf', ['-fi'], { cwd: sourceDir })
  await runCommand('./configure', getConfigureArgs(targetTriple), {
    cwd: sourceDir,
    env: {
      ...process.env,
      ...getConfigureEnv(targetTriple, manifest),
    },
  })
  await runCommand('make', [`-j${os.cpus().length}`], { cwd: sourceDir })

  const builtBinary = path.join(sourceDir, 'src', 'aria2c')
  await validateSidecar(builtBinary, {
    targetTriple,
    validation: target.validation,
  })
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

export function getWindowsArchiveExtractCommand(archivePath, extractDir, hostPlatform = process.platform) {
  if (hostPlatform === 'win32') {
    return {
      command: 'powershell',
      args: [
        '-NoProfile',
        '-Command',
        `Expand-Archive -Path "${archivePath}" -DestinationPath "${extractDir}" -Force`,
      ],
    }
  }

  return {
    command: 'unzip',
    args: ['-q', '-o', archivePath, '-d', extractDir],
  }
}

export function getLinuxDockerBuildCommand(
  repoRoot,
  targetTriple,
  manifest = defaultManifest,
) {
  const target = getTargetConfig(targetTriple, manifest)
  const dockerBuild = target.dockerBuild
  if (!dockerBuild) {
    throw new Error(`Docker build is not configured for target: ${targetTriple}`)
  }

  const installPackages = dockerBuild.packages.join(' ')
  const dockerEnvArgs = Object.entries(dockerBuild.env ?? {}).flatMap(([key, value]) => [
    '-e',
    `${key}=${value}`,
  ])
  const innerCommand = [
    'apt-get update',
    `apt-get install -y ${installPackages}`,
    'node scripts/prepare-sidecar.mjs',
  ].join(' && ')

  return {
    command: 'docker',
    args: [
      'run',
      '--rm',
      '-v',
      `${repoRoot}:/workspace`,
      '-w',
      '/workspace',
      '-e',
      `TARGET_TRIPLE=${targetTriple}`,
      '-e',
      'MOTRIX_SIDECAR_USE_DOCKER=0',
      '-e',
      'MOTRIX_SIDECAR_IN_DOCKER=1',
      ...dockerEnvArgs,
      dockerBuild.image,
      'bash',
      '-lc',
      innerCommand,
    ],
  }
}

async function prepareFromWindowsArchive({ repoRoot, targetTriple, target, artifact, sidecarPath }) {
  const downloadDir = path.join(repoRoot, '.sidecar-cache')
  const extractDir = path.join(downloadDir, artifact.extractDir)

  await ensureDir(downloadDir)

  const archivePath = await ensureArtifactArchive({ repoRoot, downloadDir, artifact })

  let extractedBinary = path.join(extractDir, artifact.extractDir, artifact.binaryName)
  if (!(await exists(extractedBinary))) {
    await ensureDir(extractDir)
    const extractCommand = getWindowsArchiveExtractCommand(archivePath, extractDir)
    if (!(await commandExists(extractCommand.command))) {
      throw new Error(formatMissingArchiveToolMessage(extractCommand.command))
    }
    await runCommand(extractCommand.command, extractCommand.args, { cwd: repoRoot })

    // Search for aria2c.exe in the extracted tree (zip structure varies per release)
    const found = await findAria2cInDir(extractDir)
    if (found) extractedBinary = found
  }

  await validateSidecar(extractedBinary, {
    targetTriple,
    validation: target.validation,
  })
  await fs.copyFile(extractedBinary, sidecarPath)
  await fs.chmod(sidecarPath, 0o755)
}

export async function prepareSidecar({
  repoRoot = process.cwd(),
  targetTriple = process.env.TARGET_TRIPLE ?? process.env.TARGET ?? process.env.TAURI_ENV_TARGET_TRIPLE,
  version = process.env.MOTRIX_ARIA2_VERSION ?? DEFAULT_ARIA2_VERSION,
  logger = console,
} = {}) {
  const manifest = loadSidecarManifest(repoRoot)
  if (version !== manifest.aria2Version) {
    throw new Error(
      `MOTRIX_ARIA2_VERSION (${version}) must match sidecar-manifest.json (${manifest.aria2Version}). Update the manifest to change aria2 versions.`
    )
  }

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
  const target = getSidecarTarget(manifest, targetTriple)
  const strategy = resolvePrepareStrategy(targetTriple, process.env, manifest)
  const artifactName = strategy === target.strategy
    ? target.artifact
    : (target.fallbackArtifact ?? target.artifact)
  const artifact = getSidecarArtifactByName(manifest, artifactName)
  await ensureDir(path.dirname(sidecarPath))

  try {
    await validateSidecar(sidecarPath, {
      targetTriple,
      validation: target.validation,
    })
    logger.log(`Using existing aria2 sidecar: ${sidecarPath}`)
    return sidecarPath
  } catch {
    // prepare a real sidecar below
  }

  const reused = shouldReuseSystemAria2c(targetTriple, process.env, process.platform, manifest)
    ? await tryReuseSystemAria2c(sidecarPath, targetTriple, target)
    : false
  if (!reused) {
    if (strategy === 'release-download') {
      await prepareFromReleaseDownload({ repoRoot, targetTriple, target, artifact, sidecarPath })
    } else if (strategy === 'windows-archive') {
      await prepareFromWindowsArchive({ repoRoot, targetTriple, target, artifact, sidecarPath })
    } else if (strategy === 'source-build') {
      await prepareFromSource({ repoRoot, manifest, targetTriple, target, artifact, sidecarPath })
    } else {
      throw new Error(`Unsupported sidecar preparation strategy: ${strategy}`)
    }
  }

  await validateSidecar(sidecarPath, {
    targetTriple,
    validation: target.validation,
  })
  logger.log(`Prepared aria2 sidecar: ${sidecarPath}`)
  return sidecarPath
}

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
