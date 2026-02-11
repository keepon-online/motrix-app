import { useAppStore } from '@/stores/app'
import { useDark } from '@vueuse/core'
import { watch } from 'vue'

export function useTheme() {
  const appStore = useAppStore()
  const isDark = useDark({
    selector: 'html',
    attribute: 'class',
    valueDark: 'dark',
    valueLight: '',
  })

  function initTheme() {
    if (!appStore.config) return

    const theme = appStore.config.theme
    if (theme === 'dark') {
      isDark.value = true
    } else if (theme === 'light') {
      isDark.value = false
    } else {
      // 'auto' - follow system preference
      isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
    }
  }

  watch(
    () => appStore.config?.theme,
    (theme) => {
      if (theme === 'dark') {
        isDark.value = true
      } else if (theme === 'light') {
        isDark.value = false
      } else if (theme === 'auto') {
        isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
      }
    }
  )

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
