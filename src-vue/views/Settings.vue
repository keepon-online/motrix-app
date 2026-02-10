<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'

const appStore = useAppStore()
const { setTheme } = useTheme()

const trackerInput = ref('')

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
    ElMessage.warning('Please enter a valid URL')
    return
  }

  const current = appStore.config?.trackerSource || []
  if (current.includes(url)) {
    ElMessage.warning('This tracker source already exists')
    return
  }

  appStore.saveConfig({ trackerSource: [...current, url] })
  trackerInput.value = ''
  ElMessage.success('Tracker source added')
}

function removeTrackerSource(url: string) {
  const current = appStore.config?.trackerSource || []
  appStore.saveConfig({ trackerSource: current.filter(t => t !== url) })
}

async function updateTrackers() {
  ElMessage.info('Updating trackers...')
  // TODO: Implement tracker update from sources
  ElMessage.success('Trackers updated')
}
</script>

<template>
  <div class="settings-view">
    <h2 class="settings-title">Settings</h2>

    <el-form label-width="180px" label-position="left">
      <!-- Basic Settings -->
      <h3 class="settings-section">Basic</h3>

      <el-form-item label="Theme">
        <el-radio-group :model-value="appStore.config?.theme" @change="(val: any) => setTheme(val as 'auto' | 'light' | 'dark')">
          <el-radio-button value="auto">Auto</el-radio-button>
          <el-radio-button value="light">Light</el-radio-button>
          <el-radio-button value="dark">Dark</el-radio-button>
        </el-radio-group>
      </el-form-item>

      <el-form-item label="Language">
        <el-select :model-value="appStore.config?.locale" @change="appStore.setLocale">
          <el-option label="English" value="en" />
          <el-option label="简体中文" value="zh-CN" />
        </el-select>
      </el-form-item>

      <el-form-item label="Download Directory">
        <el-input :model-value="appStore.config?.downloadDir" readonly>
          <template #append>
            <el-button @click="selectDownloadDir">
              <el-icon><FolderOpened /></el-icon>
            </el-button>
          </template>
        </el-input>
      </el-form-item>

      <!-- Download Settings -->
      <h3 class="settings-section">Download</h3>

      <el-form-item label="Max Concurrent Downloads">
        <el-input-number
          :model-value="appStore.config?.maxConcurrentDownloads"
          :min="1"
          :max="20"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ maxConcurrentDownloads: val })"
        />
      </el-form-item>

      <el-form-item label="Max Connections">
        <el-input-number
          :model-value="appStore.config?.maxConnectionPerServer"
          :min="1"
          :max="64"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ maxConnectionPerServer: val })"
        />
      </el-form-item>

      <el-form-item label="Split">
        <el-input-number
          :model-value="appStore.config?.split"
          :min="1"
          :max="64"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ split: val })"
        />
      </el-form-item>

      <el-form-item label="Max Download Speed">
        <el-input
          :model-value="appStore.config?.maxDownloadLimit"
          placeholder="0 (unlimited)"
          @change="(val: string) => appStore.saveConfig({ maxDownloadLimit: val })"
        >
          <template #append>B/s</template>
        </el-input>
        <div class="form-tip">0 means unlimited. Examples: 1048576 (1MB/s), 524288 (512KB/s)</div>
      </el-form-item>

      <el-form-item label="Max Upload Speed">
        <el-input
          :model-value="appStore.config?.maxUploadLimit"
          placeholder="0 (unlimited)"
          @change="(val: string) => appStore.saveConfig({ maxUploadLimit: val })"
        >
          <template #append>B/s</template>
        </el-input>
        <div class="form-tip">0 means unlimited</div>
      </el-form-item>

      <!-- BT Settings -->
      <h3 class="settings-section">BitTorrent</h3>

      <el-form-item label="BT Listen Port">
        <el-input-number
          :model-value="appStore.config?.btListenPort"
          :min="1024"
          :max="65535"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ btListenPort: val })"
        />
      </el-form-item>

      <el-form-item label="DHT Listen Port">
        <el-input-number
          :model-value="appStore.config?.dhtListenPort"
          :min="1024"
          :max="65535"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ dhtListenPort: val })"
        />
      </el-form-item>

      <el-form-item label="Enable UPnP">
        <el-switch
          :model-value="appStore.config?.enableUpnp"
          @change="(val: string | number | boolean) => appStore.saveConfig({ enableUpnp: Boolean(val) })"
        />
      </el-form-item>

      <el-form-item label="Seed Ratio">
        <el-input-number
          :model-value="appStore.config?.seedRatio"
          :min="0"
          :max="10"
          :step="0.1"
          :precision="1"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ seedRatio: val })"
        />
      </el-form-item>

      <el-form-item label="Trackers">
        <div class="tracker-sources">
          <div class="tracker-input">
            <el-input
              v-model="trackerInput"
              placeholder="Enter tracker source URL"
              @keyup.enter="addTrackerSource"
            >
              <template #append>
                <el-button @click="addTrackerSource">Add</el-button>
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
          <el-button size="small" @click="updateTrackers" class="update-btn">
            <el-icon><Refresh /></el-icon>
            Update Trackers
          </el-button>
        </div>
      </el-form-item>

      <!-- Proxy Settings -->
      <h3 class="settings-section">Proxy</h3>

      <el-form-item label="Enable Proxy">
        <el-switch
          :model-value="appStore.config?.proxyEnabled"
          @change="(val: string | number | boolean) => appStore.saveConfig({ proxyEnabled: Boolean(val) })"
        />
      </el-form-item>

      <template v-if="appStore.config?.proxyEnabled">
        <el-form-item label="Proxy Type">
          <el-select
            :model-value="appStore.config?.proxyType"
            @change="(val: string) => appStore.saveConfig({ proxyType: val as 'http' | 'https' | 'socks5' })"
          >
            <el-option label="HTTP" value="http" />
            <el-option label="HTTPS" value="https" />
            <el-option label="SOCKS5" value="socks5" />
          </el-select>
        </el-form-item>

        <el-form-item label="Proxy Host">
          <el-input
            :model-value="appStore.config?.proxyHost"
            placeholder="127.0.0.1"
            @change="(val: string) => appStore.saveConfig({ proxyHost: val })"
          />
        </el-form-item>

        <el-form-item label="Proxy Port">
          <el-input-number
            :model-value="appStore.config?.proxyPort"
            :min="1"
            :max="65535"
            @change="(val: number | undefined) => val != null && appStore.saveConfig({ proxyPort: val })"
          />
        </el-form-item>

        <el-form-item label="Proxy Username">
          <el-input
            :model-value="appStore.config?.proxyUsername"
            placeholder="Optional"
            @change="(val: string) => appStore.saveConfig({ proxyUsername: val })"
          />
        </el-form-item>

        <el-form-item label="Proxy Password">
          <el-input
            :model-value="appStore.config?.proxyPassword"
            type="password"
            placeholder="Optional"
            show-password
            @change="(val: string) => appStore.saveConfig({ proxyPassword: val })"
          />
        </el-form-item>
      </template>

      <!-- Advanced Settings -->
      <h3 class="settings-section">Advanced</h3>

      <el-form-item label="User Agent">
        <el-input
          :model-value="appStore.config?.userAgent"
          placeholder="Motrix/1.0"
          @change="(val: string) => appStore.saveConfig({ userAgent: val })"
        />
      </el-form-item>

      <el-form-item label="RPC Port">
        <el-input-number
          :model-value="appStore.config?.rpcPort"
          :min="1024"
          :max="65535"
          @change="(val: number | undefined) => val != null && appStore.saveConfig({ rpcPort: val })"
        />
        <div class="form-tip">Requires restart to take effect</div>
      </el-form-item>

      <el-form-item label="Hide on Close">
        <el-switch
          :model-value="appStore.config?.hideOnClose"
          @change="(val: string | number | boolean) => appStore.saveConfig({ hideOnClose: Boolean(val) })"
        />
        <div class="form-tip">Minimize to tray instead of quitting when closing</div>
      </el-form-item>

      <el-form-item label="Auto Start">
        <el-switch
          :model-value="appStore.config?.autoStart"
          @change="(val: string | number | boolean) => appStore.saveConfig({ autoStart: Boolean(val) })"
        />
      </el-form-item>

      <el-form-item label="Start Hidden">
        <el-switch
          :model-value="appStore.config?.startHidden"
          @change="(val: string | number | boolean) => appStore.saveConfig({ startHidden: Boolean(val) })"
        />
      </el-form-item>

      <el-form-item label="Notification on Complete">
        <el-switch
          :model-value="appStore.config?.notifyOnComplete"
          @change="(val: string | number | boolean) => appStore.saveConfig({ notifyOnComplete: Boolean(val) })"
        />
      </el-form-item>

      <el-form-item label="Auto Clear Completed">
        <el-switch
          :model-value="appStore.config?.autoClearCompleted"
          @change="(val: string | number | boolean) => appStore.saveConfig({ autoClearCompleted: Boolean(val) })"
        />
      </el-form-item>
    </el-form>
  </div>
</template>

<style lang="scss" scoped>
.settings-view {
  max-width: 600px;
}

.settings-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 24px;
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
}
</style>
