<script setup lang="ts">
import { computed } from 'vue'
import { formatSpeed } from '@/utils'

const props = defineProps<{
  downloadSpeed: string | number
  uploadSpeed: string | number
}>()

const maxSpeed = 10 * 1024 * 1024 // 10 MB/s as visual max

// SVG arc parameters
const cx = 60
const cy = 55
const r = 45
const startAngle = 135
const endAngle = 405
const totalAngle = endAngle - startAngle

function polarToCartesian(angle: number) {
  const rad = (angle * Math.PI) / 180
  return {
    x: cx + r * Math.cos(rad),
    y: cy + r * Math.sin(rad),
  }
}

function describeArc(start: number, end: number) {
  const s = polarToCartesian(start)
  const e = polarToCartesian(end)
  const largeArc = end - start > 180 ? 1 : 0
  return `M ${s.x} ${s.y} A ${r} ${r} 0 ${largeArc} 1 ${e.x} ${e.y}`
}

const bgArc = computed(() => describeArc(startAngle, endAngle))

const dlSpeed = computed(() => {
  const s = typeof props.downloadSpeed === 'string' ? parseInt(props.downloadSpeed) : props.downloadSpeed
  return isFinite(s) ? s : 0
})

const dlAngle = computed(() => {
  const ratio = Math.min(dlSpeed.value / maxSpeed, 1)
  return startAngle + ratio * totalAngle
})

const dlArc = computed(() => {
  if (dlSpeed.value <= 0) return ''
  return describeArc(startAngle, dlAngle.value)
})

const dlText = computed(() => formatSpeed(props.downloadSpeed))
const ulText = computed(() => formatSpeed(props.uploadSpeed))
</script>

<template>
  <div class="speedometer">
    <svg viewBox="0 0 120 80" class="speedometer-svg">
      <!-- Background arc -->
      <path :d="bgArc" fill="none" stroke="var(--el-border-color-lighter)" stroke-width="6" stroke-linecap="round" />
      <!-- Download speed arc -->
      <path v-if="dlArc" :d="dlArc" fill="none" stroke="var(--el-color-primary)" stroke-width="6" stroke-linecap="round" />
    </svg>
    <div class="speed-labels">
      <div class="speed-dl">
        <span class="speed-icon">↓</span>
        <span class="speed-value">{{ dlText }}</span>
      </div>
      <div class="speed-ul">
        <span class="speed-icon">↑</span>
        <span class="speed-value">{{ ulText }}</span>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.speedometer {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.speedometer-svg {
  width: 100%;
  max-width: 120px;
  height: auto;
}

.speed-labels {
  display: flex;
  gap: 12px;
  margin-top: 2px;
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.speed-dl, .speed-ul {
  display: flex;
  align-items: center;
  gap: 2px;
}

.speed-icon {
  font-weight: 600;
  font-size: 10px;
}

.speed-dl .speed-icon {
  color: var(--el-color-primary);
}

.speed-ul .speed-icon {
  color: var(--el-color-success);
}
</style>
