<script setup lang="ts">
import {
  ShieldCheck, Globe2, Settings, Terminal,
} from 'lucide-vue-next'

const props = defineProps<{
  activeTab: string
  showLogs: boolean
}>()

const emit = defineEmits<{
  (e: 'navigate', tab: string): void
  (e: 'toggle-logs'): void
}>()

const items = [
  { id: 'dashboard', label: 'Shield', icon: ShieldCheck },
  { id: 'nodes', label: 'Nodes', icon: Globe2 },
  { id: 'settings', label: 'Setup', icon: Settings },
] as const
</script>

<template>
  <nav class="bottom-nav glass-panel">
    <button
        v-for="item in items"
        :key="item.id"
        class="nav-item"
        :class="{ active: props.activeTab === item.id }"
        @click="emit('navigate', item.id)"
    >
      <component :is="item.icon" class="w-5 h-5"/>
      <span class="label">{{ item.label }}</span>
    </button>
    <button
        class="nav-item"
        :class="{ active: props.showLogs }"
        @click="emit('toggle-logs')"
    >
      <Terminal class="w-5 h-5"/>
      <span class="label">Logs</span>
    </button>
  </nav>
</template>

<style scoped>
.bottom-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: calc(64px + env(safe-area-inset-bottom, 0px));
  padding-bottom: env(safe-area-inset-bottom, 0px);
  z-index: 50;
  display: flex;
  align-items: stretch;
  justify-content: space-around;
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  border-top-left-radius: 0;
  border-top-right-radius: 0;
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
  border-top: 1px solid rgba(253, 253, 253, 0.05);
  background: rgba(18, 18, 18, 0.85);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  pointer-events: auto;
}
.nav-item {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  color: rgba(253, 253, 253, 0.4);
  transition: color 0.2s, transform 0.15s;
  min-width: 44px;
  min-height: 44px;
  background: transparent;
  border: 0;
  padding: 0;
  cursor: pointer;
  pointer-events: auto;
  -webkit-tap-highlight-color: transparent;
  font: inherit;
  color: rgba(253, 253, 253, 0.4);
}
.nav-item:active {
  transform: scale(0.92);
}
.nav-item.active {
  color: #9d68ff;
}
.label {
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
</style>
