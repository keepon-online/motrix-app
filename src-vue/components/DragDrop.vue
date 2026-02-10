<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaskStore } from '@/stores/task'
import { useAppStore } from '@/stores/app'
import { ElMessage } from 'element-plus'

const { t } = useI18n()
const taskStore = useTaskStore()
const appStore = useAppStore()
const isDragging = ref(false)

function isUrl(text: string): boolean {
  return /^(https?|ftp|magnet|thunder):/.test(text.trim())
}

function isTorrentFile(name: string): boolean {
  return name.toLowerCase().endsWith('.torrent')
}

async function handleDrop(e: DragEvent) {
  e.preventDefault()
  isDragging.value = false

  if (!e.dataTransfer) return

  // Handle dropped files
  const files = e.dataTransfer.files
  if (files.length > 0) {
    for (const file of Array.from(files)) {
      if (isTorrentFile(file.name)) {
        try {
          const buffer = await file.arrayBuffer()
          const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)))
          await taskStore.addTorrent(base64, { dir: appStore.downloadDir })
          ElMessage.success(`${t('dialog.addTask')}: ${file.name}`)
        } catch (error) {
          ElMessage.error(`${t('task.error')}: ${file.name}`)
        }
      }
    }
    return
  }

  // Handle dropped text (URLs)
  const text = e.dataTransfer.getData('text/plain')
  if (text) {
    const urls = text.split('\n').map(u => u.trim()).filter(u => isUrl(u))
    if (urls.length > 0) {
      try {
        await taskStore.addUri(urls, { dir: appStore.downloadDir })
        ElMessage.success(`${t('dialog.addTask')}: ${urls.length}`)
      } catch (error) {
        ElMessage.error(t('task.error'))
      }
    }
  }
}

function handleDragOver(e: DragEvent) {
  e.preventDefault()
  isDragging.value = true
}

function handleDragLeave(e: DragEvent) {
  e.preventDefault()
  isDragging.value = false
}

onMounted(() => {
  document.addEventListener('drop', handleDrop)
  document.addEventListener('dragover', handleDragOver)
  document.addEventListener('dragleave', handleDragLeave)
})

onUnmounted(() => {
  document.removeEventListener('drop', handleDrop)
  document.removeEventListener('dragover', handleDragOver)
  document.removeEventListener('dragleave', handleDragLeave)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="isDragging" class="drag-overlay">
        <div class="drag-content">
          <el-icon :size="48"><Upload /></el-icon>
          <p>{{ t('dragDrop.hint') }}</p>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style lang="scss" scoped>
.drag-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drag-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px;
  background: var(--el-bg-color);
  border-radius: 16px;
  border: 2px dashed var(--el-color-primary);
  color: var(--el-color-primary);

  p {
    font-size: 16px;
    margin: 0;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
