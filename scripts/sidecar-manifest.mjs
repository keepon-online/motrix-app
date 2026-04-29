import path from 'node:path'
import { readFileSync } from 'node:fs'

export const SIDECAR_MANIFEST_FILE = 'sidecar-manifest.json'

export function loadSidecarManifest(repoRoot = process.cwd()) {
  const manifestPath = path.join(repoRoot, SIDECAR_MANIFEST_FILE)
  try {
    return JSON.parse(readFileSync(manifestPath, 'utf8'))
  } catch (error) {
    throw new Error(`Failed to load ${SIDECAR_MANIFEST_FILE}: ${error.message}`)
  }
}

export function getSidecarTarget(manifest, targetTriple) {
  const target = manifest.targets?.[targetTriple]
  if (!target) {
    const knownTargets = Object.keys(manifest.targets ?? {}).sort().join(', ')
    throw new Error(
      `Unsupported aria2 sidecar target: ${targetTriple}. Add it to ${SIDECAR_MANIFEST_FILE}. Known targets: ${knownTargets}`
    )
  }
  return target
}

export function getSidecarArtifactByName(manifest, artifactName) {
  const artifact = manifest.artifacts?.[artifactName]
  if (!artifact) {
    throw new Error(`Sidecar artifact not found in manifest: ${artifactName}`)
  }
  return artifact
}

export function getSidecarArtifact(manifest, target) {
  return getSidecarArtifactByName(manifest, target.artifact)
}

export function getSidecarValidation(manifest, targetTriple) {
  return getSidecarTarget(manifest, targetTriple).validation ?? {}
}
