import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AppConfig } from '@/types'
import { invoke } from '@tauri-apps/api/core'

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
  async function init() {
    if (initialized.value) return

    loading.value = true
    try {
      config.value = await invoke<AppConfig>('get_app_config')
      initialized.value = true
    } catch (error) {
      console.error('Failed to load config:', error)
      // Use default config
      config.value = getDefaultConfig()
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(newConfig: Partial<AppConfig>) {
    if (!config.value) return

    const updated = { ...config.value, ...newConfig }
    try {
      await invoke('save_app_config', { config: updated })
      config.value = updated

      // Sync download-related options to aria2 engine in real-time
      const aria2Keys: Record<string, string> = {
        maxConcurrentDownloads: 'max-concurrent-downloads',
        maxConnectionPerServer: 'max-connection-per-server',
        split: 'split',
        maxDownloadLimit: 'max-overall-download-limit',
        maxUploadLimit: 'max-overall-upload-limit',
        userAgent: 'user-agent',
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

  async function setLocale(locale: string) {
    await saveConfig({ locale })
  }

  async function setDownloadDir(dir: string) {
    await saveConfig({ downloadDir: dir })
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
      trackerSource: ['https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_best.txt'],
      userAgent: 'Motrix/2.0.0',
      proxyEnabled: false,
      proxyType: 'http',
      proxyHost: '',
      proxyPort: 1080,
      proxyUsername: '',
      proxyPassword: '',
      rpcPort: 16800,
      rpcSecret: '',
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
  }
})
