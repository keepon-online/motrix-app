<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'

const { t } = useI18n()
const appStore = useAppStore()
const { setTheme } = useTheme()

const trackerInput = ref('')
const trackerUpdating = ref(false)

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

async function selectDownloadDir() {
  const selected = await open({
    directory: true,
    multiple: false,
  })

  if (selected) {
    await appStore.setDownloadDir(selected as string)
  }
}

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
    ElMessage.success(t('settings.trackerCount', { count: trackers.length }))
  } catch {
    ElMessage.error(t('settings.trackerUpdateFailed'))
  } finally {
    trackerUpdating.value = false
  }
}

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
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <h2 class="settings-title">{{ t('settings.title') }}</h2>
      <el-button size="small" @click="resetDefaults">
        <el-icon><RefreshRight /></el-icon>
        {{ t('settings.resetDefaults') }}
      </el-button>
    </div>

    <div class="settings-scroll">
      <el-form label-width="180px" label-position="left">
        <!-- Basic Settings -->
        <h3 class="settings-section">{{ t('settings.basic') }}</h3>

        <el-form-item :label="t('settings.theme')">
          <el-radio-group :model-value="appStore.config?.theme" @change="(val: any) => setTheme(val as 'auto' | 'light' | 'dark')">
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
          <el-input :model-value="appStore.config?.downloadDir" readonly>
            <template #append>
              <el-button @click="selectDownloadDir">
                <el-icon><FolderOpened /></el-icon>
              </el-button>
            </template>
          </el-input>
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
          >
            <el-option
              v-for="opt in speedOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>

        <!-- BT Settings -->
        <h3 class="settings-section">{{ t('settings.bt') }}</h3>

        <el-form-item :label="t('settings.btPort')">
          <el-input-number
            :model-value="appStore.config?.btListenPort"
            :min="1024"
            :max="65535"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ btListenPort: val })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.dhtPort')">
          <el-input-number
            :model-value="appStore.config?.dhtListenPort"
            :min="1024"
            :max="65535"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ dhtListenPort: val })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.upnp')">
          <el-switch
            :model-value="appStore.config?.enableUpnp"
            @change="(val: string | number | boolean) => appStore.saveConfig({ enableUpnp: Boolean(val) })"
          />
        </el-form-item>

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
            <div class="tracker-input">
              <el-input
                v-model="trackerInput"
                :placeholder="t('settings.trackerPlaceholder')"
                @keyup.enter="addTrackerSource"
              >
                <template #append>
                  <el-button @click="addTrackerSource">{{ t('settings.trackerAdd') }}</el-button>
                </template>
              </el-input>
            </div>
            <div class="tracker-list" v-if="appStore.config?.trackerSource?.length">
              <el-tag
                v-for="url in appStore.config.trackerSource"
                :key="url"
                closable
                @close="removeTrackerSource(url)"
                class="tracker-tag"
              >
                {{ url }}
              </el-tag>
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
          <el-input
            :model-value="appStore.config?.userAgent"
            placeholder="Motrix/2.0.0"
            @change="(val: string) => appStore.saveConfig({ userAgent: val })"
          />
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

        <el-form-item :label="t('settings.continueDownload')">
          <el-switch
            :model-value="appStore.config?.continueDownload"
            @change="(val: string | number | boolean) => appStore.saveConfig({ continueDownload: Boolean(val) })"
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
          <el-input-number
            :model-value="appStore.config?.rpcPort"
            :min="1024"
            :max="65535"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ rpcPort: val })"
          />
          <div class="form-tip">{{ t('settings.rpcPortTip') }}</div>
        </el-form-item>

        <el-form-item :label="t('settings.hideOnClose')">
          <el-switch
            :model-value="appStore.config?.hideOnClose"
            @change="(val: string | number | boolean) => appStore.saveConfig({ hideOnClose: Boolean(val) })"
          />
          <div class="form-tip">{{ t('settings.hideOnCloseTip') }}</div>
        </el-form-item>

        <el-form-item :label="t('settings.autoStart')">
          <el-switch
            :model-value="appStore.config?.autoStart"
            @change="(val: string | number | boolean) => appStore.saveConfig({ autoStart: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.startHidden')">
          <el-switch
            :model-value="appStore.config?.startHidden"
            @change="(val: string | number | boolean) => appStore.saveConfig({ startHidden: Boolean(val) })"
          />
        </el-form-item>

        <el-form-item :label="t('settings.notifyOnComplete')">
          <el-switch
            :model-value="appStore.config?.notifyOnComplete"
            @change="(val: string | number | boolean) => appStore.saveConfig({ notifyOnComplete: Boolean(val) })"
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

.tracker-sources {
  width: 100%;
}

.tracker-input {
  margin-bottom: 8px;
}

.tracker-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.tracker-tag {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
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
</style>
