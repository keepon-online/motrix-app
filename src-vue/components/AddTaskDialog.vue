<script setup lang="ts">
import { ref, computed, watch, inject, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaskStore } from '@/stores/task'
import { useAppStore } from '@/stores/app'
import { open } from '@tauri-apps/plugin-dialog'
import { readText } from '@tauri-apps/plugin-clipboard-manager'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

interface TorrentFileInfo {
  index: number
  path: string
  length: number
}

interface TorrentInfo {
  name: string
  comment: string
  files: TorrentFileInfo[]
}

const { t } = useI18n()
const visible = defineModel<boolean>({ default: false })

const taskStore = useTaskStore()
const appStore = useAppStore()

// Receive pending URLs from App.vue (CLI args, deep links, second instance)
const pendingUrls = inject<Ref<string[]>>('pendingUrls', ref([]))

const activeTab = ref<'uri' | 'torrent'>('uri')
const uriInput = ref('')
const torrentFile = ref<string | null>(null)
const torrentFileName = ref('')
const torrentInfo = ref<TorrentInfo | null>(null)
const selectedFileIndices = ref<number[]>([])
const showAdvanced = ref(false)

// Options
const downloadDir = ref(appStore.downloadDir)
const split = ref(16)
const maxConnectionPerServer = ref(16)
const fileName = ref('')

// Advanced options
const userAgent = ref('')
const referer = ref('')
const cookie = ref('')
const authorization = ref('')

const canSubmit = computed(() => {
  if (activeTab.value === 'uri') {
    return uriInput.value.trim().length > 0
  }
  return torrentFile.value !== null
})

const allFilesSelected = computed(() => {
  if (!torrentInfo.value) return false
  return selectedFileIndices.value.length === torrentInfo.value.files.length
})

// Auto-detect clipboard content or consume pending URLs when dialog opens
watch(visible, async (val) => {
  if (val) {
    downloadDir.value = appStore.downloadDir
    // Consume pending URLs from CLI args / deep links / second instance
    if (pendingUrls.value.length > 0) {
      uriInput.value = pendingUrls.value.join('\n')
      pendingUrls.value = []
      return
    }
    try {
      const clipText = await readText()
      if (clipText && isUrl(clipText.trim()) && !uriInput.value) {
        uriInput.value = clipText.trim()
      }
    } catch {
      // Clipboard access may fail, ignore
    }
  }
})

function isUrl(text: string): boolean {
  return /^(https?|ftp|magnet|thunder):\/?\/?\S/i.test(text)
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

async function selectTorrent() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Torrent', extensions: ['torrent'] }],
  })

  if (selected) {
    const filePath = selected as string
    torrentFileName.value = filePath.split('/').pop()?.split('\\').pop() || 'torrent'

    // Read file for base64 (needed by aria2 addTorrent)
    const { readFile } = await import('@tauri-apps/plugin-fs')
    const contents = await readFile(filePath)
    torrentFile.value = btoa(String.fromCharCode(...contents))

    // Parse torrent to get file list
    try {
      const info = await invoke<TorrentInfo>('parse_torrent_file', { filePath })
      torrentInfo.value = info
      // Select all files by default
      selectedFileIndices.value = info.files.map(f => f.index)
    } catch (e) {
      console.warn('Failed to parse torrent:', e)
      torrentInfo.value = null
      selectedFileIndices.value = []
    }
  }
}

function toggleAllFiles() {
  if (!torrentInfo.value) return
  if (allFilesSelected.value) {
    selectedFileIndices.value = []
  } else {
    selectedFileIndices.value = torrentInfo.value.files.map(f => f.index)
  }
}

function toggleFile(index: number) {
  const idx = selectedFileIndices.value.indexOf(index)
  if (idx >= 0) {
    selectedFileIndices.value.splice(idx, 1)
  } else {
    selectedFileIndices.value.push(index)
  }
}

async function selectDirectory() {
  const selected = await open({
    directory: true,
    multiple: false,
  })

  if (selected) {
    downloadDir.value = selected as string
  }
}

async function submit() {
  const options: Record<string, unknown> = {
    split: String(split.value),
    'max-connection-per-server': String(maxConnectionPerServer.value),
  }

  // Only set dir if non-empty
  if (downloadDir.value) {
    options.dir = downloadDir.value
  }

  // Advanced options
  if (fileName.value.trim()) options.out = fileName.value.trim()
  if (userAgent.value.trim()) options['user-agent'] = userAgent.value.trim()
  if (referer.value.trim()) options.referer = referer.value.trim()
  if (cookie.value.trim()) options.header = [`Cookie: ${cookie.value.trim()}`]
  if (authorization.value.trim()) {
    const headers = options.header as string[] || []
    headers.push(`Authorization: ${authorization.value.trim()}`)
    options.header = headers
  }

  try {
    if (activeTab.value === 'uri') {
      const uris = uriInput.value
        .split('\n')
        .map((u) => u.trim())
        .filter((u) => u.length > 0)

      if (uris.length === 0) return

      for (const uri of uris) {
        await taskStore.addUri([uri], options)
      }
    } else if (torrentFile.value) {
      // Add select-file option if user has deselected some files
      if (torrentInfo.value && selectedFileIndices.value.length > 0
          && selectedFileIndices.value.length < torrentInfo.value.files.length) {
        // aria2 select-file uses 1-based indices
        const indices = selectedFileIndices.value
          .map(i => i + 1)
          .sort((a, b) => a - b)
          .join(',')
        options['select-file'] = indices
      }
      await taskStore.addTorrent(torrentFile.value, options)
    }

    resetForm()
    visible.value = false
  } catch (error) {
    console.error('Failed to add task:', error)
    ElMessage.error(String(error))
  }
}

function resetForm() {
  uriInput.value = ''
  torrentFile.value = null
  torrentFileName.value = ''
  torrentInfo.value = null
  selectedFileIndices.value = []
  fileName.value = ''
  userAgent.value = ''
  referer.value = ''
  cookie.value = ''
  authorization.value = ''
  showAdvanced.value = false
}

function handleClose() {
  resetForm()
}
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="t('dialog.addTask')"
    width="560px"
    :close-on-click-modal="false"
    @closed="handleClose"
  >
    <el-tabs v-model="activeTab">
      <el-tab-pane :label="t('dialog.url')" name="uri">
        <el-input
          v-model="uriInput"
          type="textarea"
          :rows="5"
          :placeholder="t('dialog.urlPlaceholder')"
        />
      </el-tab-pane>

      <el-tab-pane :label="t('dialog.torrent')" name="torrent">
        <div class="torrent-upload">
          <el-button @click="selectTorrent">
            <el-icon><Upload /></el-icon>
            {{ t('dialog.selectTorrent') }}
          </el-button>
          <span v-if="torrentFileName" class="torrent-name">{{ torrentFileName }}</span>
        </div>

        <!-- Torrent file list -->
        <div v-if="torrentInfo && torrentInfo.files.length > 1" class="torrent-files">
          <div class="torrent-files-header">
            <el-checkbox
              :model-value="allFilesSelected"
              :indeterminate="selectedFileIndices.length > 0 && !allFilesSelected"
              @change="toggleAllFiles"
            >
              {{ t('task.selectAll') }}
              ({{ selectedFileIndices.length }}/{{ torrentInfo.files.length }})
            </el-checkbox>
          </div>
          <div class="torrent-files-list">
            <div
              v-for="file in torrentInfo.files"
              :key="file.index"
              class="torrent-file-item"
            >
              <el-checkbox
                :model-value="selectedFileIndices.includes(file.index)"
                @change="toggleFile(file.index)"
              >
                <span class="file-path">{{ file.path }}</span>
                <span class="file-size">{{ formatFileSize(file.length) }}</span>
              </el-checkbox>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <el-divider />

    <el-form label-width="140px" label-position="left">
      <el-form-item :label="t('dialog.downloadDir')">
        <el-input v-model="downloadDir" readonly>
          <template #append>
            <el-button @click="selectDirectory">
              <el-icon><FolderOpened /></el-icon>
            </el-button>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('dialog.fileName')" v-if="activeTab === 'uri'">
        <el-input v-model="fileName" :placeholder="t('dialog.fileNamePlaceholder')" />
      </el-form-item>

      <el-form-item :label="t('dialog.split')">
        <el-slider v-model="split" :min="1" :max="64" :step="1" show-input />
      </el-form-item>

      <el-form-item :label="t('dialog.connections')">
        <el-slider v-model="maxConnectionPerServer" :min="1" :max="64" :step="1" show-input />
      </el-form-item>

      <!-- Advanced Options Toggle -->
      <div class="advanced-toggle" @click="showAdvanced = !showAdvanced">
        <el-icon>
          <ArrowRight v-if="!showAdvanced" />
          <ArrowDown v-else />
        </el-icon>
        <span>{{ t('dialog.advancedOptions') }}</span>
      </div>

      <template v-if="showAdvanced">
        <el-form-item label="User-Agent">
          <el-input v-model="userAgent" :placeholder="t('dialog.userAgentPlaceholder')" />
        </el-form-item>

        <el-form-item :label="t('dialog.referer')">
          <el-input v-model="referer" :placeholder="t('dialog.refererPlaceholder')" />
        </el-form-item>

        <el-form-item :label="t('dialog.cookie')">
          <el-input v-model="cookie" :placeholder="t('dialog.cookiePlaceholder')" />
        </el-form-item>

        <el-form-item :label="t('dialog.authorization')">
          <el-input v-model="authorization" :placeholder="t('dialog.authorizationPlaceholder')" />
        </el-form-item>
      </template>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">{{ t('dialog.cancel') }}</el-button>
      <el-button type="primary" :disabled="!canSubmit" @click="submit">
        {{ t('dialog.add') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style lang="scss" scoped>
.torrent-upload {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 0;

  .torrent-name {
    color: var(--el-text-color-secondary);
  }
}

.torrent-files {
  margin-top: 8px;
}

.torrent-files-header {
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
  margin-bottom: 4px;
}

.torrent-files-list {
  max-height: 200px;
  overflow-y: auto;
}

.torrent-file-item {
  padding: 4px 0;

  .el-checkbox {
    display: flex;
    width: 100%;
  }

  .file-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: 8px;
  }

  .file-size {
    color: var(--el-text-color-secondary);
    font-size: 12px;
    flex-shrink: 0;
  }
}

.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  margin-bottom: 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  user-select: none;

  &:hover {
    color: var(--el-color-primary);
  }
}
</style>
