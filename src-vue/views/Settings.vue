<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { useAria2Diagnostics } from '@/composables/useAria2Diagnostics'
import { useTheme } from '@/composables/useTheme'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { UpnpStatus } from '@/types'

const { t } = useI18n()
const appStore = useAppStore()
const { setTheme } = useTheme()
const {
  diagnostics: aria2Diagnostics,
  panelStatus: aria2PanelStatus,
  markRestartStarted,
  markRestartFailed,
  markRestartSucceeded,
} = useAria2Diagnostics()

const trackerInput = ref('')
const trackerUpdating = ref(false)
const upnpStatus = ref<UpnpStatus | null>(null)
const upnpLoading = ref(false)

// Engine paths (developer section)
const enginePaths = ref<Record<string, string>>({})
const logLevel = ref('info')

// Directory history popover
const showDirPopover = ref(false)

// Extract friendly name from tracker source URL
function getSourceName(url: string): string {
  try {
    const u = new URL(url)
    const parts = u.pathname.split('/').filter(Boolean)
    if (u.hostname === 'raw.githubusercontent.com' && parts.length >= 2) {
      return `${parts[0]}/${parts[1]}`
    }
    if (u.hostname === 'cdn.jsdelivr.net' && parts.length >= 3) {
      return `${parts[1]}/${parts[2]}`
    }
    const filename = parts[parts.length - 1] || ''
    return `${u.hostname}/${filename}`
  } catch {
    return url
  }
}

// Count loaded trackers
const trackerCount = computed(() => {
  const bt = appStore.config?.btTracker
  if (!bt) return 0
  return bt.split(',').filter(Boolean).length
})

// Speed limit presets (in B/s)
const speedOptions = [
  { label: t('settings.speedUnlimited'), value: '0' },
  { label: '128 KB/s', value: '131072' },
  { label: '256 KB/s', value: '262144' },
  { label: '512 KB/s', value: '524288' },
  { label: '1 MB/s', value: '1048576' },
  { label: '2 MB/s', value: '2097152' },
  { label: '5 MB/s', value: '5242880' },
  { label: '10 MB/s', value: '10485760' },
]

// Min split size options
const minSplitSizeOptions = [
  { label: '1M', value: '1M' },
  { label: '2M', value: '2M' },
  { label: '4M', value: '4M' },
  { label: '8M', value: '8M' },
  { label: '16M', value: '16M' },
  { label: '20M', value: '20M' },
]

// Log level options
const logLevelOptions = [
  { label: 'Error', value: 'error' },
  { label: 'Warn', value: 'warn' },
  { label: 'Info', value: 'info' },
  { label: 'Debug', value: 'debug' },
  { label: 'Trace', value: 'trace' },
]

// User-Agent presets
const userAgentPresets = [
  { label: t('settings.userAgentPresetAria2'), value: 'aria2/1.37.0' },
  { label: t('settings.userAgentPresetTransmission'), value: 'Transmission/3.00' },
  { label: t('settings.userAgentPresetChrome'), value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' },
  { label: t('settings.userAgentPresetBaidu'), value: 'netdisk' },
]

// Last check update time formatted
const lastCheckUpdateText = computed(() => {
  const ts = appStore.config?.lastCheckUpdateTime
  if (!ts) return ''
  const d = new Date(ts)
  return d.toLocaleString()
})

function formatTimestamp(ts: number | null): string {
  if (!ts) return ''
  return new Date(ts).toLocaleString()
}

const aria2StatusLabel = computed(() => {
  switch (aria2PanelStatus.value) {
    case 'connected':
      return t('settings.aria2StatusConnected')
    case 'starting':
      return t('settings.aria2StatusStarting')
    case 'restarting':
      return t('settings.aria2StatusRestarting')
    case 'reconnecting':
      return t('settings.aria2StatusReconnecting')
    case 'disconnected':
      return t('settings.aria2StatusDisconnected')
    case 'terminated':
      return t('settings.aria2StatusTerminated')
  }
})

const aria2StatusTagType = computed(() => {
  switch (aria2PanelStatus.value) {
    case 'connected':
      return 'success'
    case 'starting':
    case 'restarting':
      return 'warning'
    case 'reconnecting':
      return 'info'
    case 'disconnected':
    case 'terminated':
      return 'danger'
  }
})

const aria2LastErrorText = computed(() => aria2Diagnostics.value.lastError ?? t('settings.aria2NoRecentError'))
const aria2LastErrorTimeText = computed(() => formatTimestamp(aria2Diagnostics.value.lastErrorAt))
const aria2LastSuccessText = computed(() => {
  const formatted = formatTimestamp(aria2Diagnostics.value.lastSuccessAt)
  return formatted || t('settings.aria2Never')
})
const aria2ActionLabel = computed(() => (
  appStore.restartNeeded ? t('settings.restartNow') : t('settings.retryConnection')
))

// Random helpers
function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function randomString(len: number): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  return Array.from({ length: len }, () => chars[Math.floor(Math.random() * chars.length)]).join('')
}

function randomizeRpcPort() {
  const port = randomInt(16800, 20000)
  appStore.saveConfig({ rpcPort: port })
}

function randomizeRpcSecret() {
  const secret = randomString(16)
  appStore.saveConfig({ rpcSecret: secret })
}

function randomizeBtPort() {
  const port = randomInt(20000, 24999)
  appStore.saveConfig({ btListenPort: port })
}

function randomizeDhtPort() {
  const port = randomInt(25000, 29999)
  appStore.saveConfig({ dhtListenPort: port })
}

// Directory selection with history tracking
async function selectDownloadDir() {
  const selected = await open({
    directory: true,
    multiple: false,
  })
  if (selected) {
    await appStore.setDownloadDir(selected as string)
  }
}

async function selectDirFromHistory(dir: string) {
  await appStore.setDownloadDir(dir)
  showDirPopover.value = false
}

// Trackers management
function addTrackerSource() {
  const url = trackerInput.value.trim()
  if (!url) return

  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    ElMessage.warning(t('settings.trackerInvalidUrl'))
    return
  }

  const current = appStore.config?.trackerSource || []
  if (current.includes(url)) {
    ElMessage.warning(t('settings.trackerExists'))
    return
  }

  appStore.saveConfig({ trackerSource: [...current, url] })
  trackerInput.value = ''
  ElMessage.success(t('settings.trackerAdded'))
}

function removeTrackerSource(url: string) {
  const current = appStore.config?.trackerSource || []
  appStore.saveConfig({ trackerSource: current.filter(t => t !== url) })
}

async function updateTrackers() {
  const sources = appStore.config?.trackerSource || []
  if (sources.length === 0) {
    ElMessage.warning(t('settings.trackerInvalidUrl'))
    return
  }

  trackerUpdating.value = true
  ElMessage.info(t('settings.trackerUpdating'))

  try {
    const trackers = await invoke<string[]>('fetch_tracker_list', { sources })
    const btTracker = trackers.join(',')

    await appStore.saveConfig({ btTracker })
    try {
      await invoke('change_global_option', { options: { 'bt-tracker': btTracker } })
    } catch (e) {
      console.warn('Failed to sync bt-tracker to aria2:', e)
    }
    ElMessage.success(t('settings.trackerCount', { count: trackers.length }))
  } catch {
    ElMessage.error(t('settings.trackerUpdateFailed'))
  } finally {
    trackerUpdating.value = false
  }
}

// Auto start toggle
async function toggleAutoStart(val: string | number | boolean) {
  const enabled = Boolean(val)
  try {
    const { enable, disable } = await import('@tauri-apps/plugin-autostart')
    if (enabled) {
      await enable()
    } else {
      await disable()
    }
    await appStore.saveConfig({ autoStart: enabled })
  } catch (e) {
    console.error('Failed to toggle autostart:', e)
    ElMessage.error(String(e))
  }
}

// Reset actions
async function resetDefaults() {
  try {
    await ElMessageBox.confirm(
      t('settings.resetConfirm'),
      t('settings.resetDefaults'),
      { confirmButtonText: t('settings.resetDefaults'), cancelButtonText: t('dialog.cancel'), type: 'warning' }
    )
    await appStore.resetConfig()
    ElMessage.success(t('settings.resetSuccess'))
  } catch {
    // User cancelled
  }
}

async function resetSession() {
  try {
    await ElMessageBox.confirm(
      t('settings.sessionResetConfirm'),
      t('settings.sessionReset'),
      { confirmButtonText: t('settings.sessionReset'), cancelButtonText: t('dialog.cancel'), type: 'warning' }
    )
    await invoke('reset_session')
    ElMessage.success(t('settings.sessionResetSuccess'))
  } catch {
    // User cancelled or error
  }
}

async function factoryReset() {
  try {
    await ElMessageBox.confirm(
      t('settings.factoryResetConfirm'),
      t('settings.factoryReset'),
      { confirmButtonText: t('settings.factoryReset'), cancelButtonText: t('dialog.cancel'), type: 'error' }
    )
    await invoke('factory_reset')
    ElMessage.success(t('settings.factoryResetSuccess'))
    // Restart the app to apply
    const { relaunch } = await import('@tauri-apps/plugin-process')
    await relaunch()
  } catch {
    // User cancelled or error
  }
}

// Config import/export
async function exportConfig() {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const { writeTextFile } = await import('@tauri-apps/plugin-fs')
    const filePath = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'motrix-config.json',
    })
    if (filePath && appStore.config) {
      await writeTextFile(filePath, JSON.stringify(appStore.config, null, 2))
      ElMessage.success(t('settings.exportSuccess'))
    }
  } catch (e) {
    console.error('Failed to export config:', e)
    ElMessage.error(String(e))
  }
}

async function importConfig() {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (filePath) {
      const { readTextFile } = await import('@tauri-apps/plugin-fs')
      const text = await readTextFile(filePath as string)
      const parsed = JSON.parse(text)
      if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
        throw new Error('Invalid config format')
      }
      const requiredStrings = ['locale', 'theme', 'downloadDir']
      for (const key of requiredStrings) {
        if (key in parsed && typeof parsed[key] !== 'string') {
          throw new Error(`Invalid type for ${key}`)
        }
      }
      await appStore.saveConfig(parsed)
      ElMessage.success(t('settings.importSuccess'))
    }
  } catch (e) {
    console.error('Failed to import config:', e)
    ElMessage.error(String(e))
  }
}

// Engine restart
async function doRestartEngine() {
  markRestartStarted()
  try {
    await appStore.restartEngine()
    markRestartSucceeded()
    ElMessage.success(t('settings.engineRestarted'))
  } catch (e) {
    const message = markRestartFailed(e)
    ElMessage.error(`${t('settings.restartFailed')}: ${message}`)
  }
}

// UPnP status management
async function fetchUpnpStatus() {
  try {
    upnpStatus.value = await invoke<UpnpStatus>('get_upnp_status')
  } catch {
    // ignore
  }
}

async function refreshUpnp() {
  upnpLoading.value = true
  try {
    upnpStatus.value = await invoke<UpnpStatus>('refresh_upnp')
  } catch (e) {
    console.warn('Failed to refresh UPnP:', e)
  } finally {
    upnpLoading.value = false
  }
}

// Re-fetch UPnP status when toggle changes
watch(() => appStore.config?.enableUpnp, () => {
  fetchUpnpStatus()
})

// Load engine paths on mount
onMounted(async () => {
  try {
    enginePaths.value = await invoke<Record<string, string>>('get_engine_paths')
  } catch {
    // ignore
  }
  fetchUpnpStatus()
})

// Show in folder helper
async function showPathInFolder(path: string) {
  try {
    await invoke('show_in_folder', { path })
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// Check for updates
async function checkForUpdates() {
  try {
    const { check } = await import('@tauri-apps/plugin-updater')
    const update = await check()
    if (update) {
      ElMessage.info(`Update available: v${update.version}`)
    } else {
      ElMessage.success(t('about.upToDate'))
    }
    await appStore.saveConfig({ lastCheckUpdateTime: Date.now() })
  } catch (e) {
    ElMessage.error(String(e))
  }
}
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <h2 class="settings-title">{{ t('settings.title') }}</h2>
      <div class="settings-actions">
        <el-button size="small" @click="exportConfig">
          {{ t('settings.export') }}
        </el-button>
        <el-button size="small" @click="importConfig">
          {{ t('settings.import') }}
        </el-button>
        <el-button size="small" @click="resetDefaults">
          <el-icon><RefreshRight /></el-icon>
          {{ t('settings.resetDefaults') }}
        </el-button>
      </div>
    </div>

    <!-- Engine restart banner -->
    <div v-if="appStore.restartNeeded" class="restart-banner">
      <el-icon><Warning /></el-icon>
      <span>{{ t('settings.restartNeeded') }}</span>
      <el-button type="warning" size="small" @click="doRestartEngine">
        {{ t('settings.restartNow') }}
      </el-button>
    </div>

    <div class="settings-scroll">
      <el-form label-width="180px" label-position="left">
        <!-- Basic Settings -->
        <h3 class="settings-section">{{ t('settings.basic') }}</h3>

        <el-form-item :label="t('settings.theme')">
          <el-radio-group :model-value="appStore.config?.theme" @change="(val: string | number | boolean | undefined) => val && setTheme(val as 'auto' | 'light' | 'dark')">
            <el-radio-button value="auto">{{ t('settings.themeAuto') }}</el-radio-button>
            <el-radio-button value="light">{{ t('settings.themeLight') }}</el-radio-button>
            <el-radio-button value="dark">{{ t('settings.themeDark') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>

        <el-form-item :label="t('settings.language')">
          <el-select :model-value="appStore.config?.locale" @change="appStore.setLocale">
            <el-option label="English" value="en" />
            <el-option label="简体中文" value="zh-CN" />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('settings.downloadDir')">
          <div class="dir-input-group">
            <el-input :model-value="appStore.config?.downloadDir" readonly>
              <template #append>
                <el-button @click="selectDownloadDir">
                  <el-icon><FolderOpened /></el-icon>
                </el-button>
              </template>
            </el-input>
            <!-- Directory history popover -->
            <el-popover
              placement="bottom"
              :width="320"
              trigger="click"
              v-model:visible="showDirPopover"
            >
              <template #reference>
                <el-button size="small" class="dir-history-btn">
                  <el-icon><Clock /></el-icon>
                </el-button>
              </template>
              <div class="dir-popover">
                <div v-if="appStore.directoryFavorites.length > 0" class="dir-section">
                  <div class="dir-section-title">{{ t('settings.favoriteDirs') }}</div>
                  <div
                    v-for="dir in appStore.directoryFavorites"
                    :key="'fav-' + dir"
                    class="dir-item"
                    @click="selectDirFromHistory(dir)"
                  >
                    <el-icon><Star /></el-icon>
                    <span :title="dir">{{ dir }}</span>
                    <el-button
                      size="small" text circle
                      @click.stop="appStore.toggleDirectoryFavorite(dir)"
                    >
                      <el-icon><Close /></el-icon>
                    </el-button>
                  </div>
                </div>
                <div v-if="appStore.directoryHistory.length > 0" class="dir-section">
                  <div class="dir-section-title">{{ t('settings.recentDirs') }}</div>
                  <div
                    v-for="dir in appStore.directoryHistory"
                    :key="'recent-' + dir"
                    class="dir-item"
                    @click="selectDirFromHistory(dir)"
                  >
                    <span :title="dir">{{ dir }}</span>
                    <el-button
                      size="small" text circle
                      @click.stop="appStore.toggleDirectoryFavorite(dir)"
                    >
                      <el-icon><Star /></el-icon>
                    </el-button>
                  </div>
                </div>
                <div v-if="appStore.directoryHistory.length === 0 && appStore.directoryFavorites.length === 0" class="dir-empty">
                  {{ t('settings.noRecentDirs') }}
                </div>
              </div>
            </el-popover>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.autoStart')">
          <el-switch
            :model-value="appStore.config?.autoStart"
            @change="toggleAutoStart"
          />
        </el-form-item>

        <el-form-item :label="t('settings.startHidden')">
          <el-switch
            :model-value="appStore.config?.startHidden"
            @change="(val: string | number | boolean) => appStore.saveConfig({ startHidden: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.hideOnClose')">
          <el-switch
            :model-value="appStore.config?.hideOnClose"
            @change="(val: string | number | boolean) => appStore.saveConfig({ hideOnClose: Boolean(val) })"
          />
          <div class="form-tip">{{ t('settings.hideOnCloseTip') }}</div>
        </el-form-item>

        <el-form-item :label="t('settings.keepWindowState')">
          <el-switch
            :model-value="appStore.config?.keepWindowState"
            @change="(val: string | number | boolean) => appStore.saveConfig({ keepWindowState: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.notifyOnComplete')">
          <el-switch
            :model-value="appStore.config?.notifyOnComplete"
            @change="(val: string | number | boolean) => appStore.saveConfig({ notifyOnComplete: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.newTaskShowDownloading')">
          <el-switch
            :model-value="appStore.config?.newTaskShowDownloading"
            @change="(val: string | number | boolean) => appStore.saveConfig({ newTaskShowDownloading: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.noConfirmBeforeDeleteTask')">
          <el-switch
            :model-value="appStore.config?.noConfirmBeforeDeleteTask"
            @change="(val: string | number | boolean) => appStore.saveConfig({ noConfirmBeforeDeleteTask: Boolean(val) })"
          />
        </el-form-item>

        <!-- Download Settings -->
        <h3 class="settings-section">{{ t('settings.download') }}</h3>

        <el-form-item :label="t('settings.maxConcurrent')">
          <el-input-number
            :model-value="appStore.config?.maxConcurrentDownloads"
            :min="1"
            :max="20"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ maxConcurrentDownloads: val })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.maxConnections')">
          <el-input-number
            :model-value="appStore.config?.maxConnectionPerServer"
            :min="1"
            :max="64"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ maxConnectionPerServer: val })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.split')">
          <el-input-number
            :model-value="appStore.config?.split"
            :min="1"
            :max="64"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ split: val })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.minSplitSize')">
          <el-select
            :model-value="appStore.config?.minSplitSize"
            @change="(val: string) => appStore.saveConfig({ minSplitSize: val })"
          >
            <el-option
              v-for="opt in minSplitSizeOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
          <div class="form-tip">{{ t('settings.minSplitSizeTip') }}</div>
        </el-form-item>

        <el-form-item :label="t('settings.maxDownloadSpeed')">
          <el-select
            :model-value="appStore.config?.maxDownloadLimit"
            @change="(val: string) => appStore.saveConfig({ maxDownloadLimit: val })"
            style="width: 160px"
          >
            <el-option
              v-for="opt in speedOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('settings.maxUploadSpeed')">
          <el-select
            :model-value="appStore.config?.maxUploadLimit"
            @change="(val: string) => appStore.saveConfig({ maxUploadLimit: val })"
            style="width: 160px"
          >
            <el-option
              v-for="opt in speedOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('settings.maxOverallDownloadLimit')">
          <el-select
            :model-value="appStore.config?.maxOverallDownloadLimit"
            @change="(val: string) => appStore.saveConfig({ maxOverallDownloadLimit: val })"
            style="width: 160px"
          >
            <el-option
              v-for="opt in speedOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('settings.maxOverallUploadLimit')">
          <el-select
            :model-value="appStore.config?.maxOverallUploadLimit"
            @change="(val: string) => appStore.saveConfig({ maxOverallUploadLimit: val })"
            style="width: 160px"
          >
            <el-option
              v-for="opt in speedOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('settings.continueDownload')">
          <el-switch
            :model-value="appStore.config?.continueDownload"
            @change="(val: string | number | boolean) => appStore.saveConfig({ continueDownload: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.autoClearCompleted')">
          <el-switch
            :model-value="appStore.config?.autoClearCompleted"
            @change="(val: string | number | boolean) => appStore.saveConfig({ autoClearCompleted: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.resumeAllWhenAppLaunched')">
          <el-switch
            :model-value="appStore.config?.resumeAllWhenAppLaunched"
            @change="(val: string | number | boolean) => appStore.saveConfig({ resumeAllWhenAppLaunched: Boolean(val) })"
          />
        </el-form-item>

        <!-- BT Settings -->
        <h3 class="settings-section">{{ t('settings.bt') }}</h3>

        <el-form-item :label="t('settings.btPort')">
          <div class="port-input-group">
            <el-input-number
              :model-value="appStore.config?.btListenPort"
              :min="1024"
              :max="65535"
              @change="(val: number | undefined) => val != null && appStore.saveConfig({ btListenPort: val })"
            />
            <el-button size="small" circle @click="randomizeBtPort" :title="t('settings.randomPort')">
              <el-icon><MagicStick /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.dhtPort')">
          <div class="port-input-group">
            <el-input-number
              :model-value="appStore.config?.dhtListenPort"
              :min="1024"
              :max="65535"
              @change="(val: number | undefined) => val != null && appStore.saveConfig({ dhtListenPort: val })"
            />
            <el-button size="small" circle @click="randomizeDhtPort" :title="t('settings.randomPort')">
              <el-icon><MagicStick /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.upnp')">
          <el-switch
            :model-value="appStore.config?.enableUpnp"
            @change="(val: string | number | boolean) => appStore.saveConfig({ enableUpnp: Boolean(val) })"
          />
        </el-form-item>

        <template v-if="appStore.config?.enableUpnp && upnpStatus">
          <div class="upnp-status-panel">
            <div class="upnp-status-row">
              <span class="upnp-label">{{ t('settings.upnpStatus') }}</span>
              <el-tag
                :type="upnpStatus.gatewayFound ? 'success' : 'warning'"
                size="small"
                effect="plain"
              >
                {{ upnpStatus.gatewayFound ? t('settings.upnpConnected') : t('settings.upnpDisconnected') }}
              </el-tag>
            </div>
            <div class="upnp-status-row" v-if="upnpStatus.externalIp">
              <span class="upnp-label">{{ t('settings.externalIp') }}</span>
              <span class="upnp-value">{{ upnpStatus.externalIp }}</span>
            </div>
            <div class="upnp-status-row" v-if="upnpStatus.mappedPorts.length > 0">
              <span class="upnp-label">{{ t('settings.mappedPorts') }}</span>
              <div class="upnp-ports">
                <el-tag
                  v-for="mp in upnpStatus.mappedPorts"
                  :key="`${mp.port}-${mp.protocol}`"
                  size="small"
                  effect="plain"
                  type="info"
                >
                  {{ mp.port }}/{{ mp.protocol }}
                </el-tag>
              </div>
            </div>
            <el-button
              size="small"
              :loading="upnpLoading"
              @click="refreshUpnp"
            >
              {{ t('settings.refreshUpnp') }}
            </el-button>
          </div>
        </template>

        <el-form-item :label="t('settings.seedRatio')">
          <el-input-number
            :model-value="appStore.config?.seedRatio"
            :min="0"
            :max="10"
            :step="0.1"
            :precision="1"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ seedRatio: val })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.seedTime')">
          <el-input-number
            :model-value="appStore.config?.seedTime"
            :min="0"
            :max="99999"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ seedTime: val })"
          />
          <span class="input-suffix">{{ t('settings.seedTimeUnit') }}</span>
          <div class="form-tip">{{ t('settings.seedTimeTip') }}</div>
        </el-form-item>

        <el-form-item :label="t('settings.trackers')">
          <div class="tracker-sources">
            <div class="tracker-status" v-if="trackerCount > 0">
              <el-tag type="success" size="small" effect="plain">
                {{ t('settings.trackerCount', { count: trackerCount }) }}
              </el-tag>
            </div>
            <div class="tracker-list" v-if="appStore.config?.trackerSource?.length">
              <div
                v-for="url in appStore.config.trackerSource"
                :key="url"
                class="tracker-source-item"
              >
                <div class="tracker-source-info">
                  <span class="tracker-source-name">{{ getSourceName(url) }}</span>
                  <span class="tracker-source-url" :title="url">{{ url }}</span>
                </div>
                <el-button
                  size="small"
                  type="danger"
                  text
                  circle
                  @click="removeTrackerSource(url)"
                >
                  <el-icon><Close /></el-icon>
                </el-button>
              </div>
            </div>
            <div class="tracker-input">
              <el-input
                v-model="trackerInput"
                :placeholder="t('settings.trackerPlaceholder')"
                @keyup.enter="addTrackerSource"
                size="small"
              >
                <template #append>
                  <el-button @click="addTrackerSource">{{ t('settings.trackerAdd') }}</el-button>
                </template>
              </el-input>
            </div>
            <el-button size="small" @click="updateTrackers" :loading="trackerUpdating" class="update-btn">
              <el-icon v-if="!trackerUpdating"><Refresh /></el-icon>
              {{ t('settings.trackerUpdate') }}
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.btForceEncryption')">
          <el-switch
            :model-value="appStore.config?.btForceEncryption"
            @change="(val: string | number | boolean) => appStore.saveConfig({ btForceEncryption: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.btRequireCrypto')">
          <el-switch
            :model-value="appStore.config?.btRequireCrypto"
            @change="(val: string | number | boolean) => appStore.saveConfig({ btRequireCrypto: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.pauseMetadata')">
          <el-switch
            :model-value="appStore.config?.pauseMetadata"
            @change="(val: string | number | boolean) => appStore.saveConfig({ pauseMetadata: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.btSaveMetadata')">
          <el-switch
            :model-value="appStore.config?.btSaveMetadata"
            @change="(val: string | number | boolean) => appStore.saveConfig({ btSaveMetadata: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.btLoadSavedMetadata')">
          <el-switch
            :model-value="appStore.config?.btLoadSavedMetadata"
            @change="(val: string | number | boolean) => appStore.saveConfig({ btLoadSavedMetadata: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.btRemoveUnselectedFile')">
          <el-switch
            :model-value="appStore.config?.btRemoveUnselectedFile"
            @change="(val: string | number | boolean) => appStore.saveConfig({ btRemoveUnselectedFile: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.btDetachSeedOnly')">
          <el-switch
            :model-value="appStore.config?.btDetachSeedOnly"
            @change="(val: string | number | boolean) => appStore.saveConfig({ btDetachSeedOnly: Boolean(val) })"
          />
        </el-form-item>

        <!-- Proxy Settings -->
        <h3 class="settings-section">{{ t('settings.proxy') }}</h3>

        <el-form-item :label="t('settings.proxyEnable')">
          <el-switch
            :model-value="appStore.config?.proxyEnabled"
            @change="(val: string | number | boolean) => appStore.saveConfig({ proxyEnabled: Boolean(val) })"
          />
        </el-form-item>

        <template v-if="appStore.config?.proxyEnabled">
          <el-form-item :label="t('settings.proxyType')">
            <el-select
              :model-value="appStore.config?.proxyType"
              @change="(val: string) => appStore.saveConfig({ proxyType: val as 'http' | 'https' | 'socks5' })"
            >
              <el-option label="HTTP" value="http" />
              <el-option label="HTTPS" value="https" />
              <el-option label="SOCKS5" value="socks5" />
            </el-select>
          </el-form-item>

          <el-form-item :label="t('settings.proxyHost')">
            <el-input
              :model-value="appStore.config?.proxyHost"
              placeholder="127.0.0.1"
              @change="(val: string) => appStore.saveConfig({ proxyHost: val })"
            />
          </el-form-item>

          <el-form-item :label="t('settings.proxyPort')">
            <el-input-number
              :model-value="appStore.config?.proxyPort"
              :min="1"
              :max="65535"
              @change="(val: number | undefined) => val != null && appStore.saveConfig({ proxyPort: val })"
            />
          </el-form-item>

          <el-form-item :label="t('settings.proxyUsername')">
            <el-input
              :model-value="appStore.config?.proxyUsername"
              :placeholder="t('settings.optional')"
              @change="(val: string) => appStore.saveConfig({ proxyUsername: val })"
            />
          </el-form-item>

          <el-form-item :label="t('settings.proxyPassword')">
            <el-input
              :model-value="appStore.config?.proxyPassword"
              type="password"
              :placeholder="t('settings.optional')"
              show-password
              @change="(val: string) => appStore.saveConfig({ proxyPassword: val })"
            />
          </el-form-item>

          <el-form-item :label="t('settings.noProxy')">
            <el-input
              :model-value="appStore.config?.noProxy"
              :placeholder="t('settings.noProxyPlaceholder')"
              @change="(val: string) => appStore.saveConfig({ noProxy: val })"
            />
          </el-form-item>
        </template>

        <!-- Advanced Settings -->
        <h3 class="settings-section">{{ t('settings.advanced') }}</h3>

        <el-form-item :label="t('settings.userAgent')">
          <div class="user-agent-group">
            <el-input
              :model-value="appStore.config?.userAgent"
              placeholder="Motrix/2.0.0"
              @change="(val: string) => appStore.saveConfig({ userAgent: val })"
            />
            <div class="preset-buttons">
              <el-button
                v-for="preset in userAgentPresets"
                :key="preset.label"
                size="small"
                @click="appStore.saveConfig({ userAgent: preset.value })"
              >
                {{ preset.label }}
              </el-button>
            </div>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.allowOverwrite')">
          <el-switch
            :model-value="appStore.config?.allowOverwrite"
            @change="(val: string | number | boolean) => appStore.saveConfig({ allowOverwrite: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.autoFileRenaming')">
          <el-switch
            :model-value="appStore.config?.autoFileRenaming"
            @change="(val: string | number | boolean) => appStore.saveConfig({ autoFileRenaming: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.followMetalink')">
          <el-select
            :model-value="appStore.config?.followMetalink"
            @change="(val: string) => appStore.saveConfig({ followMetalink: val })"
          >
            <el-option label="true" value="true" />
            <el-option label="false" value="false" />
            <el-option label="mem" value="mem" />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('settings.rpcPort')">
          <div class="port-input-group">
            <el-input-number
              :model-value="appStore.config?.rpcPort"
              :min="1024"
              :max="65535"
              @change="(val: number | undefined) => val != null && appStore.saveConfig({ rpcPort: val })"
            />
            <el-button size="small" circle @click="randomizeRpcPort" :title="t('settings.randomPort')">
              <el-icon><MagicStick /></el-icon>
            </el-button>
          </div>
          <div class="form-tip">{{ t('settings.rpcPortTip') }}</div>
        </el-form-item>

        <el-form-item label="RPC Secret">
          <div class="rpc-secret-group">
            <el-input
              :model-value="appStore.config?.rpcSecret"
              type="password"
              show-password
              @change="(val: string) => appStore.saveConfig({ rpcSecret: val })"
            />
            <el-button size="small" circle @click="randomizeRpcSecret" :title="t('settings.randomSecret')">
              <el-icon><MagicStick /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.protocols')">
          <div class="protocol-switches">
            <el-switch
              :model-value="appStore.config?.defaultMagnetClient"
              active-text="magnet://"
              @change="(val: string | number | boolean) => appStore.saveConfig({ defaultMagnetClient: Boolean(val) })"
            />
            <el-switch
              :model-value="appStore.config?.defaultThunderClient"
              active-text="thunder://"
              @change="(val: string | number | boolean) => appStore.saveConfig({ defaultThunderClient: Boolean(val) })"
            />
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.autoCheckUpdate')">
          <div class="update-check-group">
            <el-switch
              :model-value="appStore.config?.autoCheckUpdate"
              @change="(val: string | number | boolean) => appStore.saveConfig({ autoCheckUpdate: Boolean(val) })"
            />
            <el-button size="small" text type="primary" @click="checkForUpdates">
              {{ t('settings.checkNow') }}
            </el-button>
            <span v-if="lastCheckUpdateText" class="last-check-time">
              {{ t('settings.lastCheckUpdate', { time: lastCheckUpdateText }) }}
            </span>
          </div>
        </el-form-item>

        <!-- Developer Section -->
        <h3 class="settings-section">{{ t('settings.developer') }}</h3>

        <div class="aria2-status-panel">
          <div class="aria2-status-panel__header">
            <div class="aria2-status-panel__heading">
              <div class="aria2-status-panel__title-row">
                <span class="aria2-status-panel__title">{{ t('settings.aria2Status') }}</span>
                <el-tag :type="aria2StatusTagType" effect="dark" size="small">
                  {{ aria2StatusLabel }}
                </el-tag>
              </div>
              <div class="aria2-status-panel__subtitle">
                {{ t('settings.rpcPort') }}: {{ appStore.config?.rpcPort ?? '—' }}
              </div>
            </div>
            <el-button
              :type="appStore.restartNeeded ? 'warning' : 'primary'"
              size="small"
              :loading="aria2Diagnostics.isRestarting"
              @click="doRestartEngine"
            >
              {{ aria2ActionLabel }}
            </el-button>
          </div>

          <div class="aria2-status-grid">
            <div class="aria2-status-card">
              <span class="aria2-status-card__label">{{ t('settings.aria2CurrentState') }}</span>
              <span class="aria2-status-card__value">{{ aria2StatusLabel }}</span>
            </div>
            <div class="aria2-status-card">
              <span class="aria2-status-card__label">{{ t('settings.aria2LastSuccess') }}</span>
              <span class="aria2-status-card__value">{{ aria2LastSuccessText }}</span>
            </div>
            <div class="aria2-status-card aria2-status-card--wide" :class="{ 'has-error': aria2Diagnostics.lastError }">
              <span class="aria2-status-card__label">{{ t('settings.aria2LastError') }}</span>
              <span class="aria2-status-card__value aria2-status-card__value--multiline">
                {{ aria2LastErrorText }}
              </span>
              <span v-if="aria2LastErrorTimeText" class="aria2-status-card__meta">
                {{ aria2LastErrorTimeText }}
              </span>
            </div>
          </div>
        </div>

        <el-form-item v-if="enginePaths.aria2Config" :label="t('settings.aria2ConfigPath')">
          <div class="path-field">
            <el-input :model-value="enginePaths.aria2Config" readonly size="small" />
            <el-button size="small" text @click="showPathInFolder(enginePaths.aria2Config)">
              <el-icon><Folder /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item v-if="enginePaths.aria2Session" :label="t('settings.aria2SessionPath')">
          <div class="path-field">
            <el-input :model-value="enginePaths.aria2Session" readonly size="small" />
            <el-button size="small" text @click="showPathInFolder(enginePaths.aria2Session)">
              <el-icon><Folder /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item v-if="enginePaths.appLogDir" :label="t('settings.appLogPath')">
          <div class="path-field">
            <el-input :model-value="enginePaths.appLogDir" readonly size="small" />
            <el-button size="small" text @click="showPathInFolder(enginePaths.appLogDir)">
              <el-icon><Folder /></el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item :label="t('settings.logLevel')">
          <el-select
            v-model="logLevel"
            size="small"
            style="width: 120px"
          >
            <el-option
              v-for="opt in logLevelOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>

        <div class="danger-zone">
          <el-button type="warning" @click="resetSession">
            <el-icon><RefreshRight /></el-icon>
            {{ t('settings.sessionReset') }}
          </el-button>
          <el-button type="danger" @click="factoryReset">
            <el-icon><Delete /></el-icon>
            {{ t('settings.factoryReset') }}
          </el-button>
        </div>
      </el-form>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.settings-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.settings-actions {
  display: flex;
  gap: 8px;
}

.settings-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0;
}

.settings-scroll {
  flex: 1;
  overflow-y: auto;
  max-width: 640px;
  padding-right: 12px;
}

.settings-section {
  font-size: 16px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin: 24px 0 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--el-border-color-lighter);

  &:first-of-type {
    margin-top: 0;
  }
}

.restart-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  margin-bottom: 12px;
  border-radius: 6px;
  background: var(--el-color-warning-light-9);
  border: 1px solid var(--el-color-warning-light-7);
  color: var(--el-color-warning-dark-2);
  font-size: 13px;
  flex-shrink: 0;
}

.aria2-status-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  margin-bottom: 16px;
  border-radius: 12px;
  background:
    linear-gradient(135deg, var(--el-fill-color-light), color-mix(in srgb, var(--el-color-primary-light-9) 45%, transparent));
  border: 1px solid var(--el-border-color-light);
}

.aria2-status-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.aria2-status-panel__heading {
  min-width: 0;
}

.aria2-status-panel__title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.aria2-status-panel__title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.aria2-status-panel__subtitle {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.aria2-status-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.aria2-status-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid var(--el-border-color-lighter);
}

.is-dark .aria2-status-card {
  background: rgba(18, 24, 33, 0.72);
}

.aria2-status-card--wide {
  grid-column: 1 / -1;
}

.aria2-status-card.has-error {
  border-color: var(--el-color-danger-light-5);
}

.aria2-status-card__label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--el-text-color-secondary);
}

.aria2-status-card__value {
  font-size: 13px;
  line-height: 1.5;
  color: var(--el-text-color-primary);
}

.aria2-status-card__value--multiline {
  word-break: break-word;
}

.aria2-status-card__meta {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.tracker-sources {
  width: 100%;
}

.tracker-status {
  margin-bottom: 8px;
}

.tracker-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 8px;
}

.tracker-source-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 6px;
  background: var(--el-fill-color-light);
  transition: background 0.15s;

  &:hover {
    background: var(--el-fill-color);
  }
}

.tracker-source-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tracker-source-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.tracker-source-url {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tracker-input {
  margin-bottom: 8px;
}

.update-btn {
  margin-top: 4px;
}

.form-tip {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
  line-height: 1.5;
}

.input-suffix {
  margin-left: 8px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

// Directory input group
.dir-input-group {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;

  .el-input {
    flex: 1;
  }
}

.dir-history-btn {
  flex-shrink: 0;
}

.dir-popover {
  max-height: 240px;
  overflow-y: auto;
}

.dir-section {
  margin-bottom: 12px;
}

.dir-section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
  text-transform: uppercase;
}

.dir-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover {
    background: var(--el-fill-color);
  }

  span {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

.dir-empty {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  text-align: center;
  padding: 12px;
}

// Port input group with random button
.port-input-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

// UPnP status panel
.upnp-status-panel {
  margin: 0 0 18px 0;
  padding: 12px 16px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background: var(--el-fill-color-lighter, #f5f7fa);
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;

  .upnp-status-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .upnp-label {
    color: var(--el-text-color-secondary);
    font-size: 13px;
    min-width: 80px;
  }

  .upnp-value {
    font-family: monospace;
    font-size: 13px;
  }

  .upnp-ports {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
}

// RPC secret group
.rpc-secret-group {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;

  .el-input {
    flex: 1;
  }
}

// User-Agent presets
.user-agent-group {
  width: 100%;
}

.preset-buttons {
  display: flex;
  gap: 4px;
  margin-top: 6px;
  flex-wrap: wrap;
}

// Protocol switches
.protocol-switches {
  display: flex;
  gap: 16px;
}

@media (max-width: 640px) {
  .aria2-status-panel__header {
    flex-direction: column;
  }

  .aria2-status-grid {
    grid-template-columns: 1fr;
  }
}

// Update check group
.update-check-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.last-check-time {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

// Path field
.path-field {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;

  .el-input {
    flex: 1;
  }
}

// Danger zone
.danger-zone {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
  display: flex;
  gap: 12px;
}
</style>
