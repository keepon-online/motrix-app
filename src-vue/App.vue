<script setup lang="ts">
import { onMounted, ref, provide } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import { useAria2Events } from '@/composables/useAria2Events'
import { useUpdater } from '@/composables/useUpdater'
import { listen } from '@tauri-apps/api/event'
import TitleBar from '@/components/TitleBar.vue'
import Sidebar from '@/components/Sidebar.vue'
import DragDrop from '@/components/DragDrop.vue'

const router = useRouter()
const appStore = useAppStore()
const { initTheme } = useTheme()

// Global state for pending URLs to add (used by AddTaskDialog)
const pendingUrls = ref<string[]>([])
const showAddDialog = ref(false)
provide('pendingUrls', pendingUrls)
provide('showAddDialog', showAddDialog)

// Setup aria2 event listener
useAria2Events()

// Auto-check for updates (delayed, silent)
const { checkForUpdate } = useUpdater()

// Handle incoming URLs from CLI args, second instance, or deep links
function handleIncomingUrls(urls: string[]) {
  if (urls.length === 0) return

  const downloadableUrls: string[] = []
  for (const url of urls) {
    // Handle torrent files directly
    if (url.toLowerCase().endsWith('.torrent')) {
      downloadableUrls.push(url)
      continue
    }

    // Parse structured motrix:// or mo:// URLs
    const parsed = parseMotrixUrl(url)
    if (parsed) {
      downloadableUrls.push(parsed)
      continue
    }

    // Strip motrix:// or mo:// prefix if present (simple form)
    const cleanUrl = url.replace(/^(?:motrix|mo):\/\//, '')
    if (cleanUrl) downloadableUrls.push(cleanUrl)
  }

  if (downloadableUrls.length > 0) {
    pendingUrls.value = downloadableUrls
    showAddDialog.value = true
  }
}

/** Parse structured motrix://new-task?url=xxx or mo://new-task?url=xxx */
function parseMotrixUrl(url: string): string | null {
  const match = url.match(/^(?:motrix|mo):\/\/new-task\?(.+)$/i)
  if (!match) return null
  try {
    const params = new URLSearchParams(match[1])
    return params.get('url') || null
  } catch {
    return null
  }
}

onMounted(async () => {
  await appStore.init()
  initTheme()

  // Listen for URLs from first launch CLI args (non-blocking)
  listen<string[]>('open-urls', (event) => {
    handleIncomingUrls(event.payload)
  }).catch((e) => console.warn('Failed to listen open-urls:', e))

  // Listen for menu events
  listen('menu-add-task', () => {
    showAddDialog.value = true
  }).catch((e) => console.warn('Failed to listen menu-add-task:', e))

  listen('menu-preferences', () => {
    router.push('/settings')
  }).catch((e) => console.warn('Failed to listen menu-preferences:', e))

  // Listen for deep link URLs (non-blocking, dynamic import)
  import('@tauri-apps/plugin-deep-link')
    .then(({ onOpenUrl }) => {
      onOpenUrl((urls: string[]) => {
        handleIncomingUrls(urls)
      }).catch((e: unknown) => console.warn('Deep link registration failed:', e))
    })
    .catch((e) => console.warn('Deep link not available:', e))

  // Auto-check for updates after 5 seconds
  setTimeout(() => checkForUpdate(true), 5000)

  // Auto-sync trackers if 12 hours have passed
  setTimeout(() => appStore.autoSyncTrackers(), 8000)
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
  padding: 20px 24px;
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
