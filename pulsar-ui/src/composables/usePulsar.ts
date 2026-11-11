import { ref, computed, watch, onMounted, provide, inject, reactive, type InjectionKey } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { load } from '@tauri-apps/plugin-store'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { logger } from '../utils/logger'
import type {
  Profile, Subscription, RoutingConfig, PlatformInfo, NodeGroup, TabId, VpnPhase,
} from '../types'
import { POPULAR_COUNTRIES } from '../types'

export type PulsarApi = ReturnType<typeof createPulsar>
export const PULSAR_KEY: InjectionKey<PulsarApi> = Symbol('pulsar')

export function usePulsar() {
  const api = inject(PULSAR_KEY)
  if (!api) throw new Error('Pulsar not provided')
  return api
}

export function createPulsar() {
  const isConnected = ref(false)
  const vpnBusy = ref(false)
  const vpnPhase = ref<VpnPhase>('off')
  const isImporting = ref(false)
  const isPinging = ref(false)
  const currentMetrics = ref({ download: 0, upload: 0 })
  const activeTab = ref<TabId>('dashboard')
  const toast = ref<string | null>(null)
  let toastTimer: number | null = null

  const showWizard = ref(false)
  const wizardStep = ref(1)
  const showImportModal = ref(false)
  const importError = ref('')
  const logFilter = ref<'all' | 'info' | 'err'>('all')
  const openMenuId = ref<number | null>(null)
  const statusShake = ref(false)
  const connectedAt = ref<number | null>(null)
  const sessionNow = ref(Date.now())
  let sessionTimer: number | null = null

  let store: Awaited<ReturnType<typeof load>> | null = null
  const importUrl = ref('')
  const searchQuery = ref('')
  const sortBy = ref<'name' | 'ping'>('name')
  const sortOrder = ref<'asc' | 'desc'>('asc')
  const subscriptions = ref<Subscription[]>([])
  const standaloneProfiles = ref<Profile[]>([])
  const expandedSubs = ref<Set<string>>(new Set())

  const editModalOpen = ref(false)
  const editingProfileId = ref<number | null>(null)
  const editedDnsType = ref<'https' | 'http' | 'udp'>('https')
  const editedAllowInsecure = ref(false)
  const editedUseIpv6 = ref(false)

  const logs = ref<string[]>([
    '[SYSTEM] Pulsar Engine Initialized',
    '[SYSTEM] Waiting for connection protocol…',
  ])
  const logsContainer = ref<HTMLElement | null>(null)

  const settings = ref({
    autoStart: false,
    minimizeToTray: true,
    setupWizardCompleted: false,
    enableGeoip: false,
    geoipRules: {} as Record<string, 'proxy' | 'direct' | 'block'>,
  })

  const platformInfo = ref<PlatformInfo>({
    os: 'unknown',
    apps_label: 'Applications',
    apps_placeholder: 'process name',
    apps_append_suffix: false,
    apps_supported: false,
  })

  const routing = ref<RoutingConfig>({ mode: 'bypass', domains: [], apps: [] })
  const newDomainInput = ref('')
  const newAppInput = ref('')
  const isMobile = ref(false)

  const profiles = computed(() => [
    ...subscriptions.value.flatMap(s => s.nodes),
    ...standaloneProfiles.value,
  ])

  const downloadSpeed = computed(() =>
    ((currentMetrics.value.download * 8) / (1024 * 1024)).toFixed(1),
  )
  const uploadSpeed = computed(() =>
    ((currentMetrics.value.upload * 8) / (1024 * 1024)).toFixed(1),
  )

  const activeProfile = computed(() =>
    profiles.value.find(p => p.active) || {
      id: -1,
      name: 'Узел не выбран',
      protocol: '—',
      ping: -1,
      active: false,
      rawLink: '',
      flag: '🌍',
      country: '',
      dnsType: 'https' as const,
      allowInsecure: false,
      useIpv6: false,
    },
  )

  const sessionLabel = computed(() => {
    if (!connectedAt.value || !isConnected.value) return '00:00'
    const s = Math.floor((sessionNow.value - connectedAt.value) / 1000)
    const m = Math.floor(s / 60)
    const h = Math.floor(m / 60)
    if (h > 0) {
      return `${pad(h)}:${pad(m % 60)}:${pad(s % 60)}`
    }
    return `${pad(m)}:${pad(s % 60)}`
  })

  const tabTitle = computed(() =>
    ({ dashboard: 'Подключение', nodes: 'Серверы', settings: 'Настройки', logs: 'Журнал' })[activeTab.value],
  )

  const statusWord = computed(() => {
    if (vpnPhase.value === 'connecting') return 'ПОДКЛЮЧЕНИЕ…'
    if (vpnPhase.value === 'disconnecting') return 'ОТКЛЮЧЕНИЕ…'
    return isConnected.value ? 'ЗАЩИЩЕНО' : 'НЕ ЗАЩИЩЕНО'
  })

  const statusSubHtml = computed(() => {
    if (vpnBusy.value) return 'Обмен рукопожатием с узлом…'
    if (isConnected.value) return `Туннель через <b>${escapeHtml(activeProfile.value.name)}</b>`
    return 'Трафик идёт напрямую. Подключитесь для шифрования.'
  })

  const tbLabel = computed(() => {
    if (vpnPhase.value === 'connecting') return 'Подключение…'
    if (vpnPhase.value === 'disconnecting') return 'Отключение…'
    return isConnected.value ? 'Защищено' : 'Отключено'
  })

  const ctaLabel = computed(() => {
    if (vpnPhase.value === 'connecting') return 'Подключение…'
    if (vpnPhase.value === 'disconnecting') return 'Отключение…'
    return isConnected.value ? 'Отключить' : 'Подключить'
  })

  const filteredLogs = computed(() => {
    if (logFilter.value === 'all') return logs.value
    if (logFilter.value === 'err') return logs.value.filter(l => /error|fail|err/i.test(l))
    return logs.value.filter(l => !/error|fail|err/i.test(l))
  })

  const nodeGroups = computed<NodeGroup[]>(() => {
    const groups: NodeGroup[] = subscriptions.value.map(sub => ({
      id: sub.id, name: sub.name, sub, nodes: sub.nodes,
    }))
    if (standaloneProfiles.value.length > 0) {
      groups.push({
        id: '__standalone__',
        name: 'Локальные узлы',
        sub: null,
        nodes: standaloneProfiles.value,
      })
    }
    return groups
  })

  function pad(n: number) { return String(n).padStart(2, '0') }
  function escapeHtml(s: string) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }

  function showToast(msg: string) {
    toast.value = msg
    if (toastTimer) window.clearTimeout(toastTimer)
    toastTimer = window.setTimeout(() => { toast.value = null }, 2200)
  }

  function startSessionClock() {
    stopSessionClock()
    sessionNow.value = Date.now()
    sessionTimer = window.setInterval(() => { sessionNow.value = Date.now() }, 1000)
  }
  function stopSessionClock() {
    if (sessionTimer) { window.clearInterval(sessionTimer); sessionTimer = null }
  }

  function getFilteredAndSortedNodes(nodes: Profile[]) {
    if (!nodes) return []
    let result = nodes
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      result = nodes.filter(p =>
        p.name.toLowerCase().includes(q)
        || p.protocol.toLowerCase().includes(q)
        || p.country.toLowerCase().includes(q),
      )
    }
    return [...result].sort((a, b) => {
      const mod = sortOrder.value === 'asc' ? 1 : -1
      if (sortBy.value === 'name') return a.name.localeCompare(b.name) * mod
      const w = (p: number) => (p > 0 ? p : p === 0 ? 999998 : 999999)
      return (w(a.ping) - w(b.ping)) * mod
    })
  }

  function toggleSort(field: 'name' | 'ping') {
    if (sortBy.value === field) sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
    else { sortBy.value = field; sortOrder.value = 'asc' }
  }

  let pendingLogs: string[] = []
  let logFlushScheduled = false
  function flushLogs() {
    logFlushScheduled = false
    if (!pendingLogs.length) return
    logs.value.push(...pendingLogs)
    pendingLogs = []
    if (logs.value.length > 500) logs.value.splice(0, logs.value.length - 500)
    if (logsContainer.value) logsContainer.value.scrollTop = logsContainer.value.scrollHeight
  }
  function addLog(msg: string) {
    pendingLogs.push(msg)
    if (!logFlushScheduled) {
      logFlushScheduled = true
      requestAnimationFrame(flushLogs)
    }
  }

  async function saveData() {
    if (!store) return
    await store.set('subscriptions', subscriptions.value)
    await store.set('standaloneProfiles', standaloneProfiles.value)
    await store.set('routing', routing.value)
    await store.set('settings', settings.value)
    await store.save()
  }

  async function toggleVpn() {
    if (vpnBusy.value) return
    if (!profiles.value.length) {
      showToast('Сначала добавьте узел')
      statusShake.value = true
      setTimeout(() => { statusShake.value = false }, 400)
      activeTab.value = 'nodes'
      return
    }
    const active = profiles.value.find(p => p.active)
    if (!active?.rawLink) {
      showToast('Выберите узел')
      activeTab.value = 'nodes'
      return
    }

    const target = !isConnected.value
    vpnBusy.value = true
    vpnPhase.value = target ? 'connecting' : 'disconnecting'
    try {
      addLog(`[SYSTEM] Attempting to ${target ? 'connect' : 'disconnect'}…`)
      const res = await invoke<string>('toggle_vpn', {
        payload: {
          connect: target,
          link: active.rawLink,
          routing_config: {
            mode: routing.value.mode,
            domains: routing.value.domains,
            apps: routing.value.apps,
            enable_geoip: settings.value.enableGeoip,
            geoip_rules: settings.value.geoipRules,
            dns_type: active.dnsType || 'https',
            allow_insecure: active.allowInsecure ?? false,
            use_ipv6: active.useIpv6 ?? false,
          },
        },
      })
      isConnected.value = res === 'Connected'
      vpnPhase.value = isConnected.value ? 'on' : 'off'
      if (isConnected.value) {
        connectedAt.value = Date.now()
        startSessionClock()
      } else {
        connectedAt.value = null
        stopSessionClock()
      }
      addLog(`[SYSTEM] ${res}`)
    } catch (e) {
      addLog(`[ERROR] ${e}`)
      isConnected.value = false
      vpnPhase.value = 'off'
      connectedAt.value = null
      stopSessionClock()
      statusShake.value = true
      setTimeout(() => { statusShake.value = false }, 400)
    } finally {
      vpnBusy.value = false
    }
  }

  async function toggleAutoStart() {
    try {
      if (settings.value.autoStart) {
        await disable()
        settings.value.autoStart = false
      } else {
        await enable()
        settings.value.autoStart = true
      }
      await saveData()
    } catch (e) {
      showToast('Автозапуск: ' + e)
    }
  }

  async function toggleTray() {
    settings.value.minimizeToTray = !settings.value.minimizeToTray
    await invoke('set_minimize_to_tray', { enable: settings.value.minimizeToTray })
    await saveData()
  }

  async function refreshPings() {
    if (isPinging.value) return
    isPinging.value = true
    addLog('[SYSTEM] Started latency check…')
    profiles.value.forEach(p => { p.ping = 0 })
    await Promise.allSettled(profiles.value.map(async (p) => {
      try { p.ping = await invoke<number>('check_ping', { link: p.rawLink }) }
      catch { p.ping = -1 }
    }))
    await saveData()
    isPinging.value = false
  }

  function openEditModal(profile: Profile) {
    editingProfileId.value = profile.id
    editedDnsType.value = profile.dnsType || 'https'
    editedAllowInsecure.value = profile.allowInsecure ?? false
    editedUseIpv6.value = profile.useIpv6 ?? false
    editModalOpen.value = true
    openMenuId.value = null
  }

  async function saveProfileSettings() {
    if (editingProfileId.value === null) return
    const apply = (nodes: Profile[]) => {
      const p = nodes.find(n => n.id === editingProfileId.value)
      if (p) {
        p.dnsType = editedDnsType.value
        p.allowInsecure = editedAllowInsecure.value
        p.useIpv6 = editedUseIpv6.value
      }
    }
    apply(standaloneProfiles.value)
    subscriptions.value.forEach(s => apply(s.nodes))
    await saveData()
    editModalOpen.value = false
    editingProfileId.value = null
  }

  async function completeWizard() {
    settings.value.setupWizardCompleted = true
    await saveData()
    showWizard.value = false
  }

  async function handleWizardImport() {
    await handleImport()
    if (profiles.value.length > 0) wizardStep.value = 3
  }

  function addDomain() {
    if (!newDomainInput.value.trim()) return
    routing.value.domains.push(newDomainInput.value.trim())
    newDomainInput.value = ''
    saveData()
  }
  function removeDomain(idx: number) {
    routing.value.domains.splice(idx, 1)
    saveData()
  }
  function addApp() {
    if (!newAppInput.value.trim()) return
    let app = newAppInput.value.trim()
    if (platformInfo.value.apps_append_suffix && !app.toLowerCase().endsWith('.exe')) app += '.exe'
    routing.value.apps.push(app)
    newAppInput.value = ''
    saveData()
  }
  function removeApp(idx: number) {
    routing.value.apps.splice(idx, 1)
    saveData()
  }

  async function setActiveProfile(id: number) {
    subscriptions.value.forEach(s => s.nodes.forEach(p => { p.active = p.id === id }))
    standaloneProfiles.value.forEach(p => { p.active = p.id === id })
    await saveData()
  }

  async function deleteProfile(id: number) {
    const wasActive = profiles.value.find(p => p.id === id)?.active
    standaloneProfiles.value = standaloneProfiles.value.filter(p => p.id !== id)
    subscriptions.value.forEach(s => { s.nodes = s.nodes.filter(p => p.id !== id) })
    if (wasActive) {
      if (profiles.value.length) profiles.value[0].active = true
      else isConnected.value = false
    }
    openMenuId.value = null
    await saveData()
  }

  async function deleteSubscription(id: string) {
    const sub = subscriptions.value.find(s => s.id === id)
    if (sub?.nodes.some(n => n.active)) isConnected.value = false
    subscriptions.value = subscriptions.value.filter(s => s.id !== id)
    expandedSubs.value.delete(id)
    await saveData()
  }

  function parseLocationFromName(name: string) {
    const match = name.match(/[\u{1F1E6}-\u{1F1FF}]{2}/u)
    let country = 'Unknown'
    if (name.includes('RU')) country = 'Russia'
    else if (name.includes('DE')) country = 'Germany'
    else if (name.includes('NL')) country = 'Netherlands'
    else if (name.includes('US')) country = 'USA'
    else if (name.includes('PL')) country = 'Poland'
    return { flag: match ? match[0] : '🌍', country }
  }

  function createProfileObj(url: string, subId?: string): Profile | null {
    let protocol = 'Custom'
    let name = 'Node'
    if (url.startsWith('vless://')) protocol = 'VLESS'
    else if (url.startsWith('wireguard://')) protocol = 'WireGuard'
    else if (url.startsWith('warp://')) protocol = 'WARP'
    else if (url.startsWith('ss://')) protocol = 'Shadowsocks'
    else if (url.startsWith('trojan://')) protocol = 'Trojan'
    else if (url.startsWith('hysteria2://') || url.startsWith('hy2://')) protocol = 'Hysteria2'

    const hashIndex = url.indexOf('#')
    if (hashIndex !== -1) {
      try { name = decodeURIComponent(url.slice(hashIndex + 1)) } catch { /* ignore */ }
    }
    const { flag, country } = parseLocationFromName(name)
    return {
      id: Date.now() + Math.random(),
      name, protocol, ping: 0, active: false, rawLink: url, flag, country,
      subscriptionId: subId, dnsType: 'https', allowInsecure: false, useIpv6: false,
    }
  }

  async function handleImport() {
    if (!importUrl.value) return
    const input = importUrl.value.trim()
    importUrl.value = ''
    isImporting.value = true
    importError.value = ''
    try {
      if (input.startsWith('http')) {
        addLog('[SYSTEM] Fetching subscription…')
        const res = await invoke<{ nodes: string[]; userinfo: string | null }>('fetch_subscription', { url: input })
        const subId = Date.now().toString()
        const newNodes = res.nodes.map(l => createProfileObj(l, subId)).filter(Boolean) as Profile[]
        let expireTime: number | undefined
        if (res.userinfo) {
          const m = res.userinfo.match(/expire=(\d+)/)
          if (m) expireTime = parseInt(m[1]) * 1000
        }
        if (!profiles.value.length && newNodes.length) newNodes[0].active = true
        subscriptions.value.push({
          id: subId, url: input, name: new URL(input).hostname,
          lastUpdated: Date.now(), expire: expireTime, nodes: newNodes,
        })
        expandedSubs.value.add(subId)
        expandedSubs.value = new Set(expandedSubs.value)
        addLog(`[SYSTEM] Imported subscription with ${newNodes.length} nodes`)
      } else {
        const p = createProfileObj(input)
        if (p) {
          if (!profiles.value.length) p.active = true
          standaloneProfiles.value.push(p)
          expandedSubs.value.add('__standalone__')
          expandedSubs.value = new Set(expandedSubs.value)
          addLog(`[SYSTEM] Imported standalone profile: ${p.name}`)
        }
      }
      await saveData()
      refreshPings()
      showImportModal.value = false
    } catch (e) {
      addLog(`[ERROR] Import failed: ${e}`)
      importError.value = String(e)
    } finally {
      isImporting.value = false
    }
  }

  async function refreshSubscription(sub: Subscription) {
    addLog(`[SYSTEM] Updating subscription ${sub.name}…`)
    try {
      const res = await invoke<{ nodes: string[]; userinfo: string | null }>('fetch_subscription', { url: sub.url })
      const activeLink = profiles.value.find(p => p.active)?.rawLink
      const newNodes = res.nodes.map(l => createProfileObj(l, sub.id)).filter(Boolean) as Profile[]
      if (activeLink) {
        const found = newNodes.find(n => n.rawLink === activeLink)
        if (found) found.active = true
      }
      sub.nodes = newNodes
      sub.lastUpdated = Date.now()
      if (res.userinfo) {
        const m = res.userinfo.match(/expire=(\d+)/)
        if (m) sub.expire = parseInt(m[1]) * 1000
      }
      await saveData()
      refreshPings()
      addLog(`[SYSTEM] Subscription updated. Total nodes: ${newNodes.length}`)
    } catch (e) {
      addLog(`[ERROR] Failed to update subscription: ${e}`)
    }
  }

  function toggleSubExpanded(id: string) {
    if (expandedSubs.value.has(id)) expandedSubs.value.delete(id)
    else expandedSubs.value.add(id)
    expandedSubs.value = new Set(expandedSubs.value)
  }

  async function quickAdd() {
    try {
      let text = ''
      try {
        text = platformInfo.value.os === 'android'
          ? await invoke<string>('read_clipboard')
          : await navigator.clipboard.readText()
      } catch (e) {
        addLog(`[WARN] clipboard read: ${e}`)
      }
      const trimmed = (text || '').trim()
      if (!trimmed) { showToast('Буфер пуст'); return }
      const ok = /^(https?:|vless:\/\/|vmess:\/\/|trojan:\/\/|ss:\/\/|hy2:\/\/|hysteria2:\/\/|wireguard:\/\/|warp:\/\/)/.test(trimmed)
      importUrl.value = trimmed
      if (ok) {
        await handleImport()
        showToast('Импортировано из буфера')
      } else {
        showImportModal.value = true
        showToast('Вставлено — проверьте и добавьте')
      }
    } catch {
      showToast('Не удалось вставить')
    }
  }

  function openImport() {
    importError.value = ''
    showImportModal.value = true
  }

  function cycleGeo(code: string) {
    const cur = settings.value.geoipRules[code] || 'proxy'
    settings.value.geoipRules[code] = cur === 'proxy' ? 'direct' : cur === 'direct' ? 'block' : 'proxy'
    saveData()
  }

  function closeMenus() { openMenuId.value = null }

  watch(activeTab, (tab) => {
    searchQuery.value = ''
    invoke('set_logs_streaming', { enable: tab === 'logs' }).catch(() => {})
  })

  onMounted(async () => {
    logger.info('App', 'Vue app mounting')
    const mq = window.matchMedia('(max-width: 760px)')
    isMobile.value = mq.matches
    mq.addEventListener('change', e => { isMobile.value = e.matches })

    try {
      platformInfo.value = await invoke<PlatformInfo>('get_platform_info')
      const vpnState = await invoke<string>('get_vpn_state')
      isConnected.value = vpnState === 'Connected' || vpnState === 'Connecting'
      vpnPhase.value = vpnState === 'Connected' ? 'on' : vpnState === 'Connecting' ? 'connecting' : 'off'
      if (isConnected.value) { connectedAt.value = Date.now(); startSessionClock() }
    } catch (e) {
      logger.warn('App', 'get_platform_info failed', e)
    }

    try {
      store = await load('store.json', { autoSave: false, defaults: {} })
      const savedSubs = await store.get('subscriptions')
      if (savedSubs) subscriptions.value = savedSubs as Subscription[]
      const savedStandalone = await store.get('standaloneProfiles')
      if (savedStandalone) standaloneProfiles.value = savedStandalone as Profile[]
      const oldProfiles = await store.get('profiles')
      if (oldProfiles && (!savedStandalone || !(savedStandalone as Profile[]).length)) {
        standaloneProfiles.value = oldProfiles as Profile[]
        await store.delete('profiles')
      }
      const migrate = (p: Profile) => ({
        ...p,
        dnsType: p.dnsType || 'https' as const,
        allowInsecure: p.allowInsecure ?? false,
        useIpv6: p.useIpv6 ?? false,
      })
      standaloneProfiles.value = standaloneProfiles.value.map(migrate)
      subscriptions.value.forEach(s => { s.nodes = s.nodes.map(migrate) })

      const savedRouting = await store.get('routing')
      if (savedRouting) routing.value = savedRouting as RoutingConfig
      const savedSettings = await store.get('settings')
      if (savedSettings) {
        settings.value = savedSettings as typeof settings.value
        if (!settings.value.geoipRules) settings.value.geoipRules = {}
      }
      settings.value.autoStart = await isEnabled()
      await invoke('set_minimize_to_tray', { enable: settings.value.minimizeToTray })
      if (!settings.value.setupWizardCompleted) showWizard.value = true
    } catch (e) {
      logger.error('App', 'Store init error', e)
    }

    if (!expandedSubs.value.size) {
      subscriptions.value.forEach(s => expandedSubs.value.add(s.id))
      if (standaloneProfiles.value.length) expandedSubs.value.add('__standalone__')
      expandedSubs.value = new Set(expandedSubs.value)
    }
    if (profiles.value.length) refreshPings()

    try { await invoke('set_logs_streaming', { enable: activeTab.value === 'logs' }) } catch { /* */ }

    if ((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      try {
        await listen<{ download: number; upload: number }>('vpn_metrics', e => {
          currentMetrics.value = e.payload
        })
        await listen<string>('vpn_log', e => addLog(`[CORE] ${e.payload}`))
        for (let i = 0; i < 60; i++) {
          try { await invoke('start_metrics_stream'); break }
          catch { if (i < 59) await new Promise(r => setTimeout(r, 2000)) }
        }
      } catch (e) {
        console.error('Failed to init streams:', e)
      }
    }
  })

  const api = {
    POPULAR_COUNTRIES,
    isConnected, vpnBusy, vpnPhase, isImporting, isPinging, currentMetrics,
    activeTab, toast, showWizard, wizardStep, showImportModal, importError,
    logFilter, openMenuId, statusShake, importUrl, searchQuery, sortBy, sortOrder,
    subscriptions, standaloneProfiles, expandedSubs,
    editModalOpen, editedDnsType, editedAllowInsecure, editedUseIpv6,
    logs, logsContainer, settings, platformInfo, routing,
    newDomainInput, newAppInput, isMobile,
    profiles, downloadSpeed, uploadSpeed, activeProfile, sessionLabel,
    tabTitle, statusWord, statusSubHtml, tbLabel, ctaLabel, filteredLogs, nodeGroups,
    showToast, toggleVpn, toggleAutoStart, toggleTray, refreshPings,
    openEditModal, saveProfileSettings, completeWizard, handleWizardImport,
    addDomain, removeDomain, addApp, removeApp,
    setActiveProfile, deleteProfile, deleteSubscription,
    handleImport, refreshSubscription, toggleSubExpanded, toggleSort,
    getFilteredAndSortedNodes, quickAdd, openImport, cycleGeo, closeMenus,
    saveData,
  }

  const reactiveApi = reactive(api)
  provide(PULSAR_KEY, reactiveApi)
  return reactiveApi
}
