import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AppConfig } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { i18n } from '@/main'

export const useAppStore = defineStore('app', () => {
  // State
  const config = ref<AppConfig | null>(null)
  const loading = ref(false)
  const initialized = ref(false)
  const engineReady = ref(false)
  const engineError = ref('')

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

  async function saveConfig(newConfig: Partial<AppConfig>) {
    if (!config.value) return

    const updated = { ...config.value, ...newConfig }
    try {
      await invoke('save_app_config', { config: updated })
      config.value = updated

      // Sync download-related options to aria2 engine in real-time
      // Only global options that can be changed at runtime via aria2 changeGlobalOption
      // Note: split, max-connection-per-server are per-task options, not global
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
      }

      const engineOptions: Record<string, string> = {}
      for (const [configKey, aria2Key] of Object.entries(aria2Keys)) {
        if (configKey in newConfig) {
          engineOptions[aria2Key] = String((newConfig as Record<string, unknown>)[configKey])
        }
      }

      if (Object.keys(engineOptions).length > 0) {
        try {
          await invoke('change_global_option', { options: engineOptions })
        } catch (e) {
          console.warn('Failed to sync options to aria2:', e)
        }
      }
    } catch (error) {
      console.error('Failed to save config:', error)
      throw error
    }
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

  async function addRecentDir(dir: string) {
    if (!config.value) return
    const dirs = config.value.recentDirs.filter(d => d !== dir)
    dirs.unshift(dir)
    if (dirs.length > 10) dirs.length = 10
    await saveConfig({ recentDirs: dirs })
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
      runMode: 'tray',
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
      proxyScope: 'all',
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
      keepSeeding: true,
      traySpeedometer: false,
      recentDirs: [],
      favoriteDirs: [],
      rpcListenAll: false,
    }
  }

  return {
    // State
    config,
    loading,
    initialized,
    engineReady,
    engineError,
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
    addRecentDir,
    resetConfig,
    autoSyncTrackers,
  }
})
