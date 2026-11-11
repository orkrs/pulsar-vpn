<script setup lang="ts">
import PulsarSphere from './PulsarSphere.vue'
import { usePulsar } from '../composables/usePulsar'
import { formatPing, pingClass } from '../types'

const p = usePulsar()
</script>

<template>
  <section class="view view-connect" :class="{ on: p.activeTab === 'dashboard' }">
    <div class="c-left">
      <div>
        <h1 class="status-word" :class="{ shake: p.statusShake }">{{ p.statusWord }}</h1>
        <p class="status-sub" v-html="p.statusSubHtml" />
      </div>

      <div
        class="profile-card"
        role="button"
        tabindex="0"
        @click="p.activeTab = 'nodes'"
        @keydown.enter="p.activeTab = 'nodes'"
      >
        <span class="pc-mark" aria-hidden="true">{{ p.activeProfile.flag }}</span>
        <span class="pc-body">
          <span class="pc-name">{{ p.activeProfile.name }}</span>
          <span class="pc-meta">
            <span class="badge">{{ p.activeProfile.protocol }}</span>
            <span class="ping" :class="pingClass(p.activeProfile.ping)"><i /><!--
            -->{{ formatPing(p.activeProfile.ping) }}</span>
          </span>
        </span>
        <span class="pc-change">Сменить</span>
      </div>

      <div class="stats">
        <div class="stat">
          <div class="stat-label">Приём</div>
          <div class="stat-val">{{ p.isConnected ? p.downloadSpeed : '0.0' }}<small>Мбит/с</small></div>
        </div>
        <div class="stat">
          <div class="stat-label">Отдача</div>
          <div class="stat-val">{{ p.isConnected ? p.uploadSpeed : '0.0' }}<small>Мбит/с</small></div>
        </div>
        <div class="stat">
          <div class="stat-label">Сессия</div>
          <div class="stat-val">{{ p.sessionLabel }}</div>
        </div>
      </div>

      <div class="cta-row">
        <button class="power" :class="{ busy: p.vpnBusy }" :aria-label="p.ctaLabel" @click="p.toggleVpn()">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M12 3v9" /><path d="M6.3 6.5a8 8 0 1 0 11.4 0" />
          </svg>
        </button>
        <span class="cta-text">
          {{ p.ctaLabel }}
          <span class="cta-hint">{{ p.isConnected ? 'Соединение активно' : 'Нажмите для туннеля' }}</span>
        </span>
      </div>
    </div>

    <div class="sphere-wrap">
      <PulsarSphere
        :speed="p.currentMetrics.download + p.currentMetrics.upload"
        :is-connected="p.isConnected"
        :busy="p.vpnBusy"
        :active="!p.isMobile && p.activeTab === 'dashboard'"
      />
    </div>
  </section>
</template>
