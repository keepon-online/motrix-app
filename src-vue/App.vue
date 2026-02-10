<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import { useAria2Events } from '@/composables/useAria2Events'
import TitleBar from '@/components/TitleBar.vue'
import Sidebar from '@/components/Sidebar.vue'
import DragDrop from '@/components/DragDrop.vue'

const appStore = useAppStore()
const { initTheme } = useTheme()

// Setup aria2 event listener
useAria2Events()

onMounted(async () => {
  await appStore.init()
  initTheme()
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
