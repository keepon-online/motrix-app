<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-shell'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { useUpdater } from '@/composables/useUpdater'

const { t } = useI18n()
const appVersion = ref('')
const aria2Version = ref('')
const { status, newVersion, downloadProgress, errorMessage, checkForUpdate, installAndRelaunch } = useUpdater()

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '2.0.0'
  }
  try {
    const info = await invoke<{ version: string }>('get_engine_version')
    aria2Version.value = info.version
  } catch {
    // aria2 may not be ready yet
  }
})
</script>

<template>
  <div class="about-view">
    <div class="about-logo">
      <img src="@/assets/logo.svg" alt="Motrix" />
    </div>

    <h1 class="about-title">{{ t('app.name') }}</h1>
    <p class="about-version">{{ t('app.version', { version: appVersion }) }}</p>

    <div class="about-update">
      <el-button
        v-if="status === 'idle' || status === 'upToDate' || status === 'error' || status === 'checking'"
        size="small"
        @click="checkForUpdate()"
        :loading="status === 'checking'"
      >
        {{ status === 'upToDate' ? t('about.upToDate') : t('about.checkUpdate') }}
      </el-button>
      <div v-else-if="status === 'downloading'" class="update-progress">
        <span>{{ t('about.downloading', { version: newVersion }) }}</span>
        <el-progress :percentage="downloadProgress" :stroke-width="6" style="width: 200px" />
      </div>
      <el-button
        v-else-if="status === 'ready'"
        size="small"
        type="primary"
        @click="installAndRelaunch()"
      >
        {{ t('about.installAndRestart') }}
      </el-button>
      <p v-if="status === 'error'" class="update-error">{{ errorMessage }}</p>
    </div>

    <p class="about-description">{{ t('app.description') }}</p>

    <div class="about-tech">
      <p>{{ t('app.builtWith') }}</p>
      <div class="tech-stack">
        <el-tag>Tauri 2.0</el-tag>
        <el-tag>Vue 3</el-tag>
        <el-tag>Vite</el-tag>
        <el-tag v-if="aria2Version">Aria2 {{ aria2Version }}</el-tag>
        <el-tag v-else>Aria2</el-tag>
        <el-tag>Rust</el-tag>
      </div>
    </div>

    <div class="about-links">
      <el-button text @click="open('https://motrix.app')">
        <el-icon><Link /></el-icon>
        {{ t('about.website') }}
      </el-button>
      <el-button text @click="open('https://github.com/agalwood/Motrix')">
        <el-icon><Link /></el-icon>
        {{ t('about.github') }}
      </el-button>
      <el-button text @click="open('https://github.com/agalwood/Motrix/releases')">
        <el-icon><Link /></el-icon>
        {{ t('about.releases') }}
      </el-button>
    </div>

    <div class="about-copyright">
      <p>{{ t('app.copyright') }}</p>
      <p>{{ t('app.license') }}</p>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.about-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
}

.about-logo {
  width: 96px;
  height: 96px;
  margin-bottom: 16px;

  img {
    width: 100%;
    height: 100%;
  }
}

.about-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin: 0 0 8px;
}

.about-version {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin: 0 0 8px;
}

.about-update {
  margin-bottom: 8px;

  .update-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--el-text-color-secondary);
  }

  .update-error {
    font-size: 12px;
    color: var(--el-color-danger);
    margin: 4px 0 0;
  }
}

.about-description {
  font-size: 16px;
  color: var(--el-text-color-regular);
  margin: 0 0 24px;
}

.about-tech {
  margin-bottom: 24px;

  p {
    font-size: 12px;
    color: var(--el-text-color-secondary);
    margin: 0 0 8px;
  }

  .tech-stack {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: center;
  }
}

.about-links {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
}

.about-copyright {
  font-size: 12px;
  color: var(--el-text-color-secondary);

  p {
    margin: 0 0 4px;
  }
}
</style>
