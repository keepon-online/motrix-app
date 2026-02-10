import { useAppStore } from '@/stores/app'
import { useDark, useToggle } from '@vueuse/core'
import { watch } from 'vue'

export function useTheme() {
  const appStore = useAppStore()
  const isDark = useDark({
    selector: 'html',
    attribute: 'class',
    valueDark: 'dark',
    valueLight: '',
  })
  const toggleDark = useToggle(isDark)

  function initTheme() {
    if (!appStore.config) return

    const theme = appStore.config.theme
    if (theme === 'dark') {
      isDark.value = true
    } else if (theme === 'light') {
      isDark.value = false
    }
    // 'auto' will use system preference (handled by useDark)
  }

  watch(
    () => appStore.config?.theme,
    (theme) => {
      if (theme === 'dark') {
        isDark.value = true
      } else if (theme === 'light') {
        isDark.value = false
      }
    }
  )

  async function setTheme(theme: 'auto' | 'light' | 'dark') {
    await appStore.setTheme(theme)
    initTheme()
  }

  return {
    isDark,
    toggleDark,
    initTheme,
    setTheme,
  }
}
