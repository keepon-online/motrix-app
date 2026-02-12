<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  bitfield: string
  totalLength: string
  completedLength: string
}>()

const canvas = ref<HTMLCanvasElement | null>(null)

// Parse hex bitfield string into array of bits
function parseBitfield(hex: string): boolean[] {
  const bits: boolean[] = []
  for (const ch of hex) {
    const nibble = parseInt(ch, 16)
    if (isNaN(nibble)) continue
    bits.push((nibble & 8) !== 0)
    bits.push((nibble & 4) !== 0)
    bits.push((nibble & 2) !== 0)
    bits.push((nibble & 1) !== 0)
  }
  return bits
}

const progress = computed(() => {
  const total = parseInt(props.totalLength || '0')
  const completed = parseInt(props.completedLength || '0')
  if (total <= 0) return 0
  return Math.min(100, (completed / total) * 100)
})

function draw() {
  const el = canvas.value
  if (!el) return

  const ctx = el.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = el.getBoundingClientRect()
  el.width = rect.width * dpr
  el.height = rect.height * dpr
  ctx.scale(dpr, dpr)

  const w = rect.width
  const h = rect.height

  // Background
  ctx.fillStyle = 'var(--el-fill-color-light, #f5f7fa)'
  ctx.fillRect(0, 0, w, h)

  if (!props.bitfield) {
    // No bitfield data, draw simple progress bar
    const cssVar = getComputedStyle(el).getPropertyValue('--el-color-primary') || '#409eff'
    ctx.fillStyle = cssVar.trim()
    ctx.fillRect(0, 0, w * (progress.value / 100), h)
    return
  }

  const bits = parseBitfield(props.bitfield)
  if (bits.length === 0) return

  const pieceWidth = w / bits.length
  const completedColor = getComputedStyle(el).getPropertyValue('--el-color-primary')?.trim() || '#409eff'
  const emptyColor = getComputedStyle(el).getPropertyValue('--el-fill-color-light')?.trim() || '#f5f7fa'

  // Draw each piece
  for (let i = 0; i < bits.length; i++) {
    ctx.fillStyle = bits[i] ? completedColor : emptyColor
    ctx.fillRect(i * pieceWidth, 0, Math.ceil(pieceWidth), h)
  }
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  draw()
  if (canvas.value) {
    resizeObserver = new ResizeObserver(() => draw())
    resizeObserver.observe(canvas.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

watch(() => props.bitfield, () => draw())
watch(() => props.completedLength, () => draw())
</script>

<template>
  <div class="task-bitfield">
    <canvas ref="canvas" class="bitfield-canvas" />
    <div class="bitfield-info">
      <span>{{ progress.toFixed(1) }}%</span>
      <span v-if="bitfield">{{ parseBitfield(bitfield).filter(b => b).length }} / {{ parseBitfield(bitfield).length }} pieces</span>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.task-bitfield {
  width: 100%;
}

.bitfield-canvas {
  width: 100%;
  height: 20px;
  border-radius: 4px;
  overflow: hidden;
  display: block;
}

.bitfield-info {
  display: flex;
  justify-content: space-between;
  margin-top: 6px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
