/**
 * Format bytes to human readable string
 */
export function formatBytes(bytes: number | string, decimals = 2): string {
  const b = typeof bytes === 'string' ? parseInt(bytes) : bytes
  if (!isFinite(b) || b <= 0) return '0 B'

  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']

  const i = Math.floor(Math.log(b) / Math.log(k))
  return parseFloat((b / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

/**
 * Format speed to human readable string
 */
export function formatSpeed(bytesPerSecond: number | string): string {
  return formatBytes(bytesPerSecond) + '/s'
}

/**
 * Format duration in seconds to human readable string
 */
export function formatDuration(seconds: number): string {
  if (!isFinite(seconds) || seconds < 0) return '--'

  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)

  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  }
  return `${m}:${s.toString().padStart(2, '0')}`
}

/**
 * Calculate remaining time
 */
export function calcRemainingTime(
  totalLength: number | string,
  completedLength: number | string,
  downloadSpeed: number | string
): number {
  const total = typeof totalLength === 'string' ? parseInt(totalLength) : totalLength
  const completed = typeof completedLength === 'string' ? parseInt(completedLength) : completedLength
  const speed = typeof downloadSpeed === 'string' ? parseInt(downloadSpeed) : downloadSpeed

  if (speed === 0) return Infinity
  return (total - completed) / speed
}

/**
 * Calculate progress percentage
 */
export function calcProgress(
  totalLength: number | string,
  completedLength: number | string
): number {
  const total = typeof totalLength === 'string' ? parseInt(totalLength) : totalLength
  const completed = typeof completedLength === 'string' ? parseInt(completedLength) : completedLength

  if (total === 0) return 0
  return Math.round((completed / total) * 100)
}

/**
 * Get task name from task object
 */
export function getTaskName(task: { files?: { path?: string }[]; bittorrent?: { info?: { name?: string } } }): string {
  // Try bittorrent name first
  if (task.bittorrent?.info?.name) {
    return task.bittorrent.info.name
  }

  // Try first file path
  if (task.files?.[0]?.path) {
    const path = task.files[0].path
    return path.split('/').pop() || path.split('\\').pop() || 'Unknown'
  }

  return 'Unknown'
}

/**
 * Check if task is BT task
 */
export function isBtTask(task: { bittorrent?: unknown }): boolean {
  return !!task.bittorrent
}

/**
 * Decode thunder:// URL to real download URL
 * thunder:// format: thunder://BASE64(AA<real_url>ZZ)
 */
export function decodeThunderUrl(url: string): string {
  if (!url.toLowerCase().startsWith('thunder://')) return url
  try {
    const encoded = url.slice(10) // Skip "thunder://"
    const decoded = atob(encoded)
    // thunder wraps URL with "AA" prefix and "ZZ" suffix
    const trimmed = decoded.replace(/^AA/, '').replace(/ZZ$/, '')
    return trimmed || url
  } catch {
    return url
  }
}

/**
 * Parse a cURL command string into URL and options
 */
export function parseCurlCommand(input: string): { url: string; headers: Record<string, string>; output?: string; proxy?: string } | null {
  const trimmed = input.trim()
  if (!trimmed.toLowerCase().startsWith('curl ')) return null

  const result: { url: string; headers: Record<string, string>; output?: string; proxy?: string } = {
    url: '',
    headers: {},
  }

  // Tokenize respecting quotes
  const tokens: string[] = []
  let current = ''
  let inSingle = false
  let inDouble = false
  let escaped = false

  for (const ch of trimmed) {
    if (escaped) {
      current += ch
      escaped = false
      continue
    }
    if (ch === '\\') {
      escaped = true
      continue
    }
    if (ch === "'" && !inDouble) {
      inSingle = !inSingle
      continue
    }
    if (ch === '"' && !inSingle) {
      inDouble = !inDouble
      continue
    }
    if ((ch === ' ' || ch === '\t') && !inSingle && !inDouble) {
      if (current) {
        tokens.push(current)
        current = ''
      }
      continue
    }
    current += ch
  }
  if (current) tokens.push(current)

  // Parse tokens (skip "curl")
  let i = 1
  while (i < tokens.length) {
    const token = tokens[i]
    if (token === '-H' || token === '--header') {
      i++
      if (i < tokens.length) {
        const header = tokens[i]
        const colonIdx = header.indexOf(':')
        if (colonIdx > 0) {
          const key = header.slice(0, colonIdx).trim()
          const value = header.slice(colonIdx + 1).trim()
          result.headers[key] = value
        }
      }
    } else if (token === '-o' || token === '--output') {
      i++
      if (i < tokens.length) result.output = tokens[i]
    } else if (token === '-x' || token === '--proxy') {
      i++
      if (i < tokens.length) result.proxy = tokens[i]
    } else if (token === '-A' || token === '--user-agent') {
      i++
      if (i < tokens.length) result.headers['User-Agent'] = tokens[i]
    } else if (token === '-e' || token === '--referer') {
      i++
      if (i < tokens.length) result.headers['Referer'] = tokens[i]
    } else if (token === '-b' || token === '--cookie') {
      i++
      if (i < tokens.length) result.headers['Cookie'] = tokens[i]
    } else if (!token.startsWith('-') && !result.url) {
      result.url = token
    }
    i++
  }

  return result.url ? result : null
}
