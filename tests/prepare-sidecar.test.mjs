import test from 'node:test'
import assert from 'node:assert/strict'

const prepareModule = await import('../scripts/prepare-sidecar.mjs')

test('getPrepareStrategy uses prebuilt release downloads for packaged targets by default', () => {
  assert.equal(
    prepareModule.getPrepareStrategy('x86_64-pc-windows-msvc'),
    'release-download',
  )
  assert.equal(
    prepareModule.getPrepareStrategy('x86_64-unknown-linux-gnu'),
    'release-download',
  )
  assert.equal(
    prepareModule.getPrepareStrategy('aarch64-apple-darwin'),
    'release-download',
  )
})

test('getPrepareStrategy rejects targets missing from the sidecar manifest', () => {
  assert.throws(
    () => prepareModule.getPrepareStrategy('armv7-unknown-linux-gnueabihf'),
    /Unsupported aria2 sidecar target/,
  )
})

test('download URLs are pinned to the configured aria2 version', () => {
  assert.equal(
    prepareModule.getSourceArchiveUrl('1.37.0'),
    'https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0.tar.xz',
  )
  assert.equal(
    prepareModule.getWindowsArchiveUrl('1.37.0'),
    'https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip',
  )
})

test('prebuilt sidecar asset URLs are pinned to the repository release tag', () => {
  assert.equal(
    prepareModule.getSidecarReleaseTag('1.37.0'),
    'aria2-sidecar-1.37.0',
  )
  assert.equal(
    prepareModule.getPrebuiltSidecarUrl('1.37.0', 'motrix-aria2c-x86_64-unknown-linux-gnu'),
    'https://github.com/keepon-online/motrix-app/releases/download/aria2-sidecar-1.37.0/motrix-aria2c-x86_64-unknown-linux-gnu',
  )
})

test('prebuilt sidecar artifact file names match target-specific binary names', () => {
  assert.equal(
    prepareModule.getPrebuiltArtifactFileName('x86_64-pc-windows-msvc'),
    'motrix-aria2c-x86_64-pc-windows-msvc.exe',
  )
  assert.equal(
    prepareModule.getPrebuiltArtifactFileName('x86_64-unknown-linux-gnu'),
    'motrix-aria2c-x86_64-unknown-linux-gnu',
  )
})

test('configure args use platform-appropriate TLS backend', () => {
  assert.deepEqual(
    prepareModule.getConfigureArgs('x86_64-unknown-linux-gnu'),
    [
      '--without-gnutls',
      '--with-openssl',
      '--without-libgcrypt',
      '--without-libxml2',
      '--with-libexpat',
      '--without-libcares',
      '--with-libssh2',
      '--with-sqlite3',
      '--with-ca-bundle=/etc/ssl/certs/ca-certificates.crt',
    ],
  )
  assert.deepEqual(
    prepareModule.getConfigureArgs('aarch64-apple-darwin'),
    ['--without-gnutls', '--with-appletls', '--without-libgcrypt'],
  )
})

test('linux source builds request a static sidecar', () => {
  assert.deepEqual(
    prepareModule.getConfigureEnv('x86_64-unknown-linux-gnu'),
    { ARIA2_STATIC: 'yes' },
  )
  assert.deepEqual(
    prepareModule.getConfigureEnv('aarch64-apple-darwin'),
    {},
  )
})

test('explicit fallback flag switches source-capable targets off the prebuilt path', () => {
  assert.equal(
    prepareModule.resolvePrepareStrategy(
      'x86_64-unknown-linux-gnu',
      { MOTRIX_SIDECAR_REGENERATE: '1' },
    ),
    'source-build',
  )
  assert.equal(
    prepareModule.resolvePrepareStrategy(
      'aarch64-apple-darwin',
      { MOTRIX_SIDECAR_BUILD_FROM_SOURCE: '1' },
    ),
    'source-build',
  )
  assert.equal(
    prepareModule.resolvePrepareStrategy(
      'x86_64-pc-windows-msvc',
      { MOTRIX_SIDECAR_REGENERATE: '1' },
    ),
    'windows-archive',
  )
})

test('system aria2c reuse is opt-in and must match target platform', () => {
  assert.equal(
    prepareModule.shouldReuseSystemAria2c('x86_64-unknown-linux-gnu', {}),
    false,
  )

  const linuxReuse = prepareModule.shouldReuseSystemAria2c(
    'x86_64-unknown-linux-gnu',
    { MOTRIX_REUSE_SYSTEM_ARIA2C: '1' },
    'linux',
  )
  assert.equal(linuxReuse, true)

  const windowsCrossReuse = prepareModule.shouldReuseSystemAria2c(
    'x86_64-pc-windows-gnu',
    { MOTRIX_REUSE_SYSTEM_ARIA2C: '1' },
    'linux',
  )
  assert.equal(windowsCrossReuse, false)
})

test('windows archive extraction uses a real zip extractor on non-Windows hosts', () => {
  const command = prepareModule.getWindowsArchiveExtractCommand(
    '/tmp/aria2.zip',
    '/tmp/extract',
    'linux',
  )

  assert.equal(command.command, 'unzip')
  assert.deepEqual(command.args, ['-q', '-o', '/tmp/aria2.zip', '-d', '/tmp/extract'])
})

test('linux docker build command is pinned to the manifest image and target', () => {
  const command = prepareModule.getLinuxDockerBuildCommand(
    '/repo',
    'x86_64-unknown-linux-gnu',
  )

  assert.equal(command.command, 'docker')
  assert.ok(command.args.includes('node:22-bullseye'))
  assert.ok(command.args.includes('TARGET_TRIPLE=x86_64-unknown-linux-gnu'))
  assert.ok(command.args.includes('LIBSSH2_LIBS=-lssh2 -lgcrypt -lgpg-error -lz'))
  assert.ok(command.args.some((arg) => arg.includes('/repo:/workspace')))
})

test('source build workspace is isolated by target and build context', () => {
  const artifact = { extractDir: 'aria2-1.37.0' }

  assert.equal(
    prepareModule.getSourceBuildDir('/repo', 'x86_64-unknown-linux-gnu', artifact, 'native'),
    '/repo/.sidecar-cache/build/x86_64-unknown-linux-gnu/native/aria2-1.37.0',
  )
  assert.equal(
    prepareModule.getSourceBuildDir('/repo', 'x86_64-unknown-linux-gnu', artifact, 'docker'),
    '/repo/.sidecar-cache/build/x86_64-unknown-linux-gnu/docker/aria2-1.37.0',
  )
})

test('findMissingStaticLibraries accepts compiler-reported libstdc++ path', async () => {
  const manifest = {
    targets: {
      'x86_64-unknown-linux-gnu': {
        requiredStaticLibraries: [
          {
            name: 'libstdc++.a',
            paths: ['/missing/libstdc++.a'],
            resolveCommands: [
              ['g++', '-print-file-name=libstdc++.a'],
            ],
          },
        ],
      },
    },
  }

  const missing = await prepareModule.findMissingStaticLibraries(
    'x86_64-unknown-linux-gnu',
    manifest,
    {
      pathExists: async (candidate) => candidate === '/toolchain/libstdc++.a',
      resolveCommand: async (command, args) => {
        assert.equal(command, 'g++')
        assert.deepEqual(args, ['-print-file-name=libstdc++.a'])
        return '/toolchain/libstdc++.a\n'
      },
    },
  )

  assert.deepEqual(missing, [])
})

test('missing build tools message is actionable on linux', () => {
  const message = prepareModule.formatMissingBuildToolsMessage(
    'x86_64-unknown-linux-gnu',
    ['autoreconf', 'make'],
  )

  assert.match(message, /Missing build tools for aria2 source build/)
  assert.match(message, /autoreconf, make/)
  assert.match(message, /apt install/)
  assert.match(message, /libexpat1-dev/)
  assert.match(message, /libgpg-error-dev/)
})

test('missing unzip message is actionable for Windows sidecar extraction', () => {
  const message = prepareModule.formatMissingArchiveToolMessage('unzip')

  assert.match(message, /Missing archive tool/)
  assert.match(message, /unzip/)
  assert.match(message, /apt install unzip/)
})
