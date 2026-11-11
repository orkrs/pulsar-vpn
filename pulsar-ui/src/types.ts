export interface Profile {
  id: number
  name: string
  protocol: string
  ping: number
  active: boolean
  rawLink: string
  flag: string
  country: string
  subscriptionId?: string
  dnsType?: 'https' | 'http' | 'udp'
  allowInsecure?: boolean
  useIpv6?: boolean
}

export interface Subscription {
  id: string
  url: string
  name: string
  expire?: number
  lastUpdated: number
  nodes: Profile[]
}

export interface RoutingConfig {
  mode: 'global' | 'bypass' | 'proxy'
  domains: string[]
  apps: string[]
}

export interface PlatformInfo {
  os: string
  apps_label: string
  apps_placeholder: string
  apps_append_suffix: boolean
  apps_supported: boolean
}

export interface NodeGroup {
  id: string
  name: string
  sub: Subscription | null
  nodes: Profile[]
}

export type TabId = 'dashboard' | 'nodes' | 'settings' | 'logs'
export type VpnPhase = 'off' | 'on' | 'connecting' | 'disconnecting'

export const POPULAR_COUNTRIES = [
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
] as const

export function pingClass(ping: number) {
  if (ping <= 0) return ''
  if (ping < 100) return 'g'
  if (ping < 200) return 'y'
  return 'r'
}

export function formatPing(ping: number) {
  if (ping > 0) return `${ping} мс`
  if (ping === 0) return '…'
  return 'таймаут'
}
