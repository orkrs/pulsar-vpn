<script setup lang="ts">
import {ref, computed, onMounted, watch} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'
import {load} from '@tauri-apps/plugin-store'
import {enable, disable, isEnabled} from '@tauri-apps/plugin-autostart'
import {
  ShieldCheck, Globe2, Settings, ArrowDownToLine, ArrowUpFromLine,
  Lock, Zap, Server, Plus, Link, Trash2, CheckCircle2,
  Power, ShieldAlert, Search, Loader2, X, Monitor, Globe,
  Terminal, Trash, MonitorCheck, RefreshCcw,
  ArrowDownWideNarrow, ArrowUpNarrowWide, ChevronDown, ChevronRight,
  Sparkles, ArrowRight, Check, Settings2, ClipboardPaste
} from 'lucide-vue-next'
import PulsarSphere from './components/PulsarSphere.vue'
import BottomNav from './components/BottomNav.vue'
import {logger} from './utils/logger'

// --- ИНТЕРФЕЙСЫ ---
interface Subscription {
  id: string
  url: string
  name: string
  expire?: number
  lastUpdated: number
  nodes: Profile[]
}

interface Profile {
  id: number
  name: string
  protocol: string
  ping: number // 0 = Pinging, -1 = Timeout, >0 = ms
  active: boolean
  rawLink: string
  flag: string
  country: string
  subscriptionId?: string
  dnsType?: 'https' | 'http' | 'udp'
  allowInsecure?: boolean // ДОБАВЛЕНО
  /** Включить IPv6 в TUN/DNS/dial; по умолчанию выкл. (только IPv4). */
  useIpv6?: boolean
}

interface RoutingConfig {
  mode: 'global' | 'bypass' | 'proxy'
  domains: string[]
  apps: string[]
}

// --- СОСТОЯНИЕ ---
const isConnected = ref(false)
const isImporting = ref(false)
const isPinging = ref(false)

const currentMetrics = ref({download: 0, upload: 0})

const activeTab = ref<'dashboard' | 'nodes' | 'settings'>('dashboard')
const showLogs = ref(false)
const toast = ref<string | null>(null)
let toastTimer: number | null = null
const showToast = (msg: string) => {
  toast.value = msg
  if (toastTimer) window.clearTimeout(toastTimer)
  toastTimer = window.setTimeout(() => { toast.value = null }, 2200)
}

// WIZARD STATE
const showWizard = ref(false)
const wizardStep = ref(1)

let store: Awaited<ReturnType<typeof load>> | null = null

const importUrl = ref('')
const searchQuery = ref('')

// СОРТИРОВКА
const sortBy = ref<'name' | 'ping'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')

const subscriptions = ref<Subscription[]>([])
const standaloneProfiles = ref<Profile[]>([])
const expandedSubs = ref<Set<string>>(new Set())

// СОСТОЯНИЕ МОДАЛЬНОГО ОКНА НАСТРОЕК ПРОФИЛЯ
const editModalOpen = ref(false)
const editingProfileId = ref<number | null>(null)
const editedDnsType = ref<'https' | 'http' | 'udp'>('https')
const editedAllowInsecure = ref(false) // ДОБАВЛЕНО
const editedUseIpv6 = ref(false)

const logs = ref<string[]>([
  '[SYSTEM] Pulsar Engine Initialized',
  '[SYSTEM] Waiting for connection protocol...'
])
const logsContainer = ref<HTMLElement | null>(null)

// --- НОВЫЙ СПИСОК СТРАН ДЛЯ GEOIP ---
const popularCountries =[
  { code: 'RU', name: 'Russia', flag: '🇷🇺' },
  { code: 'US', name: 'United States', flag: '🇺🇸' },
  { code: 'DE', name: 'Germany', flag: '🇩🇪' },
  { code: 'GB', name: 'United Kingdom', flag: '🇬🇧' },
  { code: 'FR', name: 'France', flag: '🇫🇷' },
  { code: 'NL', name: 'Netherlands', flag: '🇳🇱' },
  { code: 'CA', name: 'Canada', flag: '🇨🇦' },
  { code: 'AU', name: 'Australia', flag: '🇦🇺' },
  { code: 'JP', name: 'Japan', flag: '🇯🇵' },
  { code: 'CN', name: 'China', flag: '🇨🇳' },
  { code: 'BR', name: 'Brazil', flag: '🇧🇷' },
  { code: 'IN', name: 'India', flag: '🇮🇳' },
  { code: 'IT', name: 'Italy', flag: '🇮🇹' },
  { code: 'ES', name: 'Spain', flag: '🇪🇸' },
  { code: 'SE', name: 'Sweden', flag: '🇸🇪' },
  { code: 'NO', name: 'Norway', flag: '🇳🇴' },
  { code: 'FI', name: 'Finland', flag: '🇫🇮' },
  { code: 'DK', name: 'Denmark', flag: '🇩🇰' },
  { code: 'PL', name: 'Poland', flag: '🇵🇱' },
  { code: 'UA', name: 'Ukraine', flag: '🇺🇦' },
];

const settings = ref({
  autoStart: false,
  minimizeToTray: true,
  setupWizardCompleted: false,
  enableGeoip: false,
  geoipRules: {} as Record<string, 'proxy'|'direct'|'block'>
})

// --- ПЛАТФОРМЕННАЯ ИНФО ---
// get_platform_info (Tauri-команда) сообщает UI, какие ОС-специфичные
// фичи активны: apps-list работает только на Windows/Linux/macOS
// (на Android — только когда будет реализован VpnService.Builder).
interface PlatformInfo {
  os: string
  apps_label: string
  apps_placeholder: string
  apps_append_suffix: boolean
  apps_supported: boolean
}
const platformInfo = ref<PlatformInfo>({
  os: 'unknown',
  apps_label: 'Applications',
  apps_placeholder: 'process name',
  apps_append_suffix: false,
  apps_supported: false
})

const routing = ref<RoutingConfig>({
  mode: 'bypass',
  domains: [],
  apps:[]
})

const newDomainInput = ref('')
const newAppInput = ref('')

// --- COMPUTED ---
const profiles = computed(() => {
  const subsNodes = subscriptions.value.flatMap(s => s.nodes)
  return[...subsNodes, ...standaloneProfiles.value]
})

const downloadSpeed = computed(() => ((currentMetrics.value.download * 8) / (1024 * 1024)).toFixed(1))
const uploadSpeed = computed(() => ((currentMetrics.value.upload * 8) / (1024 * 1024)).toFixed(1))

const activeProfile = computed(() =>
        profiles.value.find(p => p.active) || {
          name: 'No Node Selected',
          protocol: 'None',
          flag: '🌍',
          dnsType: 'https',
          allowInsecure: false,
          useIpv6: false
        }
)

// --- ЛОГИКА ФИЛЬТРАЦИИ И СОРТИРОВКИ ---
const getFilteredAndSortedNodes = (nodes: Profile[]) => {
  if (!nodes) return[]
  let result = nodes
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = nodes.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.protocol.toLowerCase().includes(q) ||
        p.country.toLowerCase().includes(q)
    )
  }
  return result.sort((a, b) => {
    let modifier = sortOrder.value === 'asc' ? 1 : -1
    if (sortBy.value === 'name') {
      return a.name.localeCompare(b.name) * modifier
    } else if (sortBy.value === 'ping') {
      const getPingWeight = (p: number) => {
        if (p > 0) return p
        if (p === 0) return 999998
        if (p === -1) return 999999
        return 999999
      }
      return (getPingWeight(a.ping) - getPingWeight(b.ping)) * modifier
    }
    return 0
  })
}

const toggleSort = (field: 'name' | 'ping') => {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}

watch(activeTab, () => {
  searchQuery.value = ''
})

// --- АДАПТИВНОСТЬ ---
// На мобильных (<768px) прячем сайдбар, показываем BottomNav, выключаем 3D-сферу
// для экономии батареи и CPU.
const isMobile = ref(false)
let mqMobile: MediaQueryList | null = null
function applyMq(e: MediaQueryListEvent | MediaQueryList) {
  isMobile.value = e.matches
}

// --- INIT ---
onMounted(async () => {
  logger.info('App', 'Vue app mounting');

  // Регистрируем слушатель media-query для адаптивности.
  mqMobile = window.matchMedia('(max-width: 767px)')
  applyMq(mqMobile)
  mqMobile.addEventListener('change', applyMq)

  // Платформенная информация (apps placeholder, суффиксы и т.д.)
  try {
    const info = await invoke<PlatformInfo>('get_platform_info')
    platformInfo.value = info
    logger.info('App', `Platform: ${info.os}, apps_supported=${info.apps_supported}`)
  } catch (e) {
    logger.warn('App', 'get_platform_info failed, using defaults', e)
  }

  try {
    store = await load('store.json', {autoSave: false, defaults: {}})

    const savedSubs = await store.get('subscriptions')
    if (savedSubs) subscriptions.value = savedSubs as Subscription[]

    const savedStandalone = await store.get('standaloneProfiles')
    if (savedStandalone) standaloneProfiles.value = savedStandalone as Profile[]

    const oldProfiles = await store.get('profiles')
    if (oldProfiles && (!savedStandalone || (savedStandalone as Profile[]).length === 0)) {
      standaloneProfiles.value = oldProfiles as Profile[]
      await store.delete('profiles')
    }

    // МИГРАЦИЯ ДАННЫХ ДЛЯ DNS TYPE И ALLOW INSECURE
    standaloneProfiles.value = standaloneProfiles.value.map(p => ({
      ...p,
      dnsType: p.dnsType || 'https',
      allowInsecure: p.allowInsecure ?? false, // ДОБАВЛЕНО
      useIpv6: p.useIpv6 ?? false
    }))

    subscriptions.value.forEach(sub => {
      sub.nodes = sub.nodes.map(p => ({
        ...p,
        dnsType: p.dnsType || 'https',
        allowInsecure: p.allowInsecure ?? false, // ДОБАВЛЕНО
        useIpv6: p.useIpv6 ?? false
      }))
    })

    const savedRouting = await store.get('routing')
    if (savedRouting) routing.value = savedRouting as RoutingConfig

    const savedSettings = await store.get('settings')
    if (savedSettings) {
      settings.value = savedSettings as any
      // Убедимся, что geoipRules существует
      if (!settings.value.geoipRules) settings.value.geoipRules = {}
    }
    settings.value.autoStart = await isEnabled();
    await invoke('set_minimize_to_tray', {enable: settings.value.minimizeToTray});

    // Check Wizard Status
    if (!settings.value.setupWizardCompleted) {
      showWizard.value = true
    }

  } catch (e) {
    logger.error('App', 'Store init error', e)
  }

  if (profiles.value.length > 0) refreshPings()

  if ((window as any).__TAURI_INTERNALS__) {
    try {
      await listen<{ download: number, upload: number }>('vpn_metrics', e => {
        currentMetrics.value = e.payload
      })
      await listen<string>('vpn_log', e => addLog(`[CORE] ${e.payload}`))

      const maxAttempts = 60
      for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
          await invoke('start_metrics_stream')
          break
        } catch (_) {
          if (attempt < maxAttempts - 1) await new Promise(r => setTimeout(r, 2000))
        }
      }
    } catch (e) {
      console.error('Failed to init streams:', e)
    }
  }
})

// --- LOGIC ---
// Коалесинг записей в журнал: не дёргаем реактивность и DOM на каждую
// строку из батча, а один раз за кадр. Сильно снижает нагрузку, когда
// из Rust-стрима прилетает пакет в десятки строк разом.
let pendingLogs: string[] = []
let logFlushScheduled = false
const flushLogs = () => {
  logFlushScheduled = false
  if (pendingLogs.length === 0) return
  logs.value.push(...pendingLogs)
  pendingLogs = []
  if (logs.value.length > 100) logs.value.splice(0, logs.value.length - 100)
  if (logsContainer.value) logsContainer.value.scrollTop = logsContainer.value.scrollHeight
}
const addLog = (msg: string) => {
  pendingLogs.push(msg)
  if (!logFlushScheduled) {
    logFlushScheduled = true
    requestAnimationFrame(flushLogs)
  }
}

// Уведомляем Rust о том, открыта ли панель логов: пока закрыта —
// бэкенд не шлёт "vpn_log" события в webview (см. set_logs_streaming в main.rs).
watch(showLogs, async (open) => {
  try { await invoke('set_logs_streaming', { enable: open }) } catch {}
})
onMounted(async () => {
  // На случай если панель логов уже открыта до подключения IPC
  try { await invoke('set_logs_streaming', { enable: showLogs.value }) } catch {}
})

const toggleAutoStart = async () => {
  try {
    if (settings.value.autoStart) {
      await disable();
      settings.value.autoStart = false;
    } else {
      await enable();
      settings.value.autoStart = true;
    }
    await saveData();
  } catch (e) {
    console.error(e);
    alert('Failed to toggle autostart: ' + e);
  }
}

const toggleTray = async () => {
  settings.value.minimizeToTray = !settings.value.minimizeToTray;
  await invoke('set_minimize_to_tray', {enable: settings.value.minimizeToTray});
  await saveData();
}

const refreshPings = async () => {
  if (isPinging.value) return
  isPinging.value = true
  addLog('[SYSTEM] Started latency check...')

  profiles.value.forEach(p => p.ping = 0)

  const promises = profiles.value.map(async (p) => {
    try {
      const ms = await invoke<number>('check_ping', {link: p.rawLink})
      p.ping = ms
    } catch (e) {
      p.ping = -1
    }
  })

  await Promise.allSettled(promises)
  await saveData()
  isPinging.value = false
}

const toggleVpn = async () => {
  if (profiles.value.length === 0) {
    showToast('Add a node first')
    activeTab.value = 'nodes'
    return
  }
  const active = profiles.value.find(p => p.active)
  if (!active || !active.rawLink) return

  const targetState = !isConnected.value

  try {
    addLog(`[SYSTEM] Attempting to ${targetState ? 'connect' : 'disconnect'}...`)

    const res = await invoke<string>('toggle_vpn', {
      payload: {
        connect: targetState,
        link: active.rawLink,
        routing_config: {
          mode: routing.value.mode,
          domains: routing.value.domains,
          apps: routing.value.apps,
          enable_geoip: settings.value.enableGeoip,
          geoip_rules: settings.value.geoipRules,
          dns_type: active.dnsType || 'https',
          allow_insecure: active.allowInsecure ?? false, // ДОБАВЛЕНО
          use_ipv6: active.useIpv6 ?? false
        }
      }
    })

    isConnected.value = res === 'Connected'
    addLog(`[SYSTEM] ${res}`)
  } catch (e) {
    addLog(`[ERROR] ${e}`);
    isConnected.value = false;
  }
}

const saveData = async () => {
  if (store) {
    await store.set('subscriptions', subscriptions.value)
    await store.set('standaloneProfiles', standaloneProfiles.value)
    await store.set('routing', routing.value)
    await store.set('settings', settings.value)
    await store.save()
  }
}

// Управление модальным окном настройки профиля
const openEditModal = (profile: Profile) => {
  editingProfileId.value = profile.id
  editedDnsType.value = profile.dnsType || 'https'
  editedAllowInsecure.value = profile.allowInsecure ?? false // ДОБАВЛЕНО
  editedUseIpv6.value = profile.useIpv6 ?? false
  editModalOpen.value = true
}

const saveProfileSettings = async () => {
  if (editingProfileId.value === null) return

  const updateProfile = (nodes: Profile[]) => {
    const p = nodes.find(n => n.id === editingProfileId.value)
    if (p) {
      p.dnsType = editedDnsType.value
      p.allowInsecure = editedAllowInsecure.value // ДОБАВЛЕНО
      p.useIpv6 = editedUseIpv6.value
    }
  }

  updateProfile(standaloneProfiles.value)
  subscriptions.value.forEach(sub => updateProfile(sub.nodes))

  await saveData()
  editModalOpen.value = false
  editingProfileId.value = null
}

// Wizard Logic
const completeWizard = async () => {
  settings.value.setupWizardCompleted = true
  await saveData()
  showWizard.value = false
}

const handleWizardImport = async () => {
  await handleImport()
  if (profiles.value.length > 0) {
    wizardStep.value = 3
  }
}

const addDomain = () => {
  if (newDomainInput.value.trim()) {
    routing.value.domains.push(newDomainInput.value.trim());
    newDomainInput.value = '';
    saveData()
  }
}
const removeDomain = (idx: number) => {
  routing.value.domains.splice(idx, 1);
  saveData()
}
const addApp = () => {
  if (newAppInput.value.trim()) {
    let app = newAppInput.value.trim();
    // На Windows принято указывать имя с .exe (например, chrome.exe).
    // На других платформах (Linux: chrome, macOS: Google Chrome,
    // Android: com.android.chrome) .exe не используется — добавляем
    // суффикс только если платформа его требует.
    if (platformInfo.value.apps_append_suffix && !app.toLowerCase().endsWith('.exe')) {
      app += '.exe';
    }
    routing.value.apps.push(app);
    newAppInput.value = '';
    saveData()
  }
}
const removeApp = (idx: number) => {
  routing.value.apps.splice(idx, 1);
  saveData()
}

const setActiveProfile = async (id: number) => {
  subscriptions.value.forEach(s => s.nodes.forEach(p => p.active = (p.id === id)))
  standaloneProfiles.value.forEach(p => p.active = (p.id === id))
  await saveData()
}

const deleteProfile = async (id: number) => {
  const wasActive = profiles.value.find(p => p.id === id)?.active
  standaloneProfiles.value = standaloneProfiles.value.filter(p => p.id !== id)
  subscriptions.value.forEach(s => {
    s.nodes = s.nodes.filter(p => p.id !== id)
  })

  if (wasActive) {
    if (profiles.value.length > 0) profiles.value[0].active = true
    else isConnected.value = false
  }
  await saveData()
}

const deleteSubscription = async (id: string) => {
  const sub = subscriptions.value.find(s => s.id === id)
  if (sub && sub.nodes.some(n => n.active)) isConnected.value = false
  subscriptions.value = subscriptions.value.filter(s => s.id !== id)
  expandedSubs.value.delete(id)
  await saveData()
}

const parseLocationFromName = (name: string) => {
  const emojiRegex = /[\u{1F1E6}-\u{1F1FF}]{2}/u
  const match = name.match(emojiRegex)
  let country = 'Unknown'
  if (name.includes('RU')) country = 'Russia'
  else if (name.includes('DE')) country = 'Germany'
  else if (name.includes('NL')) country = 'Netherlands'
  else if (name.includes('US')) country = 'USA'
  else if (name.includes('PL')) country = 'Poland'
  return {flag: match ? match[0] : '🌍', country}
}

const createProfileObj = (url: string, subId?: string): Profile | null => {
  let protocol = 'Custom', name = 'Node';
  if (url.startsWith('vless://')) protocol = 'VLESS';
  else if (url.startsWith('wireguard://')) protocol = 'WireGuard';
  else if (url.startsWith('warp://')) protocol = 'WARP';
  else if (url.startsWith('ss://')) protocol = 'Shadowsocks';
  else if (url.startsWith('trojan://')) protocol = 'Trojan';
  else if (url.startsWith('hysteria2://') || url.startsWith('hy2://')) protocol = 'Hysteria2';

  const hashIndex = url.indexOf('#')
  if (hashIndex !== -1) {
    try {
      name = decodeURIComponent(url.substring(hashIndex + 1))
    } catch (e) {}
  }
  const {flag, country} = parseLocationFromName(name)

  return {
    id: Date.now() + Math.random(),
    name,
    protocol,
    ping: 0,
    active: false,
    rawLink: url,
    flag,
    country,
    subscriptionId: subId,
    dnsType: 'https',
    allowInsecure: false, // ДОБАВЛЕНО
    useIpv6: false
  }
}

const handleImport = async () => {
  if (!importUrl.value) return
  const input = importUrl.value.trim()
  importUrl.value = ''
  isImporting.value = true

  try {
    if (input.startsWith('http')) {
      addLog(`[SYSTEM] Fetching subscription...`)
      const res = await invoke<{nodes: string[], userinfo: string | null}>('fetch_subscription', {url: input})
      const subId = Date.now().toString()
      const newNodes = res.nodes.map(link => createProfileObj(link, subId)).filter(p => p !== null) as Profile[]

      let expireTime = undefined
      if (res.userinfo) {
        const match = res.userinfo.match(/expire=(\d+)/)
        if (match) expireTime = parseInt(match[1]) * 1000
      }

      if (profiles.value.length === 0 && newNodes.length > 0) newNodes[0].active = true

      subscriptions.value.push({
        id: subId, url: input, name: new URL(input).hostname, lastUpdated: Date.now(), expire: expireTime, nodes: newNodes
      })
      addLog(`[SYSTEM] Imported subscription with ${newNodes.length} nodes`)
    } else {
      const p = createProfileObj(input)
      if (p) {
        if (profiles.value.length === 0) p.active = true
        standaloneProfiles.value.push(p)
        addLog(`[SYSTEM] Imported standalone profile: ${p.name}`)
      }
    }
    await saveData()
    refreshPings()
  } catch (e) {
    addLog(`[ERROR] Import failed: ${e}`)
  } finally {
    isImporting.value = false
  }
}

const refreshSubscription = async (sub: Subscription) => {
  addLog(`[SYSTEM] Updating subscription ${sub.name}...`)
  try {
    const res = await invoke<{nodes: string[], userinfo: string | null}>('fetch_subscription', { url: sub.url })
    const activeLink = profiles.value.find(p => p.active)?.rawLink
    const newNodes = res.nodes.map(link => createProfileObj(link, sub.id)).filter(p => p !== null) as Profile[]

    if (activeLink) {
      const found = newNodes.find(n => n.rawLink === activeLink)
      if (found) found.active = true
    }

    sub.nodes = newNodes
    sub.lastUpdated = Date.now()
    if (res.userinfo) {
      const match = res.userinfo.match(/expire=(\d+)/)
      if (match) sub.expire = parseInt(match[1]) * 1000
    }

    await saveData()
    refreshPings()
    addLog(`[SYSTEM] Subscription updated. Total nodes: ${newNodes.length}`)
  } catch(e) {
    addLog(`[ERROR] Failed to update subscription: ${e}`)
  }
}

const toggleSubExpanded = (id: string) => {
  if (expandedSubs.value.has(id)) expandedSubs.value.delete(id)
  else expandedSubs.value.add(id)
  // trigger reactivity for Set
  expandedSubs.value = new Set(expandedSubs.value)
}

interface NodeGroup {
  id: string
  name: string
  sub: Subscription | null
  nodes: Profile[]
}
const nodeGroups = computed<NodeGroup[]>(() => {
  const groups: NodeGroup[] = []
  for (const sub of subscriptions.value) {
    groups.push({ id: sub.id, name: sub.name, sub, nodes: sub.nodes })
  }
  if (standaloneProfiles.value.length > 0) {
    groups.push({
      id: '__standalone__',
      name: 'Standalone Nodes',
      sub: null,
      nodes: standaloneProfiles.value
    })
  }
  return groups
})

const quickAdd = async () => {
  try {
    let text = ''
    try {
      text = await navigator.clipboard.readText()
    } catch {
      text = ''
    }
    const trimmed = (text || '').trim()
    if (!trimmed) {
      showToast('Clipboard is empty')
      return
    }
    if (trimmed.startsWith('http') || trimmed.startsWith('vless://') || trimmed.startsWith('vmess://')
        || trimmed.startsWith('trojan://') || trimmed.startsWith('ss://') || trimmed.startsWith('hy2://')
        || trimmed.startsWith('hysteria2://') || trimmed.startsWith('wireguard://') || trimmed.startsWith('warp://')) {
      importUrl.value = trimmed
      await handleImport()
      showToast('Imported from clipboard')
    } else {
      importUrl.value = trimmed
      showToast('Pasted to input — review and add')
    }
  } catch (e) {
    showToast('Quick add failed')
  }
}
</script>

<template>
  <div
      class="fixed inset-0 w-full h-full bg-[#121212] text-[#FDFDFD] overflow-hidden selection:bg-[#9D68FF]/30 font-sans flex">
    <!-- Background -->
    <div class="absolute inset-0 z-0 pointer-events-none">
      <div
          class="absolute inset-0 opacity-10 bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0MCIgaGVpZ2h0PSI0MCI+CjxyZWN0IHdpZHRoPSI0MCIgaGVpZ2h0PSI0MCIgZmlsbD0ibm9uZSIvPgo8Y2lyY2xlIGN4PSIyMCIgY3k9IjIwIiByPSIxIiBmaWxsPSJyZ2JhKDI1MywgMjUzLCAyNTMsIDAuMSkiLz4KPC9zdmc+')]"></div>
      <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_0%,#121212_90%)]"></div>
    </div>

      <!-- Pulsar Layer (3D Sphere) -->
    <div
        class="fixed inset-0 z-10 flex items-center justify-center pointer-events-none transition-all duration-700 ease-[cubic-bezier(0.16,1,0.3,1)]"
        :class="activeTab === 'dashboard' ? 'scale-100 opacity-100 blur-none' : 'scale-90 opacity-20 blur-md'"
    >
      <div class="absolute inset-0">
        <PulsarSphere :speed="currentMetrics.download + currentMetrics.upload" :is-connected="isConnected" :active="!isMobile && activeTab === 'dashboard'"/>
      </div>
      <!-- Большая кнопка подключения -->
      <button
          v-if="activeTab === 'dashboard'"
          @click="toggleVpn"
          class="connect-orb relative z-[100] w-[220px] h-[220px] md:w-[450px] md:h-[450px] rounded-full flex flex-col items-center justify-center group outline-none cursor-pointer pointer-events-auto active:scale-95 transition-transform"
      >
        <!-- Внутреннее свечение и кольцо (всегда видно на мобильном, появляется при hover на десктопе) -->
        <div
            class="orb-ring absolute inset-0 rounded-full pointer-events-none transition-opacity duration-500"
            :class="isConnected
              ? 'opacity-100 shadow-[0_0_60px_rgba(157,104,255,0.6),inset_0_0_40px_rgba(157,104,255,0.4)]'
              : 'opacity-70 md:opacity-0 md:group-hover:opacity-70 shadow-[0_0_40px_rgba(157,104,255,0.2),inset_0_0_30px_rgba(157,104,255,0.15)]'"
        ></div>
        <!-- Лейбл: виден всегда на мобильном, по hover на десктопе -->
        <div
            class="orb-label md:mt-[280px] mt-0 md:opacity-0 opacity-100 md:group-hover:opacity-100 md:group-hover:-translate-y-6 px-6 py-3 rounded-full bg-[#121212]/90 backdrop-blur-2xl border border-[#9D68FF]/20 shadow-[0_0_30px_rgba(0,0,0,0.5)] transition-all duration-500 ease-out flex items-center gap-3 pointer-events-none">
          <Power class="w-5 h-5 md:w-4 md:h-4 animate-pulse" :class="isConnected ? 'text-[#9D68FF]' : 'text-white'"/>
          <span class="text-xs md:text-[10px] font-black uppercase tracking-[0.4em]"
                :class="isConnected ? 'text-[#9D68FF]' : 'text-white'">
            {{ isConnected ? 'DISCONNECT' : 'INITIALIZE' }}
          </span>
        </div>
      </button>
    </div>

    <!-- MAIN UI WRAPPER -->
    <div class="relative z-20 w-full h-full flex p-6 gap-6 pointer-events-none main-padding">
      <!-- SIDEBAR MENU (desktop) -->
      <nav
          class="glass-panel sidebar w-24 rounded-[2.5rem] flex flex-col items-center py-8 justify-between flex-shrink-0 pointer-events-auto shadow-2xl z-30 relative">
        <div class="flex flex-col items-center gap-10 w-full">
          <div
              class="w-12 h-12 rounded-[1.2rem] bg-[#9D68FF] flex items-center justify-center shadow-[0_0_30px_rgba(157,104,255,0.4)]">
            <Zap class="w-6 h-6 text-[#121212]" fill="currentColor"/>
          </div>
          <div class="flex flex-col gap-4 w-full px-4 mt-4">
            <button @click="activeTab = 'dashboard'" class="nav-btn" :class="{ active: activeTab === 'dashboard' }">
              <ShieldCheck class="w-6 h-6 icon"/>
            </button>
            <button @click="activeTab = 'nodes'" class="nav-btn" :class="{ active: activeTab === 'nodes' }">
              <Globe2 class="w-6 h-6 icon"/>
            </button>
            <button @click="activeTab = 'settings'" class="nav-btn" :class="{ active: activeTab === 'settings' }">
              <Settings class="w-6 h-6 icon"/>
            </button>
          </div>
        </div>
        <button @click="showLogs = !showLogs"
                class="w-12 h-12 rounded-full flex items-center justify-center transition-all"
                :class="showLogs ? 'bg-[#FDFDFD]/10 text-[#FDFDFD]' : 'text-[#FDFDFD]/30 hover:text-[#FDFDFD] hover:bg-[#FDFDFD]/5'">
          <Terminal class="w-5 h-5"/>
        </button>
      </nav>

      <!-- BOTTOM NAV (mobile) -->
      <BottomNav
          v-if="isMobile"
          class="bottom-nav"
          :active-tab="activeTab"
          @navigate="(t: any) => activeTab = t"
          :show-logs="showLogs"
          @toggle-logs="showLogs = !showLogs"
      />

      <!-- CONTENT AREA -->
      <main class="flex-1 flex flex-col justify-between overflow-hidden relative pointer-events-none main-area">
        <header
            class="app-header glass-panel pointer-events-auto shrink-0 z-30">
          <!-- MOBILE HEADER (compact) -->
          <div class="md:hidden flex items-center justify-between h-14 px-3 gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <div class="w-8 h-8 rounded-xl bg-[#9D68FF] flex items-center justify-center shrink-0 shadow-[0_0_20px_rgba(157,104,255,0.4)]">
                <Zap class="w-4 h-4 text-[#121212]" fill="currentColor"/>
              </div>
              <div class="min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="text-base font-black tracking-tight">Pulsar</span>
                  <span class="text-base font-black text-[#9D68FF]">VPN</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full"
                   :class="isConnected ? 'bg-[#9D68FF]/15' : 'bg-[#FDFDFD]/5'">
                <div class="w-1.5 h-1.5 rounded-full"
                     :class="isConnected ? 'bg-[#9D68FF] shadow-[0_0_8px_#9D68FF]' : 'bg-[#FDFDFD]/30'"></div>
                <span class="text-[9px] font-black uppercase tracking-widest"
                      :class="isConnected ? 'text-[#9D68FF]' : 'text-[#FDFDFD]/40'">
                  {{ isConnected ? 'ON' : 'OFF' }}
                </span>
              </div>
              <button @click="quickAdd" class="quick-add-btn" aria-label="Quick add from clipboard" title="Paste from clipboard">
                <Plus class="w-5 h-5"/>
              </button>
            </div>
          </div>
          <!-- DESKTOP HEADER (full) -->
          <div class="hidden md:flex justify-between items-center rounded-[2rem] px-8 py-5">
            <div>
              <h1 class="text-3xl font-black tracking-tight flex items-center gap-2">Pulsar<span class="text-[#9D68FF]">VPN</span>
              </h1>
              <p class="text-[10px] text-[#FDFDFD]/50 font-bold uppercase tracking-[0.2em] mt-1 flex items-center gap-2">
                <Lock class="w-3 h-3" :class="isConnected ? 'text-[#9D68FF]' : 'text-[#FDFDFD]/30'"/>
                System {{ isConnected ? 'Encrypted' : 'Exposed' }}
              </p>
            </div>
            <div class="flex items-center gap-3">
              <button @click="quickAdd" class="quick-add-btn" aria-label="Quick add from clipboard" title="Paste from clipboard">
                <ClipboardPaste class="w-4 h-4"/>
                <span class="text-xs font-bold uppercase tracking-widest">Paste</span>
              </button>
              <div class="flex items-center gap-3 px-4 py-2 bg-[#121212]/50 rounded-xl border border-[#FDFDFD]/10">
                <div class="w-2 h-2 rounded-full"
                     :class="isConnected ? 'bg-[#9D68FF] shadow-[0_0_10px_#9D68FF]' : 'bg-[#FDFDFD]/30'"></div>
                <span class="text-xs font-bold tracking-widest uppercase"
                      :class="isConnected ? 'text-[#FDFDFD]' : 'text-[#FDFDFD]/50'">{{
                    isConnected ? 'Protected' : 'Unprotected'
                  }}</span>
              </div>
            </div>
          </div>
        </header>

        <!-- SUB-PAGES -->
        <div class="relative flex-1 overflow-hidden pointer-events-none mt-6">

          <!-- DASHBOARD -->
          <transition name="fade">
            <div v-if="activeTab === 'dashboard'"
                 class="absolute inset-0 flex flex-col justify-end pb-0 pointer-events-none">
              <!-- DESKTOP: 3 metric cards -->
              <div class="hidden md:grid grid-cols-3 gap-6 shrink-0 h-44 pointer-events-auto metrics-grid">
                <div class="glass-panel rounded-[2rem] p-6 flex flex-col justify-between shadow-lg relative overflow-hidden">
                  <div class="flex justify-between items-center relative z-10">
                    <span class="text-[10px] font-bold text-[#FDFDFD]/50 uppercase tracking-widest flex items-center gap-2">
                      <ArrowDownToLine class="w-4 h-4 text-[#9D68FF]"/> Download
                    </span>
                  </div>
                  <div class="flex items-baseline gap-2 relative z-10">
                    <span class="text-5xl font-black font-mono tracking-tighter">{{ isConnected ? downloadSpeed : '0.0' }}</span>
                    <span class="text-xs font-bold text-[#FDFDFD]/30 uppercase">Mbps</span>
                  </div>
                </div>
                <div class="glass-panel rounded-[2rem] p-6 flex flex-col justify-between shadow-lg relative overflow-hidden">
                  <div class="flex justify-between items-center relative z-10">
                    <span class="text-[10px] font-bold text-[#FDFDFD]/50 uppercase tracking-widest flex items-center gap-2">
                      <ArrowUpFromLine class="w-4 h-4 text-[#9D68FF]"/> Upload
                    </span>
                  </div>
                  <div class="flex items-baseline gap-2 relative z-10">
                    <span class="text-5xl font-black font-mono tracking-tighter">{{ isConnected ? uploadSpeed : '0.0' }}</span>
                    <span class="text-xs font-bold text-[#FDFDFD]/30 uppercase">Mbps</span>
                  </div>
                </div>
                <div @click="activeTab = 'nodes'"
                     class="glass-panel rounded-[2rem] p-6 flex flex-col justify-center gap-4 cursor-pointer hover:border-[#9D68FF]/50 transition-all shadow-lg bg-gradient-to-br from-white/[0.02] to-[#9D68FF]/[0.05]">
                  <span class="text-[10px] font-bold text-[#FDFDFD]/50 uppercase tracking-widest flex items-center gap-2">
                    <Server class="w-4 h-4 text-[#FDFDFD]/50"/> Active Tunnel
                  </span>
                  <div class="flex items-center gap-4">
                    <span class="text-4xl drop-shadow-md">{{ activeProfile.flag }}</span>
                    <div class="flex flex-col">
                      <span class="text-xl font-black uppercase tracking-tight truncate w-32">{{ activeProfile.name }}</span>
                      <span class="text-[10px] font-bold text-[#9D68FF] tracking-widest uppercase mt-1">{{ activeProfile.protocol }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <!-- MOBILE: single compact info row -->
              <div class="md:hidden flex items-center gap-3 px-1 pb-2 pointer-events-auto">
                <button @click="activeTab = 'nodes'" class="flex-1 glass-panel rounded-2xl px-4 py-3 flex items-center gap-3 min-w-0 active:scale-[0.98] transition-transform">
                  <span class="text-2xl drop-shadow-sm shrink-0">{{ activeProfile.flag }}</span>
                  <div class="flex-1 min-w-0 text-left">
                    <div class="text-[9px] font-black uppercase tracking-widest text-[#FDFDFD]/40">Active</div>
                    <div class="text-sm font-black truncate">{{ activeProfile.name }}</div>
                  </div>
                  <div v-if="isConnected" class="flex flex-col items-end shrink-0">
                    <span class="text-[8px] font-black uppercase tracking-widest text-[#FDFDFD]/40">↓</span>
                    <span class="text-sm font-black font-mono text-[#9D68FF]">{{ downloadSpeed }}</span>
                  </div>
                </button>
              </div>
            </div>
          </transition>

          <!-- NODES TAB (subscriptions + standalone, accordion) -->
          <transition name="fade">
            <div v-if="activeTab === 'nodes'"
                 class="absolute inset-0 pointer-events-auto flex flex-col gap-4 custom-scroll overflow-y-auto pb-4">
              <!-- Import Box -->
              <div class="glass-panel p-4 md:p-6 rounded-[1.5rem] md:rounded-[2rem] flex flex-col gap-3 shrink-0">
                <h2 class="text-base md:text-lg font-black uppercase tracking-widest flex items-center gap-2">
                  <Plus class="w-4 h-4 md:w-5 md:h-5 text-[#9D68FF]"/> Import
                </h2>
                <div class="flex gap-2 md:gap-3">
                  <div class="flex-1 relative">
                    <div class="absolute inset-y-0 left-3 md:left-4 flex items-center pointer-events-none">
                      <Link class="w-4 h-4 md:w-5 md:h-5 text-[#FDFDFD]/30"/>
                    </div>
                    <input v-model="importUrl" @keyup.enter="handleImport" type="text"
                           placeholder="Subscription link, vless://, wireguard://..."
                           class="w-full h-11 md:h-14 bg-[#121212]/50 border border-[#FDFDFD]/10 rounded-xl md:rounded-2xl pl-10 md:pl-12 pr-3 md:pr-4 text-xs md:text-sm font-mono focus:outline-none focus:border-[#9D68FF]/50 focus:ring-1 focus:ring-[#9D68FF]/50 transition-all"/>
                  </div>
                  <button @click="handleImport" :disabled="isImporting"
                          class="h-11 md:h-14 px-4 md:px-8 bg-[#9D68FF] text-[#121212] font-black uppercase tracking-widest rounded-xl md:rounded-2xl hover:bg-[#b084ff] hover:shadow-[0_0_20px_rgba(157,104,255,0.4)] transition-all flex items-center justify-center min-w-[80px] md:min-w-[140px] text-xs md:text-sm">
                    <Loader2 v-if="isImporting" class="w-4 h-4 md:w-5 md:h-5 animate-spin"/>
                    <span v-else>ADD</span>
                  </button>
                </div>
              </div>

              <!-- Search + sort + ping -->
              <div class="flex gap-2 md:gap-3 px-1 shrink-0">
                <div class="relative flex-1">
                  <div class="absolute inset-y-0 left-3 flex items-center pointer-events-none">
                    <Search class="w-4 h-4 text-[#FDFDFD]/30"/>
                  </div>
                  <input v-model="searchQuery" type="text" placeholder="Search nodes..."
                         class="w-full h-10 bg-[#121212]/30 border border-[#FDFDFD]/10 rounded-xl pl-9 pr-4 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50 transition-all"/>
                </div>
                <div class="flex gap-1 bg-[#121212]/30 p-1 rounded-xl border border-[#FDFDFD]/10">
                  <button @click="toggleSort('name')" class="px-2.5 h-8 rounded-lg text-[10px] font-bold uppercase tracking-wider flex items-center gap-1 transition-all"
                          :class="sortBy === 'name' ? 'bg-[#9D68FF]/20 text-[#9D68FF]' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    <span class="hidden sm:inline">Name</span>
                    <span class="sm:hidden">A↓</span>
                    <component :is="sortOrder === 'asc' ? ArrowDownWideNarrow : ArrowUpNarrowWide" v-if="sortBy === 'name'" class="w-3 h-3"/>
                  </button>
                  <button @click="toggleSort('ping')" class="px-2.5 h-8 rounded-lg text-[10px] font-bold uppercase tracking-wider flex items-center gap-1 transition-all"
                          :class="sortBy === 'ping' ? 'bg-[#9D68FF]/20 text-[#9D68FF]' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    <span class="hidden sm:inline">Ping</span>
                    <span class="sm:hidden">ms</span>
                    <component :is="sortOrder === 'asc' ? ArrowDownWideNarrow : ArrowUpNarrowWide" v-if="sortBy === 'ping'" class="w-3 h-3"/>
                  </button>
                </div>
                <button @click="refreshPings" :disabled="isPinging" aria-label="Ping all"
                        class="w-10 h-10 rounded-xl bg-[#121212]/30 border border-[#FDFDFD]/10 text-[#9D68FF] hover:bg-[#9D68FF]/10 flex items-center justify-center transition-all disabled:opacity-50 shrink-0">
                  <RefreshCcw class="w-4 h-4" :class="isPinging ? 'animate-spin' : ''"/>
                </button>
              </div>

              <!-- Empty state -->
              <div v-if="nodeGroups.length === 0" class="glass-panel rounded-2xl p-8 text-center">
                <Globe2 class="w-12 h-12 mx-auto mb-3 text-[#FDFDFD]/20"/>
                <p class="text-sm text-[#FDFDFD]/50 mb-1">No nodes yet</p>
                <p class="text-xs text-[#FDFDFD]/30">Paste a subscription link or single node URL above to get started.</p>
              </div>

              <!-- Accordion groups -->
              <div class="flex flex-col gap-3">
                <div v-for="group in nodeGroups" :key="group.id" class="glass-panel rounded-2xl overflow-hidden">
                  <!-- Group header (clickable to expand) -->
                  <button @click="toggleSubExpanded(group.id)"
                          class="w-full p-3.5 md:p-4 flex items-center gap-3 text-left hover:bg-[#FDFDFD]/[0.02] transition-colors">
                    <component :is="expandedSubs.has(group.id) ? ChevronDown : ChevronRight"
                               class="w-4 h-4 text-[#FDFDFD]/40 shrink-0 transition-transform"/>
                    <div class="flex-1 min-w-0">
                      <h3 class="font-black text-sm truncate flex items-center gap-2">
                        <span class="truncate">{{ group.name }}</span>
                        <span class="text-[9px] font-mono font-black text-[#9D68FF] bg-[#9D68FF]/10 px-1.5 py-0.5 rounded shrink-0">
                          {{ group.nodes.length }}
                        </span>
                      </h3>
                      <p class="text-[10px] text-[#FDFDFD]/40 mt-0.5 truncate">
                        <span v-if="group.sub">
                          {{ new Date(group.sub.lastUpdated).toLocaleDateString() }}
                          <span v-if="group.sub.expire" :class="group.sub.expire < Date.now() ? 'text-red-400' : 'text-green-400'">
                            · exp {{ new Date(group.sub.expire).toLocaleDateString() }}
                          </span>
                        </span>
                        <span v-else>Manually added</span>
                      </p>
                    </div>
                    <!-- Refresh (only for subscription groups) -->
                    <button v-if="group.sub" @click.stop="refreshSubscription(group.sub)"
                            class="p-2 bg-[#9D68FF]/10 text-[#9D68FF] rounded-xl hover:bg-[#9D68FF] hover:text-[#121212] transition-colors shrink-0"
                            aria-label="Refresh subscription">
                      <RefreshCcw class="w-3.5 h-3.5"/>
                    </button>
                    <button v-if="group.sub" @click.stop="deleteSubscription(group.id)"
                            class="p-2 text-[#FDFDFD]/30 hover:text-red-400 hover:bg-red-400/10 rounded-xl transition-all shrink-0 hidden sm:flex"
                            aria-label="Delete subscription">
                      <Trash2 class="w-3.5 h-3.5"/>
                    </button>
                  </button>

                  <!-- Group content (expanded) -->
                  <transition name="slide-down">
                    <div v-if="expandedSubs.has(group.id)" class="border-t border-[#FDFDFD]/5">
                      <div v-if="group.nodes.length === 0" class="p-4 text-center text-xs text-[#FDFDFD]/30">
                        No nodes
                      </div>
                      <div v-else class="flex flex-col">
                        <div v-for="profile in getFilteredAndSortedNodes(group.nodes)" :key="profile.id"
                             @click="setActiveProfile(profile.id)"
                             class="node-row p-3 md:p-3.5 flex items-center gap-3 cursor-pointer border-t border-[#FDFDFD]/[0.03] first:border-t-0 transition-colors"
                             :class="profile.active ? 'bg-[#9D68FF]/10' : 'hover:bg-[#FDFDFD]/[0.02]'">
                          <div class="w-8 h-8 shrink-0 rounded-lg flex items-center justify-center transition-colors"
                               :class="profile.active ? 'bg-[#9D68FF] text-[#121212]' : 'bg-[#121212] text-[#FDFDFD]/40'">
                            <CheckCircle2 v-if="profile.active" class="w-4 h-4"/>
                            <span v-else class="text-base leading-none">{{ profile.flag }}</span>
                          </div>
                          <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-1.5">
                              <h4 class="font-bold text-xs md:text-sm truncate">{{ profile.name }}</h4>
                            </div>
                            <div class="flex items-center gap-1.5 mt-0.5">
                              <span class="text-[9px] font-black uppercase tracking-widest text-[#FDFDFD]/40">
                                {{ profile.protocol }}
                              </span>
                              <span v-if="profile.dnsType && profile.dnsType !== 'https'"
                                    class="text-[8px] font-black uppercase tracking-widest px-1 py-0.5 rounded border border-[#FDFDFD]/10 text-[#FDFDFD]/40">
                                {{ profile.dnsType }}
                              </span>
                            </div>
                          </div>
                          <div class="flex items-center gap-2 shrink-0">
                            <span class="text-xs font-mono font-bold w-12 text-right"
                                  :class="profile.ping > 0 && profile.ping < 150 ? 'text-green-400' : (profile.ping > 0 && profile.ping < 300 ? 'text-yellow-400' : (profile.ping === 0 ? 'text-[#FDFDFD]/30' : 'text-red-400'))">
                              <template v-if="profile.ping > 0">{{ profile.ping }}<span class="text-[9px] opacity-60">ms</span></template>
                              <Loader2 v-else-if="profile.ping === 0" class="w-3 h-3 inline animate-spin"/>
                              <span v-else>×</span>
                            </span>
                            <button @click.stop="openEditModal(profile)" aria-label="Edit node"
                                    class="p-1.5 text-[#FDFDFD]/30 hover:text-[#9D68FF] hover:bg-[#9D68FF]/10 rounded-lg transition-all">
                              <Settings2 class="w-3.5 h-3.5"/>
                            </button>
                            <button v-if="!group.sub" @click.stop="deleteProfile(profile.id)" aria-label="Delete node"
                                    class="p-1.5 text-[#FDFDFD]/30 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all">
                              <Trash2 class="w-3.5 h-3.5"/>
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </transition>
                </div>
              </div>
            </div>
          </transition>

          <!-- SETTINGS -->
          <transition name="fade">
            <div v-if="activeTab === 'settings'"
                 class="absolute inset-0 pointer-events-auto overflow-y-auto custom-scroll pr-2 flex flex-col gap-6 pb-6">

              <!-- Security -->
              <div class="glass-panel p-6 md:p-8 rounded-[1.5rem] md:rounded-[2rem] flex flex-col gap-3">
                <h2 class="text-base md:text-lg font-black uppercase tracking-widest flex items-center gap-2 border-b border-[#FDFDFD]/10 pb-3">
                  <ShieldAlert class="w-5 h-5 text-[#9D68FF]"/> Security
                </h2>
                <div class="flex flex-col gap-3 pt-1">
                  <div class="flex items-start gap-3">
                    <CheckCircle2 class="w-4 h-4 text-green-400 mt-0.5 shrink-0"/>
                    <div>
                      <h4 class="font-bold text-sm">DNS Hijack</h4>
                      <p class="text-xs text-[#FDFDFD]/50 mt-0.5">Always on. All port-53 traffic is routed through sing-box (rule: <code class="text-[10px] bg-[#FDFDFD]/5 px-1 py-0.5 rounded">protocol: dns → hijack-dns</code>). DoH/DoT to custom servers is subject to normal routing rules below.</p>
                    </div>
                  </div>
                  <div class="flex items-start gap-3 opacity-60">
                    <Lock class="w-4 h-4 text-[#FDFDFD]/40 mt-0.5 shrink-0"/>
                    <div>
                      <h4 class="font-bold text-sm">Kill Switch <span class="text-[9px] font-black uppercase tracking-widest text-[#FDFDFD]/40 ml-1">Planned</span></h4>
                      <p class="text-xs text-[#FDFDFD]/50 mt-0.5">Drop traffic if VPN disconnects. Will be implemented via <code class="text-[10px] bg-[#FDFDFD]/5 px-1 py-0.5 rounded">netsh advfirewall</code> on Windows / iptables on Android. Tracked separately.</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Split Tunneling -->
              <div class="glass-panel p-8 rounded-[2rem] flex flex-col gap-6">
                <div class="flex items-center justify-between border-b border-[#FDFDFD]/10 pb-4">
                  <h2 class="text-lg font-black uppercase tracking-widest flex items-center gap-2">
                    <ArrowDownToLine class="w-5 h-5 text-[#9D68FF]"/> Split Tunneling
                  </h2>
                </div>
                <div class="flex p-1 bg-[#121212]/50 rounded-xl border border-[#FDFDFD]/5">
                  <button @click="routing.mode = 'global'; saveData()"
                          class="flex-1 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all"
                          :class="routing.mode === 'global' ? 'bg-[#9D68FF] text-[#121212] shadow-md' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    Global
                  </button>
                  <button @click="routing.mode = 'bypass'; saveData()"
                          class="flex-1 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all"
                          :class="routing.mode === 'bypass' ? 'bg-[#9D68FF] text-[#121212] shadow-md' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    Bypass Mode
                  </button>
                  <button @click="routing.mode = 'proxy'; saveData()"
                          class="flex-1 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all"
                          :class="routing.mode === 'proxy' ? 'bg-[#9D68FF] text-[#121212] shadow-md' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    Only VPN
                  </button>
                </div>
                <div v-if="routing.mode !== 'global'" class="flex flex-col gap-6 mt-2">
                  <div class="flex flex-col gap-3">
                    <label class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50 flex items-center gap-2">
                      <Globe class="w-4 h-4"/> Domains
                      <span class="text-[9px] font-mono normal-case tracking-normal text-[#FDFDFD]/30">(suffix match — google.com covers mail.google.com)</span>
                    </label>
                    <div class="flex gap-2">
                      <input v-model="newDomainInput" @keyup.enter="addDomain" type="text" placeholder="e.g. google.com"
                             class="flex-1 h-10 bg-[#121212]/30 border border-[#FDFDFD]/10 rounded-xl px-4 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50 transition-all"/>
                      <button @click="addDomain" class="w-10 h-10 rounded-xl bg-[#9D68FF]/10 text-[#9D68FF] hover:bg-[#9D68FF] hover:text-[#121212] flex items-center justify-center transition-all">
                        <Plus class="w-5 h-5"/>
                      </button>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <span v-for="(domain, idx) in routing.domains" :key="idx"
                            class="px-3 py-1.5 rounded-lg bg-[#FDFDFD]/5 border border-[#FDFDFD]/5 text-xs font-mono flex items-center gap-2 group hover:border-red-500/50 transition-colors cursor-default">
                        {{ domain }}<button @click="removeDomain(idx)" class="text-[#FDFDFD]/30 group-hover:text-red-400"><X class="w-3 h-3"/></button>
                      </span>
                    </div>
                  </div>
                  <!-- Per-app routing: показываем только там, где это работает. -->
                  <div v-if="platformInfo.apps_supported" class="flex flex-col gap-3">
                    <label class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50 flex items-center gap-2">
                      <Monitor class="w-4 h-4"/> {{ platformInfo.apps_label }}
                    </label>
                    <div class="flex gap-2">
                      <input v-model="newAppInput" @keyup.enter="addApp" type="text" :placeholder="platformInfo.apps_placeholder"
                             class="flex-1 h-10 bg-[#121212]/30 border border-[#FDFDFD]/10 rounded-xl px-4 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50 transition-all"/>
                      <button @click="addApp" class="w-10 h-10 rounded-xl bg-[#9D68FF]/10 text-[#9D68FF] hover:bg-[#9D68FF] hover:text-[#121212] flex items-center justify-center transition-all">
                        <Plus class="w-5 h-5"/>
                      </button>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <span v-for="(app, idx) in routing.apps" :key="idx"
                            class="px-3 py-1.5 rounded-lg bg-[#FDFDFD]/5 border border-[#FDFDFD]/5 text-xs font-mono flex items-center gap-2 group hover:border-red-500/50 transition-colors cursor-default">
                        {{ app }}<button @click="removeApp(idx)" class="text-[#FDFDFD]/30 group-hover:text-red-400"><X class="w-3 h-3"/></button>
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- App Behavior -->
              <div class="glass-panel p-8 rounded-[2rem] flex flex-col gap-6">
                <h2 class="text-lg font-black uppercase tracking-widest flex items-center gap-2 border-b border-[#FDFDFD]/10 pb-4">
                  <MonitorCheck class="w-5 h-5 text-[#9D68FF]"/> App Behavior
                </h2>
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm">Auto-start with System</h4>
                    <p class="text-xs text-[#FDFDFD]/50 mt-1">Launch PulsarVPN silently.</p>
                  </div>
                  <button @click="toggleAutoStart" class="w-14 h-8 rounded-full transition-colors relative"
                          :class="settings.autoStart ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
                    <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                         :class="settings.autoStart ? 'translate-x-6' : 'translate-x-0'"></div>
                  </button>
                </div>
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm">Minimize to Tray on Close</h4>
                    <p class="text-xs text-[#FDFDFD]/50 mt-1">Keep VPN active in background.</p>
                  </div>
                  <button @click="toggleTray" class="w-14 h-8 rounded-full transition-colors relative"
                          :class="settings.minimizeToTray ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
                    <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                         :class="settings.minimizeToTray ? 'translate-x-6' : 'translate-x-0'"></div>
                  </button>
                </div>
              </div>

              <!-- GeoIP Routing -->
              <div class="glass-panel p-8 rounded-[2rem] flex flex-col gap-6">
                <div class="flex items-center justify-between border-b border-[#FDFDFD]/10 pb-4">
                  <h2 class="text-lg font-black uppercase tracking-widest flex items-center gap-2">
                    <Globe2 class="w-5 h-5 text-[#9D68FF]"/> GeoIP Routing
                  </h2>
                </div>
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm">Enable GeoIP</h4>
                    <p class="text-xs text-[#FDFDFD]/50 mt-1">Route traffic based on country IP.</p>
                  </div>
                  <button @click="settings.enableGeoip = !settings.enableGeoip; saveData()"
                          class="w-14 h-8 rounded-full transition-colors relative"
                          :class="settings.enableGeoip ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
                    <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                         :class="settings.enableGeoip ? 'translate-x-6' : 'translate-x-0'"></div>
                  </button>
                </div>
                <div v-if="settings.enableGeoip" class="flex flex-col gap-4 mt-2">
                  <div class="flex justify-between items-center">
                    <span class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50">Country rules</span>
                  </div>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-60 overflow-y-auto custom-scroll pr-2">
                    <div v-for="country in popularCountries" :key="country.code" class="flex items-center justify-between bg-[#121212]/30 p-2 rounded-lg border border-[#FDFDFD]/5">
                      <span class="text-sm flex items-center gap-2"><span class="text-lg">{{ country.flag }}</span> {{ country.name }}</span>
                      <select v-model="settings.geoipRules[country.code]" @change="saveData" class="bg-[#121212] border border-[#FDFDFD]/10 rounded-lg px-2 py-1 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50">
                        <option value="proxy">Proxy</option>
                        <option value="direct">Direct</option>
                        <option value="block">Block</option>
                      </select>
                    </div>
                  </div>
                  <p class="text-[10px] text-[#FDFDFD]/30 mt-1">Rules are applied after local exceptions. Block will drop traffic.</p>
                </div>
              </div>

            </div>
          </transition>
        </div>
      </main>
    </div>

    <!-- LOGS CONSOLE -->
    <transition name="slide-up">
      <div v-if="showLogs"
           class="absolute bottom-0 left-0 right-0 h-1/3 bg-[#0a0a0a]/95 backdrop-blur-3xl border-t border-[#9D68FF]/20 z-50 flex flex-col shadow-[0_-20px_50px_rgba(0,0,0,0.5)] pointer-events-auto">
        <div class="flex justify-between items-center p-3 border-b border-[#FDFDFD]/10 bg-black/40">
          <span class="text-xs font-black uppercase tracking-widest flex items-center gap-2 text-[#9D68FF]"><Terminal class="w-4 h-4"/> System Logs</span>
          <div class="flex gap-2">
            <button @click="logs =[]" class="p-1.5 rounded-lg text-[#FDFDFD]/30 hover:text-red-400 hover:bg-red-400/10 transition-colors">
              <Trash class="w-4 h-4"/>
            </button>
            <button @click="showLogs = false" class="p-1.5 rounded-lg text-[#FDFDFD]/30 hover:text-white hover:bg-white/10 transition-colors">
              <X class="w-4 h-4"/>
            </button>
          </div>
        </div>
        <div ref="logsContainer" class="flex-1 p-4 overflow-y-auto custom-scroll font-mono text-[10px] leading-relaxed">
          <div v-for="(log, i) in logs" :key="i" class="mb-1" :class="log.includes('ERROR') ? 'text-red-400' : 'text-[#A78BFA]'">{{ log }}</div>
        </div>
      </div>
    </transition>

    <!-- WIZARD OVERLAY -->
    <transition name="fade">
      <div v-if="showWizard" class="fixed inset-0 z-50 bg-[#121212] flex items-center justify-center p-4 sm:p-6 md:p-10 pointer-events-auto">
        <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(157,104,255,0.1)_0%,transparent_70%)]"></div>
        <div class="glass-panel w-full max-w-2xl min-h-[400px] rounded-[3rem] relative overflow-hidden flex flex-col mx-auto">

          <!-- STEP 1: WELCOME -->
          <div v-if="wizardStep === 1" class="flex-1 flex flex-col items-center justify-center gap-6 p-8 sm:p-10 md:p-12 text-center">
            <div class="w-20 h-20 sm:w-24 sm:h-24 rounded-[2rem] bg-[#9D68FF] flex items-center justify-center shadow-[0_0_50px_rgba(157,104,255,0.4)] animate-bounce">
              <Sparkles class="w-10 h-10 sm:w-12 sm:h-12 text-[#121212]" fill="currentColor"/>
            </div>
            <div>
              <h1 class="text-2xl sm:text-3xl md:text-4xl font-black mb-3 sm:mb-4">Welcome to Pulsar<span class="text-[#9D68FF]">VPN</span></h1>
              <p class="text-[#FDFDFD]/50 max-w-md mx-auto leading-relaxed text-sm sm:text-base">
                The next generation tunnel client. Secure, fast, and designed for privacy. Let's get you set up in seconds.
              </p>
            </div>
            <button @click="wizardStep = 2" class="mt-2 sm:mt-4 px-6 sm:px-8 py-3 sm:py-4 bg-[#FDFDFD] text-[#121212] font-black uppercase tracking-widest rounded-2xl hover:bg-[#9D68FF] hover:shadow-[0_0_30px_rgba(157,104,255,0.5)] transition-all flex items-center gap-3">
              Get Started <ArrowRight class="w-4 h-4 sm:w-5 sm:h-5"/>
            </button>
          </div>

          <!-- STEP 2: IMPORT -->
          <div v-if="wizardStep === 2" class="flex-1 flex flex-col items-center justify-center gap-6 p-8 sm:p-10 md:p-12">
            <div class="text-center">
              <h2 class="text-xl sm:text-2xl font-black mb-2">Add Your First Subscription</h2>
              <p class="text-[#FDFDFD]/50 text-sm sm:text-base">Paste your VLESS, Trojan, or Subscription link below.</p>
            </div>
            <div class="w-full max-w-md flex flex-col sm:flex-row gap-4">
              <input v-model="importUrl" @keyup.enter="handleWizardImport" type="text"
                     placeholder="Paste link here..."
                     class="flex-1 h-12 sm:h-14 bg-[#121212]/50 border border-[#FDFDFD]/10 rounded-2xl px-6 text-sm font-mono focus:outline-none focus:border-[#9D68FF]/50 focus:ring-1 focus:ring-[#9D68FF]/50 transition-all"/>
              <button @click="handleWizardImport" :disabled="isImporting"
                      class="h-12 sm:h-14 px-6 sm:px-8 bg-[#9D68FF] text-[#121212] font-black rounded-2xl hover:bg-[#b084ff] transition-all flex items-center justify-center">
                <Loader2 v-if="isImporting" class="w-5 h-5 animate-spin"/>
                <span v-else>IMPORT</span>
              </button>
            </div>
            <button @click="wizardStep = 3" class="text-xs font-bold text-[#FDFDFD]/30 hover:text-[#FDFDFD] uppercase tracking-widest transition-colors">
              Skip for now
            </button>
          </div>

          <!-- STEP 3: READY -->
          <div v-if="wizardStep === 3" class="flex-1 flex flex-col items-center justify-center gap-6 p-8 sm:p-10 md:p-12 text-center">
            <div class="w-16 h-16 sm:w-20 sm:h-20 rounded-full bg-green-500/20 text-green-400 flex items-center justify-center mb-2">
              <Check class="w-8 h-8 sm:w-10 sm:h-10"/>
            </div>
            <div>
              <h2 class="text-2xl sm:text-3xl font-black mb-3 sm:mb-4">You are all set!</h2>
              <p class="text-[#FDFDFD]/50 text-sm sm:text-base">PulsarVPN is ready to connect. Enjoy unrestricted access.</p>
            </div>
            <button @click="completeWizard" class="mt-2 sm:mt-4 px-8 sm:px-10 py-3 sm:py-4 bg-[#9D68FF] text-[#121212] font-black uppercase tracking-widest rounded-2xl hover:shadow-[0_0_30px_rgba(157,104,255,0.4)] transition-all">
              Go to Dashboard
            </button>
          </div>

          <!-- Progress Dots -->
          <div class="absolute bottom-6 left-0 right-0 flex justify-center gap-2">
            <div v-for="step in 3" :key="step"
                 class="w-2 h-2 rounded-full transition-all duration-300"
                 :class="wizardStep === step ? 'w-6 sm:w-8 bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'"></div>
          </div>

        </div>
      </div>
    </transition>

    <!-- EDIT PROFILE MODAL -->
    <transition name="fade">
      <div v-if="editModalOpen" class="fixed inset-0 z-[60] flex items-center justify-center p-4 pointer-events-auto bg-black/60 backdrop-blur-sm">
        <div class="glass-panel w-full max-w-sm rounded-[2rem] p-6 flex flex-col gap-6 shadow-2xl border border-[#9D68FF]/20 relative overflow-hidden">
          <!-- Декор -->
          <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-[#9D68FF] to-transparent opacity-50"></div>

          <h3 class="text-lg font-black uppercase tracking-widest flex items-center gap-2">
            <Settings2 class="w-5 h-5 text-[#9D68FF]"/> Node Settings
          </h3>

          <!-- DNS TYPE -->
          <div class="flex flex-col gap-3">
            <label class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50">DNS Resolution Type</label>
            <div class="relative">
              <select v-model="editedDnsType" class="w-full h-12 bg-[#121212]/80 border border-[#FDFDFD]/10 rounded-xl px-4 text-sm focus:outline-none focus:border-[#9D68FF]/50 transition-all appearance-none cursor-pointer">
                <option value="https">HTTPS (DoH) - Encrypted (Default)</option>
                <option value="http">HTTP (Plain) - Not Encrypted</option>
                <option value="udp">UDP - Traditional (Port 53)</option>
              </select>
              <!-- Custom arrow -->
              <div class="absolute inset-y-0 right-4 flex items-center pointer-events-none text-[#FDFDFD]/30">
                <ArrowDownToLine class="w-4 h-4"/>
              </div>
            </div>
            <p class="text-[10px] text-[#FDFDFD]/30 mt-1 leading-relaxed">
              Change this if the server fails to connect due to DNS resolution errors (e.g., standard Hysteria2 instances).
            </p>
          </div>

          <!-- ALLOW INSECURE TOGGLE (ДОБАВЛЕНО) -->
          <div class="flex items-center justify-between mt-4 pt-4 border-t border-[#FDFDFD]/10">
            <div>
              <h4 class="font-bold text-sm">Allow Insecure</h4>
              <p class="text-xs text-[#FDFDFD]/50">Skip TLS certificate verification (for self‑signed certs).</p>
            </div>
            <button @click="editedAllowInsecure = !editedAllowInsecure"
                    class="w-14 h-8 rounded-full transition-colors relative"
                    :class="editedAllowInsecure ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
              <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                   :class="editedAllowInsecure ? 'translate-x-6' : 'translate-x-0'"></div>
            </button>
          </div>

          <div class="flex items-center justify-between mt-4 pt-2 border-t border-[#FDFDFD]/10">
            <div>
              <h4 class="font-bold text-sm">Use IPv6</h4>
              <p class="text-xs text-[#FDFDFD]/50">Off: IPv4-only (TUN, DNS, server dial). On: allow IPv6 if the network supports it.</p>
            </div>
            <button @click="editedUseIpv6 = !editedUseIpv6"
                    class="w-14 h-8 rounded-full transition-colors relative"
                    :class="editedUseIpv6 ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
              <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                   :class="editedUseIpv6 ? 'translate-x-6' : 'translate-x-0'"></div>
            </button>
          </div>

          <div class="flex gap-3 mt-2">
            <button @click="editModalOpen = false" class="flex-1 h-10 rounded-xl bg-[#FDFDFD]/5 hover:bg-[#FDFDFD]/10 text-xs font-bold uppercase tracking-widest transition-colors">
              Cancel
            </button>
            <button @click="saveProfileSettings" class="flex-1 h-10 rounded-xl bg-[#9D68FF] hover:bg-[#b084ff] text-[#121212] text-xs font-black uppercase tracking-widest transition-all shadow-[0_0_15px_rgba(157,104,255,0.3)]">
              Save
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- TOAST -->
    <transition name="toast">
      <div v-if="toast" class="toast">{{ toast }}</div>
    </transition>

  </div>
</template>

<style scoped>
.glass-panel {
  background: rgba(18, 18, 18, 0.4);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(253, 253, 253, 0.05);
  box-shadow: 0 24px 48px -12px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(253, 253, 253, 0.05);
}
.nav-btn {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 1.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  color: rgba(253, 253, 253, 0.3);
}
.nav-btn:hover {
  background: rgba(253, 253, 253, 0.08);
  color: #fdfdfd;
}
.nav-btn.active {
  background: rgba(157, 104, 255, 0.15);
  color: #9d68ff;
  box-shadow: inset 0 0 0 1px rgba(157, 104, 255, 0.3);
}
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.4s ease, transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}
.slide-up-enter-active, .slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.slide-up-enter-from, .slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}
.custom-scroll::-webkit-scrollbar {
  width: 4px;
}
.custom-scroll::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(253, 253, 253, 0.1);
  border-radius: 10px;
}
.custom-scroll::-webkit-scrollbar-thumb:hover {
  background: #9d68ff;
}

/* --- АДАПТИВНОСТЬ (mobile < 768px) --- */
@media (max-width: 767px) {
  .sidebar {
    display: none !important;
  }
  .bottom-nav {
    display: flex !important;
  }
  .main-padding {
    padding: 0.75rem !important;
    gap: 0.75rem !important;
  }
  /* Уменьшаем лог-панель на мобильном */
  .logs-panel {
    width: 100% !important;
  }
  /* Main area: оставить место под BottomNav + safe-area-inset-bottom */
  .main-area {
    padding-bottom: calc(64px + env(safe-area-inset-bottom, 0px) + 0.5rem) !important;
  }
}
/* Desktop: скрыть BottomNav */
@media (min-width: 768px) {
  .bottom-nav {
    display: none !important;
  }
  .app-header {
    margin: 0 0 1.5rem 0;
  }
}

/* --- APP HEADER --- */
.app-header {
  display: block;
  border-radius: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  margin: 0 0 0.75rem 0;
}
@media (min-width: 768px) {
  .app-header {
    border-radius: 2rem;
    background: rgba(18, 18, 18, 0.4);
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    border: 1px solid rgba(253, 253, 253, 0.05);
    box-shadow: 0 24px 48px -12px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(253, 253, 253, 0.05);
  }
}
.quick-add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  height: 36px;
  min-width: 36px;
  padding: 0 0.6rem;
  border-radius: 12px;
  background: rgba(157, 104, 255, 0.15);
  color: #9d68ff;
  border: 1px solid rgba(157, 104, 255, 0.2);
  transition: all 0.2s;
  cursor: pointer;
  font: inherit;
}
.quick-add-btn:hover {
  background: rgba(157, 104, 255, 0.25);
  color: #c0a0ff;
}
.quick-add-btn:active {
  transform: scale(0.95);
}

/* --- TOAST --- */
.toast {
  position: fixed;
  left: 50%;
  bottom: calc(80px + env(safe-area-inset-bottom, 0px));
  transform: translateX(-50%);
  z-index: 100;
  padding: 0.65rem 1.25rem;
  background: rgba(18, 18, 18, 0.95);
  border: 1px solid rgba(157, 104, 255, 0.3);
  border-radius: 999px;
  color: #fdfdfd;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4), 0 0 20px rgba(157, 104, 255, 0.2);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  white-space: nowrap;
  max-width: 90vw;
  pointer-events: none;
}
.toast-enter-active, .toast-leave-active {
  transition: opacity 0.25s, transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-enter-from, .toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(8px);
}

/* --- ACCORDION SLIDE --- */
.slide-down-enter-active, .slide-down-leave-active {
  transition: opacity 0.25s ease, max-height 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  overflow: hidden;
}
.slide-down-enter-from, .slide-down-leave-to {
  opacity: 0;
  max-height: 0;
}
.slide-down-enter-to, .slide-down-leave-from {
  opacity: 1;
  max-height: 4000px;
}
.node-row {
  user-select: none;
}
.node-row:active {
  background: rgba(157, 104, 255, 0.05);
}

/* --- FADE for tab content --- */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.4s ease, transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

/* Connect orb ring (всегда виден на мобильном, чтобы кнопка не была невидимой) */
.orb-ring {
  border: 1.5px solid rgba(157, 104, 255, 0.4);
  background: radial-gradient(circle, rgba(157, 104, 255, 0.08) 0%, transparent 70%);
  animation: orb-pulse 3s ease-in-out infinite;
}
.orb-ring:hover {
  border-color: rgba(157, 104, 255, 0.7);
}
@keyframes orb-pulse {
  0%, 100% { box-shadow: 0 0 40px rgba(157, 104, 255, 0.2), inset 0 0 30px rgba(157, 104, 255, 0.15); }
  50% { box-shadow: 0 0 60px rgba(157, 104, 255, 0.35), inset 0 0 40px rgba(157, 104, 255, 0.2); }
}
@media (min-width: 768px) {
  .orb-ring {
    animation: none;
  }
}
</style>