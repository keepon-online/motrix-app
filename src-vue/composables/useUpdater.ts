import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'upToDate' | 'error'

const status = ref<UpdateStatus>('idle')
const errorMessage = ref('')
const newVersion = ref('')
const downloadProgress = ref(0)

export function useUpdater() {
  async function checkForUpdate(silent = false) {
    if (status.value === 'checking' || status.value === 'downloading') return

    status.value = 'checking'
    errorMessage.value = ''

    try {
      const update = await check()
      if (update) {
        status.value = 'available'
        newVersion.value = update.version

        let totalBytes = 0
        let downloadedBytes = 0
        status.value = 'downloading'

        await update.downloadAndInstall((event) => {
          if (event.event === 'Started' && event.data.contentLength) {
            totalBytes = event.data.contentLength
          } else if (event.event === 'Progress') {
            downloadedBytes += event.data.chunkLength
            if (totalBytes > 0) {
              downloadProgress.value = Math.round((downloadedBytes / totalBytes) * 100)
            }
          } else if (event.event === 'Finished') {
            status.value = 'ready'
          }
        })

        status.value = 'ready'
      } else {
        status.value = 'upToDate'
      }
    } catch (e) {
      if (!silent) {
        status.value = 'error'
        errorMessage.value = String(e)
      } else {
        status.value = 'idle'
      }
    }
  }

  async function installAndRelaunch() {
    await relaunch()
  }

  return {
    status,
    errorMessage,
    newVersion,
    downloadProgress,
    checkForUpdate,
    installAndRelaunch,
  }
}
