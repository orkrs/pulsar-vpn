<script setup lang="ts">
import { Loader2, X } from 'lucide-vue-next'
import { usePulsar } from '../composables/usePulsar'

const p = usePulsar()
</script>

<template>
  <div v-if="p.showImportModal" class="overlay" @click.self="p.showImportModal = false">
    <div class="modal" role="dialog" aria-modal="true" aria-label="Импорт профиля">
      <div class="m-head">
        <span class="m-title">Импорт профиля</span>
        <button class="m-close" aria-label="Закрыть" @click="p.showImportModal = false"><X :size="15" /></button>
      </div>
      <label class="field">
        <span>vless://, hy2:// или URL подписки</span>
        <input
          v-model="p.importUrl"
          type="url"
          name="import-url"
          autocomplete="off"
          spellcheck="false"
          placeholder="vless://…@host:443?…#Frankfurt"
          @keyup.enter="p.handleImport()"
        />
      </label>
      <div class="m-err" role="alert">{{ p.importError }}</div>
      <button class="btn-accent" style="height:44px" :disabled="p.isImporting" @click="p.handleImport()">
        <Loader2 v-if="p.isImporting" :size="16" class="animate-spin" />
        <span v-else>Добавить</span>
      </button>
    </div>
  </div>
</template>
