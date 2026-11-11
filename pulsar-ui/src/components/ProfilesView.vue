<script setup lang="ts">
import { Plus, Search, RefreshCcw, ChevronDown, ChevronRight, Settings2, X, MoreVertical } from 'lucide-vue-next'
import { usePulsar } from '../composables/usePulsar'
import { formatPing, pingClass } from '../types'

const p = usePulsar()
</script>

<template>
  <section class="view view-profiles" :class="{ on: p.activeTab === 'nodes' }">
    <div class="toolbar">
      <label class="search">
        <Search :size="16" aria-hidden="true" />
        <input
          v-model="p.searchQuery"
          type="search"
          name="node-search"
          autocomplete="off"
          spellcheck="false"
          placeholder="Поиск сервера…"
          aria-label="Поиск сервера"
        />
      </label>
      <div class="seg" role="group" aria-label="Сортировка">
        <button :class="{ on: p.sortBy === 'name' }" @click="p.toggleSort('name')">По имени</button>
        <button :class="{ on: p.sortBy === 'ping' }" @click="p.toggleSort('ping')">По пингу</button>
      </div>
      <button class="btn-ghost" :disabled="p.isPinging" @click="p.refreshPings()">
        <RefreshCcw :size="15" :class="{ 'animate-spin': p.isPinging }" aria-hidden="true" />
        Пинг
      </button>
      <button class="btn-accent" @click="p.openImport()">
        <Plus :size="15" aria-hidden="true" />
        Импорт
      </button>
    </div>

    <div class="sub-strip">
      <div class="sub-info">
        <div class="sub-name">{{ p.subscriptions.length ? `${p.subscriptions.length} подписок` : 'Локальные узлы' }}</div>
        <div class="sub-expire">{{ p.profiles.length }} узлов всего</div>
      </div>
      <div class="sub-meter">
        <div class="meter-track">
          <div class="meter-fill" :style="{ width: Math.min(100, p.profiles.length * 8) + '%' }" />
        </div>
        <div class="meter-cap">{{ p.profiles.length }} узлов</div>
      </div>
    </div>

    <div v-if="!p.nodeGroups.length" class="esearch">Нет узлов. Импортируйте профиль или подписку.</div>

    <div v-for="group in p.nodeGroups" :key="group.id" class="group-block">
      <button class="group-head" @click="p.toggleSubExpanded(group.id)">
        <component :is="p.expandedSubs.has(group.id) ? ChevronDown : ChevronRight" :size="16" aria-hidden="true" />
        <div class="gt" style="flex:1;min-width:0;text-align:left">
          <div class="sub-name">{{ group.name }} <span class="badge">{{ group.nodes.length }}</span></div>
          <div class="sub-expire">
            <template v-if="group.sub">
              {{ new Date(group.sub.lastUpdated).toLocaleDateString() }}
              <span v-if="group.sub.expire"> · до {{ new Date(group.sub.expire).toLocaleDateString() }}</span>
            </template>
            <template v-else>Вручную</template>
          </div>
        </div>
        <button
          v-if="group.sub"
          class="btn-ghost"
          style="height:32px;padding:0 10px"
          aria-label="Обновить подписку"
          @click.stop="p.refreshSubscription(group.sub!)"
        >
          <RefreshCcw :size="14" />
        </button>
        <button
          v-if="group.sub"
          class="btn-ghost"
          style="height:32px;padding:0 10px;color:var(--bad)"
          aria-label="Удалить подписку"
          @click.stop="p.deleteSubscription(group.id)"
        >
          <X :size="14" />
        </button>
      </button>

      <div v-if="p.expandedSubs.has(group.id)" class="plist">
        <div
          v-for="profile in p.getFilteredAndSortedNodes(group.nodes)"
          :key="profile.id"
          class="prow"
          :class="{ sel: profile.active, 'menu-open': p.openMenuId === profile.id }"
          @click="p.setActiveProfile(profile.id)"
        >
          <span class="radio" aria-hidden="true" />
          <div style="min-width:0;flex:1">
            <div class="prow-name">{{ profile.flag }} {{ profile.name }}</div>
            <div class="prow-host">{{ profile.protocol }} · {{ profile.country }}</div>
          </div>
          <span class="prow-ping" :class="[pingClass(profile.ping), { checking: profile.ping === 0 }]">
            {{ formatPing(profile.ping) }}
          </span>
          <button
            class="kebab"
            aria-label="Действия узла"
            @click.stop="p.openMenuId = p.openMenuId === profile.id ? null : profile.id"
          >
            <MoreVertical :size="16" />
          </button>
          <div
            v-if="p.openMenuId === profile.id"
            class="menu open"
            style="opacity:1;transform:none;pointer-events:auto;position:absolute;right:12px;top:calc(100% - 8px);z-index:30;min-width:200px;background:var(--surface3);border:1px solid var(--line-strong);border-radius:12px;padding:6px;box-shadow:0 18px 44px rgba(0,0,0,.55)"
            @click.stop
          >
            <button style="display:flex;width:100%;align-items:center;gap:10px;padding:9px 11px;border-radius:8px;font-size:13px;font-weight:600;color:var(--muted);text-align:left" @click="p.openEditModal(profile)">
              <Settings2 :size="14" /> Настройки
            </button>
            <button
              v-if="!group.sub"
              style="display:flex;width:100%;align-items:center;gap:10px;padding:9px 11px;border-radius:8px;font-size:13px;font-weight:600;color:var(--bad);text-align:left"
              @click="p.deleteProfile(profile.id)"
            >
              <X :size="14" /> Удалить
            </button>
          </div>
        </div>
        <div v-if="!p.getFilteredAndSortedNodes(group.nodes).length" class="esearch">Ничего не найдено</div>
      </div>
    </div>
  </section>
</template>
