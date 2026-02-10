<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue'
import { formatSpeed } from '@/utils'

const props = defineProps<{
  downloadSpeed: string
  uploadSpeed: string
}>()

const MAX_POINTS = 60
const downloadHistory = ref<number[]>(new Array(MAX_POINTS).fill(0))
const uploadHistory = ref<number[]>(new Array(MAX_POINTS).fill(0))
const canvasRef = ref<HTMLCanvasElement | null>(null)
let animationFrame: number | null = null

const currentDownload = computed(() => formatSpeed(props.downloadSpeed))
const currentUpload = computed(() => formatSpeed(props.uploadSpeed))

function pushData() {
  const dl = parseInt(props.downloadSpeed) || 0
  const ul = parseInt(props.uploadSpeed) || 0

  downloadHistory.value.push(dl)
  uploadHistory.value.push(ul)

  if (downloadHistory.value.length > MAX_POINTS) {
    downloadHistory.value.shift()
  }
  if (uploadHistory.value.length > MAX_POINTS) {
    uploadHistory.value.shift()
  }
}

function drawChart() {
  const canvas = canvasRef.value
  if (!canvas) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.scale(dpr, dpr)

  const w = rect.width
  const h = rect.height

  // Clear
  ctx.clearRect(0, 0, w, h)

  // Find max value for scale
  const allValues = [...downloadHistory.value, ...uploadHistory.value]
  const maxVal = Math.max(...allValues, 1024) // min 1KB scale

  // Draw grid lines
  ctx.strokeStyle = 'rgba(128, 128, 128, 0.15)'
  ctx.lineWidth = 1
  for (let i = 1; i <= 4; i++) {
    const y = (h / 5) * i
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(w, y)
    ctx.stroke()
  }

  // Draw download line (blue)
  drawLine(ctx, downloadHistory.value, w, h, maxVal, 'rgba(64, 158, 255, 0.8)', 'rgba(64, 158, 255, 0.1)')

  // Draw upload line (green)
  drawLine(ctx, uploadHistory.value, w, h, maxVal, 'rgba(103, 194, 58, 0.8)', 'rgba(103, 194, 58, 0.1)')
}

function drawLine(
  ctx: CanvasRenderingContext2D,
  data: number[],
  w: number,
  h: number,
  maxVal: number,
  strokeColor: string,
  fillColor: string
) {
  const step = w / (MAX_POINTS - 1)

  // Fill area
  ctx.beginPath()
  ctx.moveTo(0, h)
  for (let i = 0; i < data.length; i++) {
    const x = i * step
    const y = h - (data[i] / maxVal) * (h * 0.9)
    if (i === 0) {
      ctx.lineTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }
  ctx.lineTo((data.length - 1) * step, h)
  ctx.closePath()
  ctx.fillStyle = fillColor
  ctx.fill()

  // Stroke line
  ctx.beginPath()
  for (let i = 0; i < data.length; i++) {
    const x = i * step
    const y = h - (data[i] / maxVal) * (h * 0.9)
    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }
  ctx.strokeStyle = strokeColor
  ctx.lineWidth = 2
  ctx.stroke()
}

let interval: number | null = null

onMounted(() => {
  interval = window.setInterval(() => {
    pushData()
    drawChart()
  }, 1000)
  drawChart()
})

onUnmounted(() => {
  if (interval) clearInterval(interval)
  if (animationFrame) cancelAnimationFrame(animationFrame)
})

watch([() => props.downloadSpeed, () => props.uploadSpeed], () => {
  // Data will be pushed on next interval tick
})
</script>

<template>
  <div class="task-activity">
    <div class="activity-legend">
      <div class="legend-item">
        <span class="legend-dot download"></span>
        <span class="legend-label">Download</span>
        <span class="legend-value">{{ currentDownload }}</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot upload"></span>
        <span class="legend-label">Upload</span>
        <span class="legend-value">{{ currentUpload }}</span>
      </div>
    </div>
    <div class="activity-chart">
      <canvas ref="canvasRef" class="chart-canvas"></canvas>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.task-activity {
  min-height: 200px;
}

.activity-legend {
  display: flex;
  gap: 24px;
  margin-bottom: 12px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;

  &.download {
    background: rgba(64, 158, 255, 0.8);
  }

  &.upload {
    background: rgba(103, 194, 58, 0.8);
  }
}

.legend-label {
  color: var(--el-text-color-secondary);
}

.legend-value {
  color: var(--el-text-color-primary);
  font-weight: 500;
}

.activity-chart {
  width: 100%;
  height: 160px;
  background: var(--el-fill-color-lighter);
  border-radius: 6px;
  overflow: hidden;
}

.chart-canvas {
  width: 100%;
  height: 100%;
}
</style>
