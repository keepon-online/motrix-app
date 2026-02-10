<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useTaskStore } from '@/stores/task'
import { formatSpeed } from '@/utils'

const route = useRoute()
const taskStore = useTaskStore()

const menuItems = [
  { path: '/tasks/active', icon: 'Download', label: 'Downloads' },
  { path: '/tasks/waiting', icon: 'Clock', label: 'Waiting' },
  { path: '/tasks/stopped', icon: 'Finished', label: 'Completed' },
]

const isActive = (path: string) => {
  return route.path === path
}

const downloadSpeedText = computed(() => formatSpeed(taskStore.downloadSpeed))
const uploadSpeedText = computed(() => formatSpeed(taskStore.uploadSpeed))
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
      </router-link>
    </nav>

    <div class="sidebar-footer">
      <div class="speed-info">
        <div class="speed-item">
          <el-icon><Download /></el-icon>
          <span>{{ downloadSpeedText }}</span>
        </div>
        <div class="speed-item">
          <el-icon><Upload /></el-icon>
          <span>{{ uploadSpeedText }}</span>
        </div>
      </div>

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
  }
}

.sidebar-footer {
  padding: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.speed-info {
  margin-bottom: 16px;

  .speed-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--el-text-color-secondary);
    margin-bottom: 4px;
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
