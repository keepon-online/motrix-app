<script setup lang="ts">
import { computed, ref, watch } from 'vue'
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

const reconnecting = ref(false)

async function handleReconnect() {
  reconnecting.value = true
  try {
    await invoke('restart_engine')
  } catch (e) {
    console.error('Failed to restart engine:', e)
  } finally {
    reconnecting.value = false
  }
}

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
const numSeeding = computed(() => taskStore.cachedSeedingCount)

const menuItems = computed(() => [
  { path: '/tasks/active', icon: 'Download', label: t('nav.downloads'), badge: numActive.value },
  { path: '/tasks/seeding', icon: 'Upload', label: t('nav.seeding'), badge: numSeeding.value },
  { path: '/tasks/waiting', icon: 'Clock', label: t('nav.waiting'), badge: numWaiting.value },
  { path: '/tasks/stopped', icon: 'Finished', label: t('nav.stopped'), badge: numStopped.value },
])

const isActive = (path: string) => route.path === path

const dlSpeedText = computed(() => formatSpeed(taskStore.globalStat?.downloadSpeed ?? '0'))
const ulSpeedText = computed(() => formatSpeed(taskStore.globalStat?.uploadSpeed ?? '0'))

// Update dock badge with download speed
watch(() => taskStore.globalStat?.downloadSpeed, (speed) => {
  const s = parseInt(speed ?? '0')
  if (s > 0) {
    invoke('set_dock_badge', { text: formatSpeed(speed ?? '0') }).catch(() => {})
  } else {
    invoke('set_dock_badge', { text: '' }).catch(() => {})
  }
})

// Update window progress bar
watch(() => taskStore.globalStat, (stat) => {
  const active = parseInt(stat?.numActive ?? '0')
  if (active > 0) {
    const tasks = taskStore.tasks.filter(t => t.status === 'active')
    if (tasks.length > 0) {
      const totalSize = tasks.reduce((sum, t) => sum + parseInt(t.totalLength || '0'), 0)
      const completedSize = tasks.reduce((sum, t) => sum + parseInt(t.completedLength || '0'), 0)
      const progress = totalSize > 0 ? Math.round((completedSize / totalSize) * 100) : 0
      invoke('set_window_progress', { progress }).catch(() => {})
    }
  } else {
    invoke('set_window_progress', { progress: 0 }).catch(() => {})
  }
}, { deep: true })

// Update tray speed display
watch(() => taskStore.globalStat, (stat) => {
  invoke('update_tray_speed', {
    downloadSpeed: stat?.downloadSpeed ?? '0',
    uploadSpeed: stat?.uploadSpeed ?? '0',
    enabled: appStore.config?.traySpeedometer ?? false,
  }).catch(() => {})
}, { deep: true })
</script>

<template>
  <aside class="icon-rail">
    <!-- Logo -->
    <div class="rail-logo">
      <img src="@/assets/logo.svg" alt="M" />
    </div>

    <!-- Navigation -->
    <nav class="rail-nav">
      <el-tooltip
        v-for="item in menuItems"
        :key="item.path"
        :content="item.label"
        placement="right"
        :show-after="400"
      >
        <router-link
          :to="item.path"
          class="rail-item"
          :class="{ active: isActive(item.path) }"
        >
          <el-icon :size="22">
            <component :is="item.icon" />
          </el-icon>
          <span v-if="item.badge > 0" class="rail-badge">{{ item.badge > 99 ? '99+' : item.badge }}</span>
        </router-link>
      </el-tooltip>
    </nav>

    <!-- Footer -->
    <div class="rail-footer">
      <!-- Speed display -->
      <el-popover
        :visible="showSpeedMenu"
        placement="right"
        :width="150"
        trigger="click"
        @update:visible="(v: boolean) => showSpeedMenu = v"
      >
        <template #reference>
          <div class="rail-speed" :class="{ limited: isLimited }" @click="showSpeedMenu = !showSpeedMenu">
            <span class="speed-dl">↓ {{ dlSpeedText }}</span>
            <span class="speed-ul">↑ {{ ulSpeedText }}</span>
            <span v-if="isLimited" class="speed-limited-dot" />
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

      <!-- Engine status -->
      <el-tooltip
        :content="appStore.engineReady ? t('engine.connected') : t('engine.disconnected')"
        placement="right"
      >
        <div
          class="rail-engine"
          :class="{ connected: appStore.engineReady, disconnected: !appStore.engineReady }"
          @click="!appStore.engineReady && handleReconnect()"
        >
          <span class="engine-dot" />
          <el-icon v-if="!appStore.engineReady && !reconnecting" :size="12"><RefreshRight /></el-icon>
          <el-icon v-if="reconnecting" :size="12" class="is-loading"><Loading /></el-icon>
        </div>
      </el-tooltip>

      <!-- Settings & About -->
      <el-tooltip :content="t('nav.settings')" placement="right" :show-after="400">
        <router-link to="/settings" class="rail-item" :class="{ active: route.path === '/settings' }">
          <el-icon :size="20"><Setting /></el-icon>
        </router-link>
      </el-tooltip>
      <el-tooltip :content="t('nav.about')" placement="right" :show-after="400">
        <router-link to="/about" class="rail-item" :class="{ active: route.path === '/about' }">
          <el-icon :size="20"><InfoFilled /></el-icon>
        </router-link>
      </el-tooltip>
    </div>
  </aside>
</template>

<style lang="scss" scoped>
.icon-rail {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 56px;
  background: var(--motrix-rail-bg);
  border-right: 1px solid var(--el-border-color-lighter);
  padding: 8px 0;
  flex-shrink: 0;
}

.rail-logo {
  width: 28px;
  height: 28px;
  margin: 4px 0 16px;
  opacity: 0.7;

  img {
    width: 100%;
    height: 100%;
  }
}

.rail-nav {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  flex: 1;
}

.rail-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  color: var(--motrix-rail-text);
  text-decoration: none;
  transition: all 0.2s;
  position: relative;

  &:hover {
    background: var(--el-fill-color-light);
    color: var(--el-text-color-primary);
  }

  &.active {
    background: var(--motrix-rail-active-bg);
    color: var(--motrix-primary);
  }
}

.rail-badge {
  position: absolute;
  top: 2px;
  right: 0px;
  min-width: 16px;
  height: 16px;
  line-height: 16px;
  font-size: 10px;
  font-weight: 600;
  text-align: center;
  border-radius: 8px;
  padding: 0 4px;
  background: var(--motrix-primary);
  color: #fff;
}

.rail-footer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding-top: 8px;
  border-top: 1px solid var(--el-border-color-lighter);
  margin-top: 8px;
}

.rail-speed {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  font-size: 9px;
  font-weight: 500;
  cursor: pointer;
  padding: 6px 4px;
  border-radius: 8px;
  transition: background 0.2s;
  position: relative;
  width: 48px;
  text-align: center;

  &:hover {
    background: var(--el-fill-color-light);
  }

  &.limited {
    background: rgba(230, 162, 60, 0.08);
  }

  .speed-dl {
    color: var(--motrix-dl-color);
    white-space: nowrap;
  }

  .speed-ul {
    color: var(--motrix-ul-color);
    white-space: nowrap;
  }

  .speed-limited-dot {
    position: absolute;
    top: 3px;
    right: 3px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--motrix-pause-color);
  }
}

.rail-engine {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 3px;
  width: 40px;
  height: 24px;
  border-radius: 12px;
  cursor: default;
  transition: all 0.2s;

  .engine-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  &.connected .engine-dot {
    background: var(--el-color-success);
  }

  &.disconnected {
    cursor: pointer;

    .engine-dot {
      background: var(--motrix-error-color);
    }

    &:hover {
      background: var(--el-fill-color-light);
    }
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
  color: var(--motrix-primary);
  font-weight: 600;
}
</style>
