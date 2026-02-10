<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-shell'

const { t } = useI18n()
</script>

<template>
  <div class="about-view">
    <div class="about-logo">
      <img src="@/assets/logo.svg" alt="Motrix" />
    </div>

    <h1 class="about-title">{{ t('app.name') }}</h1>
    <p class="about-version">{{ t('app.version') }}</p>
    <p class="about-description">{{ t('app.description') }}</p>

    <div class="about-tech">
      <p>{{ t('app.builtWith') }}</p>
      <div class="tech-stack">
        <el-tag>Tauri 2.0</el-tag>
        <el-tag>Vue 3</el-tag>
        <el-tag>Vite</el-tag>
        <el-tag>Aria2</el-tag>
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
