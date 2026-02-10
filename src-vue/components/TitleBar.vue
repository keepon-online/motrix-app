<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/app'

const appWindow = getCurrentWindow()
const appStore = useAppStore()
const isMaximized = ref(false)

async function minimize() {
  await appWindow.minimize()
}

async function toggleMaximize() {
  if (await appWindow.isMaximized()) {
    await appWindow.unmaximize()
    isMaximized.value = false
  } else {
    await appWindow.maximize()
    isMaximized.value = true
  }
}

async function close() {
  // Hide to tray if configured, otherwise close
  if (appStore.config?.hideOnClose) {
    await appWindow.hide()
  } else {
    await appWindow.close()
  }
}

// Check initial state
appWindow.isMaximized().then((maximized) => {
  isMaximized.value = maximized
})
</script>

<template>
  <div class="title-bar" data-tauri-drag-region>
    <div class="title-bar-left">
      <img src="@/assets/logo.svg" alt="Motrix" class="logo" />
      <span class="title">Motrix</span>
    </div>

    <div class="title-bar-right">
      <button class="title-bar-btn" @click="minimize" title="Minimize">
        <el-icon><Minus /></el-icon>
      </button>
      <button class="title-bar-btn" @click="toggleMaximize" :title="isMaximized ? 'Restore' : 'Maximize'">
        <el-icon>
          <FullScreen v-if="!isMaximized" />
          <CopyDocument v-else />
        </el-icon>
      </button>
      <button class="title-bar-btn close" @click="close" title="Close">
        <el-icon><Close /></el-icon>
      </button>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.title-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 32px;
  padding: 0 8px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  user-select: none;
  -webkit-app-region: drag;
}

.title-bar-left {
  display: flex;
  align-items: center;
  gap: 8px;

  .logo {
    width: 18px;
    height: 18px;
  }

  .title {
    font-size: 13px;
    font-weight: 500;
    color: var(--el-text-color-primary);
  }
}

.title-bar-right {
  display: flex;
  -webkit-app-region: no-drag;
}

.title-bar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--el-text-color-regular);
  cursor: pointer;
  transition: background-color 0.2s;

  &:hover {
    background: var(--el-fill-color-light);
  }

  &.close:hover {
    background: #e81123;
    color: white;
  }
}
</style>
