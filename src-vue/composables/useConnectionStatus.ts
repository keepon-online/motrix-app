import { computed } from 'vue'
import { useAria2Diagnostics } from '@/composables/useAria2Diagnostics'

export function useConnectionStatus() {
  const { status, engineReady } = useAria2Diagnostics()
  const isDisconnected = computed(() => status.value === 'disconnected' || status.value === 'terminated')

  return {
    status,
    isDisconnected,
    engineReady,
  }
}
