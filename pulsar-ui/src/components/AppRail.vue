<script setup lang="ts">
import { usePulsar } from '../composables/usePulsar'
import type { TabId } from '../types'

const p = usePulsar()

const tabs: { id: TabId; label: string; path: string }[] = [
  { id: 'dashboard', label: 'Подключение', path: 'M12 2l8 3v6c0 5-3.5 9.3-8 11-4.5-1.7-8-6-8-11V5l8-3z' },
  { id: 'nodes', label: 'Серверы', path: 'globe' },
  { id: 'settings', label: 'Настройки', path: 'sliders' },
  { id: 'logs', label: 'Журнал', path: 'term' },
]
</script>

<template>
  <aside class="rail" aria-label="Навигация">
    <svg class="logo" viewBox="0 0 34 34" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <circle cx="17" cy="17" r="4" fill="currentColor" stroke="none" />
      <path d="M17 3a14 14 0 0 1 14 14" />
      <path d="M17 31A14 14 0 0 1 3 17" opacity=".45" />
      <circle class="logo-core" cx="17" cy="17" r="1.6" fill="#fff" stroke="none" opacity=".7" />
    </svg>
    <nav class="nav">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="nav-btn"
        :class="{ on: p.activeTab === t.id }"
        :title="t.label"
        :aria-label="t.label"
        :aria-current="p.activeTab === t.id ? 'page' : undefined"
        @click="p.activeTab = t.id"
      >
        <svg v-if="t.id === 'dashboard'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 2l8 3v6c0 5-3.5 9.3-8 11-4.5-1.7-8-6-8-11V5l8-3z"/></svg>
        <svg v-else-if="t.id === 'nodes'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.8 2.6 4.2 5.6 4.2 9S14.8 18.4 12 21c-2.8-2.6-4.2-5.6-4.2-9S9.2 5.6 12 3z"/></svg>
        <svg v-else-if="t.id === 'settings'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" aria-hidden="true"><path d="M4 8h10M18 8h2M4 16h2M10 16h10"/><circle cx="16" cy="8" r="2.2"/><circle cx="8" cy="16" r="2.2"/></svg>
        <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2.5"/><path d="M7 9l3.5 3L7 15M12.5 15H17"/></svg>
      </button>
    </nav>
    <span class="ver" translate="no">v0.1</span>
  </aside>
</template>
