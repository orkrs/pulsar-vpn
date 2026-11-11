<script setup lang="ts">
import { createPulsar } from './composables/usePulsar'
import AppRail from './components/AppRail.vue'
import AppTopbar from './components/AppTopbar.vue'
import ConnectView from './components/ConnectView.vue'
import ProfilesView from './components/ProfilesView.vue'
import SettingsView from './components/SettingsView.vue'
import LogsView from './components/LogsView.vue'
import ImportModal from './components/ImportModal.vue'
import EditNodeModal from './components/EditNodeModal.vue'
import WizardModal from './components/WizardModal.vue'

const p = createPulsar()
</script>

<template>
  <div class="app" :data-vpn="p.vpnPhase" @click="p.closeMenus()">
    <AppRail />
    <main class="stage">
      <AppTopbar />
      <div class="views">
        <ConnectView />
        <ProfilesView />
        <SettingsView />
        <LogsView />
      </div>
    </main>
    <ImportModal />
    <EditNodeModal />
    <WizardModal />
    <div v-if="p.toast" class="toast" role="status" aria-live="polite">{{ p.toast }}</div>
  </div>
</template>
