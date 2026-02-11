import { useAppStore } from '@/stores/app'
import { useDark } from '@vueuse/core'
import { watch, onScopeDispose } from 'vue'

export function useTheme() {
  const appStore = useAppStore()
  const isDark = useDark({
    selector: 'html',
    attribute: 'class',
    valueDark: 'dark',
    valueLight: '',
  })

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

  function applyTheme(theme: string | undefined) {
    if (theme === 'dark') {
      isDark.value = true
    } else if (theme === 'light') {
      isDark.value = false
    } else {
      // 'auto' - follow system preference
      isDark.value = mediaQuery.matches
    }
  }

  function onSystemThemeChange(e: MediaQueryListEvent) {
    if (appStore.config?.theme === 'auto') {
      isDark.value = e.matches
    }
  }

  mediaQuery.addEventListener('change', onSystemThemeChange)
  onScopeDispose(() => {
    mediaQuery.removeEventListener('change', onSystemThemeChange)
  })

  function initTheme() {
    if (!appStore.config) return
    applyTheme(appStore.config.theme)
  }

  watch(() => appStore.config?.theme, applyTheme)

  async function setTheme(theme: 'auto' | 'light' | 'dark') {
    await appStore.setTheme(theme)
    initTheme()
  }

  return {
    isDark,
    initTheme,
    setTheme,
  }
}
