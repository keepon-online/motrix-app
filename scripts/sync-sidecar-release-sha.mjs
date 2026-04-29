import fs from 'node:fs/promises'
import path from 'node:path'

import { SIDECAR_MANIFEST_FILE, loadSidecarManifest } from './sidecar-manifest.mjs'

function parseShaFile(content) {
  const hashes = new Map()
  for (const line of content.split(/\r?\n/)) {
    const match = line.match(/^([a-f0-9]{64})\s+\*?(.+)$/)
    if (!match) continue
    hashes.set(match[2].trim(), match[1])
  }
  return hashes
}

async function main() {
  const repoRoot = process.cwd()
  const shaFile = process.argv[2]
  if (!shaFile) {
    throw new Error('Usage: node scripts/sync-sidecar-release-sha.mjs <SHA256SUMS.txt>')
  }

  const manifestPath = path.join(repoRoot, SIDECAR_MANIFEST_FILE)
  const manifest = loadSidecarManifest(repoRoot)
  const shaContent = await fs.readFile(path.resolve(repoRoot, shaFile), 'utf8')
  const hashes = parseShaFile(shaContent)

  let updated = 0
  for (const artifact of Object.values(manifest.artifacts ?? {})) {
    if (!artifact.fileName) continue
    const sha = hashes.get(artifact.fileName)
    if (!sha) continue
    artifact.sha256 = sha
    updated += 1
  }

  if (updated === 0) {
    throw new Error(`No matching sidecar artifacts found in ${shaFile}`)
  }

  await fs.writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
  console.log(`Updated ${updated} sidecar checksums in ${SIDECAR_MANIFEST_FILE}`)
}

main().catch((error) => {
  console.error(error.message)
  process.exit(1)
})
