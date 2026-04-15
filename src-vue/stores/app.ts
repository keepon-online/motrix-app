import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AppConfig } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { i18n } from '@/main'

/** Debounce helper: delays execution until `wait` ms after the last call */
function debounce<T extends (...args: unknown[]) => void>(fn: T, wait: number): T {
  let timer: ReturnType<typeof setTimeout> | null = null
  return ((...args: unknown[]) => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => fn(...args), wait)
  }) as T
}

export const useAppStore = defineStore('app', () => {
  // State
  const config = ref<AppConfig | null>(null)
  const loading = ref(false)
  const initialized = ref(false)

  // Getters
  const isDark = computed(() => {
    if (!config.value) return false
    if (config.value.theme === 'dark') return true
    if (config.value.theme === 'light') return false
    // Auto: check system preference
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  })

  const locale = computed(() => config.value?.locale ?? 'en')
  const downloadDir = computed(() => config.value?.downloadDir ?? '')

  // Actions
  async function updateTrayMenu() {
    const t = i18n.global.t
    try {
      await invoke('update_tray_menu', {
        labels: {
          show: t('tray.show'),
          pauseAll: t('tray.pauseAll'),
          resumeAll: t('tray.resumeAll'),
          quit: t('tray.quit'),
        }
      })
    } catch (e) {
      console.warn('Failed to update tray menu:', e)
    }
  }

  async function init() {
    if (initialized.value) return

    loading.value = true
    try {
      config.value = await invoke<AppConfig>('get_app_config')
      initialized.value = true
      // Restore saved locale to i18n
      if (config.value.locale) {
        ;(i18n.global.locale as unknown as { value: string }).value = config.value.locale
      }
      // Migrate old single tracker source to new multi-source defaults
      await migrateTrackerSources()
      // Update tray menu with current locale
      await updateTrayMenu()
    } catch (error) {
      console.error('Failed to load config:', error)
      // Use default config
      config.value = getDefaultConfig()
    } finally {
      loading.value = false
    }
  }

  const DEFAULT_TRACKER_SOURCES = [
    'https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_all.txt',
    'https://raw.githubusercontent.com/XIU2/TrackersListCollection/master/all.txt',
    'https://raw.githubusercontent.com/DeSireFire/animeTrackerList/master/AT_all.txt',
  ]

  async function migrateTrackerSources() {
    if (!config.value) return
    const sources = config.value.trackerSource
    // Migrate if user still has only the old single default source
    const oldDefault = 'https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_best.txt'
    if (sources.length === 1 && sources[0] === oldDefault) {
      await saveConfig({ trackerSource: DEFAULT_TRACKER_SOURCES, lastTrackerUpdate: 0 })
    }
  }

  /** Map of frontend config keys to aria2 global option names */
  const aria2Keys: Record<string, string> = {
    maxConcurrentDownloads: 'max-concurrent-downloads',
    maxDownloadLimit: 'max-download-limit',
    maxUploadLimit: 'max-upload-limit',
    maxOverallDownloadLimit: 'max-overall-download-limit',
    maxOverallUploadLimit: 'max-overall-upload-limit',
    userAgent: 'user-agent',
    btForceEncryption: 'bt-force-encryption',
    btRequireCrypto: 'bt-require-crypto',
    followMetalink: 'follow-metalink',
    btSaveMetadata: 'bt-save-metadata',
    btLoadSavedMetadata: 'bt-load-saved-metadata',
    btRemoveUnselectedFile: 'bt-remove-unselected-file',
    btDetachSeedOnly: 'bt-detach-seed-only',
    seedRatio: 'seed-ratio',
    seedTime: 'seed-time',
    btTracker: 'bt-tracker',
    enableUpnp: 'enable-upnp',
    allowOverwrite: 'allow-overwrite',
    autoFileRenaming: 'auto-file-renaming',
    continueDownload: 'continue-download',
    maxConnectionPerServer: 'max-connection-per-server',
    split: 'split',
    minSplitSize: 'min-split-size',
  }

  /** Sync relevant config keys to aria2 engine at runtime */
  async function syncToAria2(newConfig: Partial<AppConfig>) {
    const engineOptions: Record<string, string> = {}
    for (const [configKey, aria2Key] of Object.entries(aria2Keys)) {
      if (configKey in newConfig) {
        engineOptions[aria2Key] = String((newConfig as Record<string, unknown>)[configKey])
      }
    }

    // Handle proxy as a composed option
    if ('proxyEnabled' in newConfig || 'proxyHost' in newConfig || 'proxyPort' in newConfig
      || 'proxyType' in newConfig || 'proxyUsername' in newConfig) {
      const c = config.value
      if (c && c.proxyEnabled && c.proxyHost) {
        const auth = c.proxyUsername ? `${c.proxyUsername}:${c.proxyPassword}@` : ''
        engineOptions['all-proxy'] = `${c.proxyType}://${auth}${c.proxyHost}:${c.proxyPort}`
      } else {
        engineOptions['all-proxy'] = ''
      }
    }

    if ('noProxy' in newConfig && config.value) {
      engineOptions['no-proxy'] = config.value.noProxy
    }

    if (Object.keys(engineOptions).length > 0) {
      try {
        await invoke('change_global_option', { options: engineOptions })
      } catch (e) {
        console.warn('Failed to sync options to aria2:', e)
      }
    }
  }

  /** Debounced persist + sync — coalesces rapid changes (e.g. spinner clicks) */
  const debouncedSave = debounce(async (updated: AppConfig, newConfig: Partial<AppConfig>) => {
    try {
      await invoke('save_app_config', { config: updated })
      await syncToAria2(newConfig)
    } catch (error) {
      console.error('Failed to save config:', error)
    }
  }, 500)

  async function saveConfig(newConfig: Partial<AppConfig>) {
    if (!config.value) return

    const updated = { ...config.value, ...newConfig }
    config.value = updated
    debouncedSave(updated, newConfig)
  }

  async function setTheme(theme: 'auto' | 'light' | 'dark') {
    await saveConfig({ theme })
  }

  async function setLocale(newLocale: string) {
    ;(i18n.global.locale as unknown as { value: string }).value = newLocale
    await saveConfig({ locale: newLocale })
    await updateTrayMenu()
  }

  async function setDownloadDir(dir: string) {
    await saveConfig({ downloadDir: dir })
  }

  const TRACKER_SYNC_INTERVAL = 12 * 60 * 60 * 1000 // 12 hours

  async function autoSyncTrackers() {
    if (!config.value) return
    const lastUpdate = config.value.lastTrackerUpdate || 0
    const now = Date.now()
    if (now - lastUpdate < TRACKER_SYNC_INTERVAL) return
    if (!config.value.trackerSource || config.value.trackerSource.length === 0) return

    try {
      const trackers = await invoke<string[]>('fetch_tracker_list', { sources: config.value.trackerSource })
      if (trackers.length > 0) {
        const btTracker = trackers.join(',')
        await saveConfig({ btTracker, lastTrackerUpdate: now })
        // Also update aria2 engine
        await invoke('change_global_option', { options: { 'bt-tracker': btTracker } }).catch(() => {})
      }
    } catch (e) {
      console.warn('Auto tracker sync failed:', e)
    }
  }

  async function resetConfig() {
    const defaults = getDefaultConfig()
    // Keep locale and rpcSecret as current
    defaults.locale = config.value?.locale ?? 'en'
    defaults.rpcSecret = config.value?.rpcSecret ?? ''
    try {
      await invoke('save_app_config', { config: defaults })
      config.value = defaults
    } catch (error) {
      console.error('Failed to reset config:', error)
      throw error
    }
  }

  function getDefaultConfig(): AppConfig {
    return {
      locale: 'en',
      theme: 'auto',
      downloadDir: '',
      autoStart: false,
      startHidden: false,
      hideOnClose: true,
      notifyOnComplete: true,
      autoClearCompleted: false,
      resumeAllWhenAppLaunched: true,
      maxConcurrentDownloads: 10,
      maxConnectionPerServer: 16,
      split: 16,
      minSplitSize: '1M',
      maxDownloadLimit: '0',
      maxUploadLimit: '0',
      btListenPort: 21301,
      dhtListenPort: 21302,
      enableUpnp: true,
      seedRatio: 1.0,
      seedTime: 60,
      btTracker: '',
      trackerSource: [
        'https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_all.txt',
        'https://raw.githubusercontent.com/XIU2/TrackersListCollection/master/all.txt',
        'https://raw.githubusercontent.com/DeSireFire/animeTrackerList/master/AT_all.txt',
      ],
      btForceEncryption: false,
      btRequireCrypto: false,
      pauseMetadata: false,
      userAgent: `Motrix/${__APP_VERSION__}`,
      proxyEnabled: false,
      proxyType: 'http',
      proxyHost: '',
      proxyPort: 1080,
      proxyUsername: '',
      proxyPassword: '',
      noProxy: '',
      rpcPort: 16800,
      rpcSecret: '',
      maxOverallDownloadLimit: '0',
      maxOverallUploadLimit: '0',
      allowOverwrite: false,
      autoFileRenaming: true,
      continueDownload: true,
      followMetalink: 'true',
      lastTrackerUpdate: 0,
      btSaveMetadata: true,
      btLoadSavedMetadata: true,
      btRemoveUnselectedFile: false,
      btDetachSeedOnly: false,
    }
  }

  return {
    // State
    config,
    loading,
    initialized,
    // Getters
    isDark,
    locale,
    downloadDir,
    // Actions
    init,
    saveConfig,
    setTheme,
    setLocale,
    setDownloadDir,
    resetConfig,
    autoSyncTrackers,
  }
})
