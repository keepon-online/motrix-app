import test from 'node:test'
import assert from 'node:assert/strict'

const prepareModule = await import('../scripts/prepare-sidecar.mjs')

test('getPrepareStrategy uses windows archive only for windows targets', () => {
  assert.equal(
    prepareModule.getPrepareStrategy('x86_64-pc-windows-msvc'),
    'windows-archive',
  )
  assert.equal(
    prepareModule.getPrepareStrategy('x86_64-unknown-linux-gnu'),
    'source-build',
  )
  assert.equal(
    prepareModule.getPrepareStrategy('aarch64-apple-darwin'),
    'source-build',
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

test('configure args use platform-appropriate TLS backend', () => {
  assert.deepEqual(
    prepareModule.getConfigureArgs('x86_64-unknown-linux-gnu'),
    ['--without-gnutls', '--with-openssl', '--without-libgcrypt'],
  )
  assert.deepEqual(
    prepareModule.getConfigureArgs('aarch64-apple-darwin'),
    ['--without-gnutls', '--with-appletls', '--without-libgcrypt'],
  )
})

test('missing build tools message is actionable on linux', () => {
  const message = prepareModule.formatMissingBuildToolsMessage(
    'x86_64-unknown-linux-gnu',
    ['autoreconf', 'make'],
  )

  assert.match(message, /Missing build tools for aria2 source build/)
  assert.match(message, /autoreconf, make/)
  assert.match(message, /apt install/)
})
