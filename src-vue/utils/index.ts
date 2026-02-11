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
