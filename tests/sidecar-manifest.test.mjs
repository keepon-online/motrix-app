import test from 'node:test'
import assert from 'node:assert/strict'

const manifestModule = await import('../scripts/sidecar-manifest.mjs')

test('sidecar manifest defines every packaged target with a strategy and artifact', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())
  const targets = Object.keys(manifest.targets).sort()

  assert.deepEqual(targets, [
    'aarch64-apple-darwin',
    'x86_64-apple-darwin',
    'x86_64-pc-windows-gnu',
    'x86_64-pc-windows-msvc',
    'x86_64-unknown-linux-gnu',
  ])

  for (const targetTriple of targets) {
    const target = manifestModule.getSidecarTarget(manifest, targetTriple)
    assert.ok(target.strategy, `${targetTriple} missing strategy`)
    assert.ok(target.artifact, `${targetTriple} missing artifact`)
    assert.ok(
      manifest.artifacts[target.artifact],
      `${targetTriple} references missing artifact ${target.artifact}`,
    )
  }
})

test('manifest pins third-party upstream artifacts with sha256 checksums', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())

  for (const [artifactName, artifact] of Object.entries(manifest.artifacts)) {
    if (/^https:\/\/github\.com\/aria2\/aria2\/releases\//.test(artifact.url)) {
      assert.match(
        artifact.sha256,
        /^[a-f0-9]{64}$/,
        `${artifactName} must pin a SHA256 checksum`,
      )
    }
  }
})

test('manifest defaults packaged targets to repository prebuilt sidecars', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())

  for (const targetTriple of [
    'aarch64-apple-darwin',
    'x86_64-apple-darwin',
    'x86_64-pc-windows-gnu',
    'x86_64-pc-windows-msvc',
    'x86_64-unknown-linux-gnu',
  ]) {
    const target = manifestModule.getSidecarTarget(manifest, targetTriple)
    const artifact = manifestModule.getSidecarArtifact(manifest, target)
    assert.equal(target.strategy, 'release-download')
    assert.match(
      artifact.url,
      /^https:\/\/github\.com\/keepon-online\/motrix-app\/releases\/download\/aria2-sidecar-/,
      `${targetTriple} should download the repository-managed prebuilt sidecar by default`,
    )
  }
})

test('manifest keeps Linux source build as an explicit fallback path only', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())
  const linuxTarget = manifestModule.getSidecarTarget(
    manifest,
    'x86_64-unknown-linux-gnu',
  )

  assert.equal(linuxTarget.strategy, 'release-download')
  assert.equal(linuxTarget.fallbackStrategy, 'source-build')
  assert.equal(linuxTarget.fallbackArtifact, 'aria2-source')
  assert.equal(linuxTarget.validation.format, 'elf')
  assert.equal(linuxTarget.validation.portableLinux, true)
  assert.equal(linuxTarget.configureEnv.ARIA2_STATIC, 'yes')
  assert.equal(linuxTarget.dockerBuild.image, 'node:22-bullseye')
  assert.equal(
    linuxTarget.dockerBuild.env.LIBSSH2_LIBS,
    '-lssh2 -lgcrypt -lgpg-error -lz',
  )
  assert.deepEqual(
    linuxTarget.requiredStaticLibraries.find((library) => library.name === 'libstdc++.a')?.resolveCommands,
    [
      ['g++', '-print-file-name=libstdc++.a'],
      ['gcc', '-print-file-name=libstdc++.a'],
    ],
  )
  assert.ok(
    linuxTarget.dockerBuild.packages.includes('libgpg-error-dev'),
    'linux docker build must install the static libgpg-error dependency',
  )
})

test('manifest pins macOS validation to Mach-O format and target architecture', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())
  const armTarget = manifestModule.getSidecarTarget(manifest, 'aarch64-apple-darwin')
  const intelTarget = manifestModule.getSidecarTarget(manifest, 'x86_64-apple-darwin')

  assert.equal(armTarget.validation.format, 'macho')
  assert.equal(armTarget.validation.arch, 'arm64')
  assert.equal(intelTarget.validation.format, 'macho')
  assert.equal(intelTarget.validation.arch, 'x86_64')
})

test('manifest keeps upstream windows archive only as an explicit fallback source', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())
  const windowsTarget = manifestModule.getSidecarTarget(
    manifest,
    'x86_64-pc-windows-msvc',
  )
  const fallbackArtifact = manifest.artifacts[windowsTarget.fallbackArtifact]

  assert.equal(windowsTarget.strategy, 'release-download')
  assert.equal(windowsTarget.fallbackStrategy, 'windows-archive')
  assert.match(
    fallbackArtifact.url,
    /^https:\/\/github\.com\/aria2\/aria2\/releases\//,
  )
})

test('repository-managed prebuilt artifacts pin target-specific file names', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())
  const linuxArtifact = manifest.artifacts['motrix-sidecar-linux-x86_64']
  const macArtifact = manifest.artifacts['motrix-sidecar-macos-aarch64']
  const windowsArtifact = manifest.artifacts['motrix-sidecar-windows-x86_64-msvc']

  assert.equal(linuxArtifact.fileName, 'motrix-aria2c-x86_64-unknown-linux-gnu')
  assert.equal(macArtifact.fileName, 'motrix-aria2c-aarch64-apple-darwin')
  assert.equal(windowsArtifact.fileName, 'motrix-aria2c-x86_64-pc-windows-msvc.exe')
})

test('manifest rejects unknown target triples', () => {
  const manifest = manifestModule.loadSidecarManifest(process.cwd())

  assert.throws(
    () => manifestModule.getSidecarTarget(manifest, 'armv7-unknown-linux-gnueabihf'),
    /Unsupported aria2 sidecar target/,
  )
})
