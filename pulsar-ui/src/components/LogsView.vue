<script setup lang="ts">
import { ref, watch } from 'vue'
import { usePulsar } from '../composables/usePulsar'

const p = usePulsar()
const termEl = ref<HTMLElement | null>(null)

watch(termEl, (el) => { p.logsContainer = el })
</script>

<template>
  <section class="view view-logs" :class="{ on: p.activeTab === 'logs' }">
    <div class="logs-head">
      <div class="seg" style="height:36px" role="group" aria-label="Фильтр логов">
        <button :class="{ on: p.logFilter === 'all' }" @click="p.logFilter = 'all'">Все</button>
        <button :class="{ on: p.logFilter === 'info' }" @click="p.logFilter = 'info'">Инфо</button>
        <button :class="{ on: p.logFilter === 'err' }" @click="p.logFilter = 'err'">Ошибки</button>
      </div>
      <button class="btn-accent" style="margin-left:auto;height:36px" @click="p.logs.splice(0, p.logs.length)">Очистить</button>
    </div>
    <div ref="termEl" class="term" aria-live="polite" aria-relevant="additions">
      <div
        v-for="(log, i) in p.filteredLogs"
        :key="i"
        class="tl"
        :class="log.includes('ERROR') || log.includes('[ERROR]') ? 'err' : log.includes('[SYSTEM]') ? 'sys' : 'info'"
      >{{ log }}</div>
    </div>
  </section>
</template>
