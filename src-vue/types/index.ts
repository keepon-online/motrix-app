// Task types
export interface Task {
  gid: string
  status: TaskStatus
  totalLength: string
  completedLength: string
  uploadLength: string
  downloadSpeed: string
  uploadSpeed: string
  connections: string
  numSeeders?: string
  dir: string
  files: TaskFile[]
  bittorrent?: BittorrentInfo
  errorCode?: string
  errorMessage?: string
  bitfield?: string
}

export type TaskStatus = 'active' | 'waiting' | 'paused' | 'error' | 'complete' | 'removed'

export interface TaskFile {
  index: string
  path: string
  length: string
  completedLength: string
  selected: string
  uris: TaskUri[]
}

export interface TaskUri {
  uri: string
  status: string
}

export interface BittorrentInfo {
  announceList?: string[][]
  comment?: string
  creationDate?: number
  mode?: string
  info?: {
    name: string
  }
}

// Global statistics
export interface GlobalStat {
  downloadSpeed: string
  uploadSpeed: string
  numActive: string
  numWaiting: string
  numStopped: string
  numStoppedTotal: string
}

// Run mode types
export type RunMode = 'standard' | 'tray' | 'hide_tray'

// Proxy scope types
export type ProxyScope = 'all' | 'http' | 'https' | 'ftp'

// Configuration types
export interface AppConfig {
  locale: string
  theme: 'auto' | 'light' | 'dark'
  downloadDir: string
  autoStart: boolean
  startHidden: boolean
  hideOnClose: boolean
  notifyOnComplete: boolean
  autoClearCompleted: boolean
  resumeAllWhenAppLaunched: boolean
  runMode: RunMode
  maxConcurrentDownloads: number
  maxConnectionPerServer: number
  split: number
  minSplitSize: string
  maxDownloadLimit: string
  maxUploadLimit: string
  btListenPort: number
  dhtListenPort: number
  enableUpnp: boolean
  seedRatio: number
  seedTime: number
  btTracker: string
  trackerSource: string[]
  btForceEncryption: boolean
  btRequireCrypto: boolean
  pauseMetadata: boolean
  userAgent: string
  proxyEnabled: boolean
  proxyType: 'http' | 'https' | 'socks5'
  proxyScope: ProxyScope
  proxyHost: string
  proxyPort: number
  proxyUsername: string
  proxyPassword: string
  noProxy: string
  rpcPort: number
  rpcSecret: string
  maxOverallDownloadLimit: string
  maxOverallUploadLimit: string
  allowOverwrite: boolean
  autoFileRenaming: boolean
  continueDownload: boolean
  followMetalink: string
  lastTrackerUpdate: number
  btSaveMetadata: boolean
  btLoadSavedMetadata: boolean
  btRemoveUnselectedFile: boolean
  btDetachSeedOnly: boolean
}

// App data paths
export interface AppDataPaths {
  appDataDir: string
  appConfigDir: string
  logDir: string
}

// Add task options
export interface AddTaskOptions {
  dir?: string
  out?: string
  split?: number
  maxConnectionPerServer?: number
  selectFile?: string
  [key: string]: unknown
}
