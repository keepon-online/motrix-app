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
 * Check if a string is a downloadable URL
 */
export function isUrl(text: string): boolean {
  return /^(https?|ftp|magnet|thunder):\/\/\S/i.test(text.trim())
}

import type { TaskStatus } from '@/types'

/**
 * Map task status to localized label
 */
export function getTaskStatusText(status: TaskStatus, t: (key: string) => string): string {
  const statusMap: Record<string, string> = {
    active: t('task.downloading'),
    waiting: t('task.waiting'),
    paused: t('task.paused'),
    error: t('task.error'),
    complete: t('task.completed'),
    removed: t('task.removed'),
  }
  return statusMap[status] || status
}

/**
 * Shared task action helpers (open file, show in folder, copy link)
 */
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

export async function openFile(path: string) {
  try {
    await invoke('open_file', { path })
  } catch (e) {
    ElMessage.warning(String(e))
  }
}

export async function showInFolder(path: string) {
  try {
    await invoke('show_in_folder', { path })
  } catch (e) {
    ElMessage.warning(String(e))
  }
}

export async function copyLink(url: string) {
  try {
    await navigator.clipboard.writeText(url)
  } catch {
    // Fallback for Tauri
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
    await writeText(url)
  }
}
