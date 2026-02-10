<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { formatSpeed } from '@/utils'

interface Peer {
  peerId: string
  ip: string
  port: string
  bitfield: string
  amChoking: string
  peerChoking: string
  downloadSpeed: string
  uploadSpeed: string
  seeder: string
}

const props = defineProps<{
  gid: string
}>()

const peers = ref<Peer[]>([])
const loading = ref(false)
let refreshInterval: number | null = null

async function fetchPeers() {
  if (!props.gid) return

  loading.value = true
  try {
    const result = await invoke<Peer[]>('get_task_peers', { gid: props.gid })
    peers.value = result || []
  } catch (error) {
    console.error('Failed to fetch peers:', error)
  } finally {
    loading.value = false
  }
}

function getPeerClient(peerId: string): string {
  // Simple peer client detection
  if (!peerId || peerId.startsWith('%00')) return 'Unknown'
  try {
    const decoded = decodeURIComponent(peerId)
    if (decoded.startsWith('-')) {
      const clientCode = decoded.substring(1, 3)
      const clientMap: Record<string, string> = {
        'AZ': 'Azureus',
        'BC': 'BitComet',
        'BT': 'BitTorrent',
        'DE': 'Deluge',
        'LT': 'libtorrent',
        'MO': 'Motrix',
        'QD': 'QQDownload',
        'SD': 'Thunder',
        'TR': 'Transmission',
        'UT': 'uTorrent',
        'XL': 'Xunlei',
        'qB': 'qBittorrent',
      }
      return clientMap[clientCode] || clientCode
    }
    return decoded.substring(0, 8)
  } catch {
    return 'Unknown'
  }
}

onMounted(() => {
  fetchPeers()
  refreshInterval = window.setInterval(fetchPeers, 3000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

watch(() => props.gid, fetchPeers)
</script>

<template>
  <div class="task-peers" v-loading="loading">
    <div v-if="peers.length === 0" class="empty">
      <el-empty description="No peers connected" :image-size="60" />
    </div>
    <el-table v-else :data="peers" size="small" max-height="300">
      <el-table-column label="IP" min-width="120">
        <template #default="{ row }">
          {{ row.ip }}:{{ row.port }}
        </template>
      </el-table-column>
      <el-table-column label="Client" min-width="100">
        <template #default="{ row }">
          {{ getPeerClient(row.peerId) }}
        </template>
      </el-table-column>
      <el-table-column label="Download" width="90" align="right">
        <template #default="{ row }">
          {{ formatSpeed(row.downloadSpeed) }}
        </template>
      </el-table-column>
      <el-table-column label="Upload" width="90" align="right">
        <template #default="{ row }">
          {{ formatSpeed(row.uploadSpeed) }}
        </template>
      </el-table-column>
      <el-table-column label="Seeder" width="60" align="center">
        <template #default="{ row }">
          <el-icon v-if="row.seeder === 'true'" color="var(--el-color-success)">
            <Check />
          </el-icon>
          <span v-else>-</span>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style lang="scss" scoped>
.task-peers {
  min-height: 100px;
}
</style>
