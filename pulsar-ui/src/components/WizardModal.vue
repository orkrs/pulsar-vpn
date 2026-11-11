<script setup lang="ts">
import { Loader2, Sparkles, ArrowRight, Check } from 'lucide-vue-next'
import { usePulsar } from '../composables/usePulsar'

const p = usePulsar()
</script>

<template>
  <div v-if="p.showWizard" class="overlay">
    <div class="modal" role="dialog" aria-modal="true" aria-label="Мастер настройки" style="max-width:480px;text-align:center">
      <div v-if="p.wizardStep === 1" style="display:flex;flex-direction:column;align-items:center;gap:16px">
        <Sparkles :size="40" style="color:var(--accent)" aria-hidden="true" />
        <h2 class="m-title" style="font-size:22px">Добро пожаловать в Pulsar VPN</h2>
        <p class="gd">Безопасный туннель. Настройка займёт минуту.</p>
        <button class="btn-accent" style="height:44px;width:100%" @click="p.wizardStep = 2">
          Начать <ArrowRight :size="16" />
        </button>
      </div>
      <div v-else-if="p.wizardStep === 2" style="display:flex;flex-direction:column;gap:14px;text-align:left">
        <h2 class="m-title">Добавьте первую подписку</h2>
        <label class="field">
          <span>Ссылка</span>
          <input
            v-model="p.importUrl"
            type="url"
            autocomplete="off"
            spellcheck="false"
            placeholder="https://… или vless://…"
            @keyup.enter="p.handleWizardImport()"
          />
        </label>
        <button class="btn-accent" style="height:44px" :disabled="p.isImporting" @click="p.handleWizardImport()">
          <Loader2 v-if="p.isImporting" :size="16" class="animate-spin" />
          <span v-else>Импорт</span>
        </button>
        <button class="btn-ghost" style="height:40px;justify-content:center" @click="p.wizardStep = 3">Пропустить</button>
      </div>
      <div v-else style="display:flex;flex-direction:column;align-items:center;gap:16px">
        <Check :size="40" style="color:var(--good)" aria-hidden="true" />
        <h2 class="m-title" style="font-size:22px">Готово</h2>
        <p class="gd">Можно подключаться.</p>
        <button class="btn-accent" style="height:44px;width:100%" @click="p.completeWizard()">К дашборду</button>
      </div>
      <div style="display:flex;gap:6px;justify-content:center;margin-top:8px">
        <span
          v-for="s in 3"
          :key="s"
          :style="{
            width: '8px',
            height: '8px',
            borderRadius: '99px',
            display: 'inline-block',
            background: p.wizardStep === s ? 'var(--accent)' : 'var(--surface3)',
          }"
        />
      </div>
    </div>
  </div>
</template>
