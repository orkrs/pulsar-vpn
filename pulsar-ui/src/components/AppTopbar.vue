<script setup lang="ts">
import { ClipboardPaste } from 'lucide-vue-next'
import { usePulsar } from '../composables/usePulsar'

const p = usePulsar()
</script>

<template>
  <header class="topbar">
    <span class="tb-title">{{ p.tabTitle }}</span>
    <span class="tb-status"><span class="dot" aria-hidden="true" />{{ p.tbLabel }}</span>
    <button class="btn-ghost" style="height:36px;font-size:12px" @click="p.quickAdd()" title="Вставить из буфера" aria-label="Вставить из буфера">
      <ClipboardPaste :size="14" aria-hidden="true" /> Paste
    </button>
  </header>
</template>
