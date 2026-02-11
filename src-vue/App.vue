<script setup lang="ts">
import { onMounted, ref, provide } from 'vue'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import { useAria2Events } from '@/composables/useAria2Events'
import { listen } from '@tauri-apps/api/event'
import TitleBar from '@/components/TitleBar.vue'
import Sidebar from '@/components/Sidebar.vue'
import DragDrop from '@/components/DragDrop.vue'

const appStore = useAppStore()
const { initTheme } = useTheme()

// Global state for pending URLs to add (used by AddTaskDialog)
const pendingUrls = ref<string[]>([])
const showAddDialog = ref(false)
provide('pendingUrls', pendingUrls)
provide('showAddDialog', showAddDialog)

// Setup aria2 event listener
useAria2Events()

// Handle incoming URLs from CLI args, second instance, or deep links
function handleIncomingUrls(urls: string[]) {
  if (urls.length === 0) return

  const downloadableUrls: string[] = []
  for (const url of urls) {
    // Handle torrent files directly
    if (url.toLowerCase().endsWith('.torrent')) {
      // Torrent files need special handling - just open dialog
      downloadableUrls.push(url)
    } else {
      // Strip motrix:// prefix if present
      const cleanUrl = url.replace(/^motrix:\/\//, '')
      if (cleanUrl) downloadableUrls.push(cleanUrl)
    }
  }

  if (downloadableUrls.length > 0) {
    pendingUrls.value = downloadableUrls
    showAddDialog.value = true
  }
}

onMounted(async () => {
  await appStore.init()
  initTheme()

  // Listen for URLs from first launch CLI args (non-blocking)
  listen<string[]>('open-urls', (event) => {
    handleIncomingUrls(event.payload)
  }).catch((e) => console.warn('Failed to listen open-urls:', e))

  // Listen for deep link URLs (non-blocking, dynamic import)
  import('@tauri-apps/plugin-deep-link')
    .then(({ onOpenUrl }) => {
      onOpenUrl((urls: string[]) => {
        handleIncomingUrls(urls)
      }).catch((e: unknown) => console.warn('Deep link registration failed:', e))
    })
    .catch((e) => console.warn('Deep link not available:', e))
})
</script>

<template>
  <div class="app-container" :class="{ 'is-dark': appStore.isDark }">
    <TitleBar />
    <div class="app-main">
      <Sidebar />
      <main class="app-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </div>
    <DragDrop />
  </div>
</template>

<style lang="scss">
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--el-bg-color);
  color: var(--el-text-color-primary);
}

.app-main {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.app-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
