import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import pkg from './package.json'

// https://vitejs.dev/config/
export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  plugins: [
    vue(),
    AutoImport({
      imports: ['vue', 'vue-router', 'pinia', '@vueuse/core'],
      resolvers: [ElementPlusResolver()],
      dts: 'src-vue/types/auto-imports.d.ts',
    }),
    Components({
      resolvers: [ElementPlusResolver()],
      dts: 'src-vue/types/components.d.ts',
    }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src-vue'),
    },
  },
  // Tauri expects a fixed port
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  // Build configuration for Tauri
  build: {
    target: 'esnext',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  // Environment variables prefix
  envPrefix: ['VITE_', 'TAURI_'],
  // Pre-bundle all Element Plus on-demand imports to avoid repeated reloads
  optimizeDeps: {
    include: [
      'element-plus/es/components/empty/style/css',
      'element-plus/es/components/button/style/css',
      'element-plus/es/components/button-group/style/css',
      'element-plus/es/components/select/style/css',
      'element-plus/es/components/option/style/css',
      'element-plus/es/components/input/style/css',
      'element-plus/es/components/input-number/style/css',
      'element-plus/es/components/checkbox/style/css',
      'element-plus/es/components/progress/style/css',
      'element-plus/es/components/drawer/style/css',
      'element-plus/es/components/alert/style/css',
      'element-plus/es/components/tabs/style/css',
      'element-plus/es/components/tab-pane/style/css',
      'element-plus/es/components/tag/style/css',
      'element-plus/es/components/dialog/style/css',
      'element-plus/es/components/form/style/css',
      'element-plus/es/components/form-item/style/css',
      'element-plus/es/components/slider/style/css',
      'element-plus/es/components/divider/style/css',
      'element-plus/es/components/loading/style/css',
      'element-plus/es/components/table/style/css',
      'element-plus/es/components/table-column/style/css',
      'element-plus/es/components/switch/style/css',
      'element-plus/es/components/radio-group/style/css',
      'element-plus/es/components/radio-button/style/css',
      'element-plus/es/components/message/style/css',
      'element-plus/es/components/message-box/style/css',
      'element-plus/es/components/notification/style/css',
      'element-plus/es/components/icon/style/css',
      'element-plus/es/components/tooltip/style/css',
      'element-plus/es/components/scrollbar/style/css',
      'element-plus/es/components/popper/style/css',
    ],
  },
})
