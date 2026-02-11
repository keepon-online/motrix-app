<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useTaskStore } from '@/stores/task'
import { useAppStore } from '@/stores/app'
import { formatSpeed } from '@/utils'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()
const route = useRoute()
const taskStore = useTaskStore()
const appStore = useAppStore()

const speedLimitPresets = ['0', '128K', '512K', '1M', '5M', '10M']
const showSpeedMenu = ref(false)

const isLimited = computed(() => {
  const limit = appStore.config?.maxOverallDownloadLimit || '0'
  return limit !== '0'
})

async function setSpeedLimit(limit: string) {
  await appStore.saveConfig({
    maxOverallDownloadLimit: limit,
    maxOverallUploadLimit: limit,
  })
  try {
    await invoke('change_global_option', {
      options: {
        'max-overall-download-limit': limit,
        'max-overall-upload-limit': limit,
      }
    })
  } catch (e) {
    console.warn('Failed to set speed limit:', e)
  }
  showSpeedMenu.value = false
}

const numActive = computed(() => parseInt(taskStore.globalStat?.numActive ?? '0'))
const numWaiting = computed(() => parseInt(taskStore.globalStat?.numWaiting ?? '0'))
const numStopped = computed(() => parseInt(taskStore.globalStat?.numStopped ?? '0'))

const menuItems = computed(() => [
  { path: '/tasks/active', icon: 'Download', label: t('nav.downloads'), badge: numActive.value },
  { path: '/tasks/waiting', icon: 'Clock', label: t('nav.waiting'), badge: numWaiting.value },
  { path: '/tasks/stopped', icon: 'Finished', label: t('nav.stopped'), badge: numStopped.value },
])

const isActive = (path: string) => {
  return route.path === path
}

const downloadSpeedText = computed(() => formatSpeed(taskStore.globalStat?.downloadSpeed ?? '0'))
const uploadSpeedText = computed(() => formatSpeed(taskStore.globalStat?.uploadSpeed ?? '0'))
</script>

<template>
  <aside class="sidebar">
    <nav class="sidebar-nav">
      <router-link
        v-for="item in menuItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path) }"
      >
        <el-icon :size="20">
          <component :is="item.icon" />
        </el-icon>
        <span class="nav-label">{{ item.label }}</span>
        <span v-if="item.badge > 0" class="nav-badge">{{ item.badge > 99 ? '99+' : item.badge }}</span>
      </router-link>
    </nav>

    <div class="sidebar-footer">
      <el-popover
        :visible="showSpeedMenu"
        placement="top"
        :width="160"
        trigger="click"
        @update:visible="(v: boolean) => showSpeedMenu = v"
      >
        <template #reference>
          <div class="speed-info" :class="{ limited: isLimited }" @click="showSpeedMenu = !showSpeedMenu">
            <div class="speed-item">
              <el-icon><Download /></el-icon>
              <span>{{ downloadSpeedText }}</span>
            </div>
            <div class="speed-item">
              <el-icon><Upload /></el-icon>
              <span>{{ uploadSpeedText }}</span>
            </div>
            <div v-if="isLimited" class="speed-limit-tag">
              {{ t('task.speedLimited') }}
            </div>
          </div>
        </template>
        <div class="speed-menu">
          <div
            v-for="preset in speedLimitPresets"
            :key="preset"
            class="speed-menu-item"
            :class="{ active: (appStore.config?.maxOverallDownloadLimit || '0') === preset }"
            @click="setSpeedLimit(preset)"
          >
            {{ preset === '0' ? t('task.noLimit') : preset + '/s' }}
          </div>
        </div>
      </el-popover>

      <div class="sidebar-actions">
        <router-link to="/settings" class="action-btn" :class="{ active: route.path === '/settings' }">
          <el-icon :size="20"><Setting /></el-icon>
        </router-link>
        <router-link to="/about" class="action-btn" :class="{ active: route.path === '/about' }">
          <el-icon :size="20"><InfoFilled /></el-icon>
        </router-link>
      </div>
    </div>
  </aside>
</template>

<style lang="scss" scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  width: 180px;
  background: var(--el-bg-color-page);
  border-right: 1px solid var(--el-border-color-lighter);
}

.sidebar-nav {
  flex: 1;
  padding: 16px 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 4px;
  border-radius: 8px;
  color: var(--el-text-color-regular);
  text-decoration: none;
  transition: all 0.2s;

  &:hover {
    background: var(--el-fill-color-light);
    color: var(--el-text-color-primary);
  }

  &.active {
    background: var(--el-color-primary-light-9);
    color: var(--el-color-primary);
  }

  .nav-label {
    font-size: 14px;
    flex: 1;
  }

  .nav-badge {
    font-size: 11px;
    min-width: 18px;
    height: 18px;
    line-height: 18px;
    text-align: center;
    border-radius: 9px;
    padding: 0 5px;
    background: var(--el-color-primary);
    color: #fff;
    flex-shrink: 0;
  }

  &.active .nav-badge {
    background: var(--el-color-primary);
  }
}

.sidebar-footer {
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.speed-info {
  margin-bottom: 16px;
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  transition: all 0.2s;

  &:hover {
    background: var(--el-fill-color-light);
  }

  &.limited {
    border: 1px solid var(--el-color-warning-light-5);
    background: var(--el-color-warning-light-9);
  }

  .speed-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--el-text-color-secondary);
    margin-bottom: 4px;
  }

  .speed-limit-tag {
    font-size: 11px;
    color: var(--el-color-warning);
    margin-top: 4px;
    text-align: center;
  }
}

.sidebar-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  color: var(--el-text-color-regular);
  transition: all 0.2s;

  &:hover {
    background: var(--el-fill-color-light);
    color: var(--el-text-color-primary);
  }

  &.active {
    background: var(--el-color-primary-light-9);
    color: var(--el-color-primary);
  }
}
</style>

<style>
.speed-menu .speed-menu-item {
  padding: 6px 12px;
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  transition: background 0.2s;
}
.speed-menu .speed-menu-item:hover {
  background: var(--el-fill-color-light);
}
.speed-menu .speed-menu-item.active {
  color: var(--el-color-primary);
  font-weight: 600;
}
</style>
