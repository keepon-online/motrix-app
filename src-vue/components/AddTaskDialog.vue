<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaskStore } from '@/stores/task'
import { useAppStore } from '@/stores/app'
import { open } from '@tauri-apps/plugin-dialog'
import { readText } from '@tauri-apps/plugin-clipboard-manager'

const { t } = useI18n()
const visible = defineModel<boolean>({ default: false })

const taskStore = useTaskStore()
const appStore = useAppStore()

const activeTab = ref<'uri' | 'torrent'>('uri')
const uriInput = ref('')
const torrentFile = ref<string | null>(null)
const torrentFileName = ref('')
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

// Auto-detect clipboard content when dialog opens
watch(visible, async (val) => {
  if (val) {
    downloadDir.value = appStore.downloadDir
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
  return /^(https?|ftp|magnet|thunder):\/?\/?/i.test(text)
}

async function selectTorrent() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Torrent', extensions: ['torrent'] }],
  })

  if (selected) {
    const { readFile } = await import('@tauri-apps/plugin-fs')
    const contents = await readFile(selected as string)
    torrentFile.value = btoa(String.fromCharCode(...contents))
    torrentFileName.value = (selected as string).split('/').pop() || 'torrent'
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
    dir: downloadDir.value,
    split: split.value,
    'max-connection-per-server': maxConnectionPerServer.value,
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
      await taskStore.addUri(uris, options)
    } else if (torrentFile.value) {
      await taskStore.addTorrent(torrentFile.value, options)
    }

    resetForm()
    visible.value = false
  } catch (error) {
    console.error('Failed to add task:', error)
  }
}

function resetForm() {
  uriInput.value = ''
  torrentFile.value = null
  torrentFileName.value = ''
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
