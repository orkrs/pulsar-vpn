<script setup lang="ts">
import { usePulsar } from '../composables/usePulsar'

const p = usePulsar()
</script>

<template>
  <section class="view view-settings" :class="{ on: p.activeTab === 'settings' }">
    <div>
      <div class="group-title">Общие</div>
      <div class="gcard">
        <div class="grow">
          <div>
            <div class="gt">Автозапуск с системой</div>
            <div class="gd">Стартовать клиент при входе в Windows.</div>
          </div>
          <button class="switch" role="switch" :aria-checked="p.settings.autoStart" aria-label="Автозапуск" @click="p.toggleAutoStart()" />
        </div>
        <div class="grow">
          <div>
            <div class="gt">Сворачивать в трей</div>
            <div class="gd">Закрытие окна сворачивает приложение, соединение не рвётся.</div>
          </div>
          <button class="switch" role="switch" :aria-checked="p.settings.minimizeToTray" aria-label="Сворачивать в трей" @click="p.toggleTray()" />
        </div>
      </div>
    </div>

    <div>
      <div class="group-title">Маршрутизация</div>
      <div class="gcard">
        <div class="grow">
          <div>
            <div class="gt">Режим</div>
            <div class="gd">Какой трафик отправлять через туннель.</div>
          </div>
          <div class="seg dns-seg" style="height:36px" role="group" aria-label="Режим маршрутизации">
            <button :class="{ on: p.routing.mode === 'global' }" @click="p.routing.mode = 'global'; p.saveData()">Global</button>
            <button :class="{ on: p.routing.mode === 'bypass' }" @click="p.routing.mode = 'bypass'; p.saveData()">Bypass</button>
            <button :class="{ on: p.routing.mode === 'proxy' }" @click="p.routing.mode = 'proxy'; p.saveData()">Only VPN</button>
          </div>
        </div>

        <div v-if="p.routing.mode !== 'global'" class="gcol">
          <div>
            <div class="gt">Сайты</div>
            <div class="gd">Домены для режимов «Кроме» и «Только». Enter — добавить.</div>
          </div>
          <div class="tageditor">
            <span v-for="(domain, idx) in p.routing.domains" :key="'d' + idx" class="tagchip">
              {{ domain }}
              <button class="tag-x" aria-label="Удалить домен" @click.stop="p.removeDomain(idx)">×</button>
            </span>
            <input
              v-model="p.newDomainInput"
              type="text"
              name="domains"
              autocomplete="off"
              spellcheck="false"
              placeholder="example.com…"
              aria-label="Добавить домен"
              @keyup.enter="p.addDomain()"
            />
          </div>
          <div class="tag-hint">Точные домены и wildcard: example.com, *.cdn.net</div>
        </div>

        <div v-if="p.routing.mode !== 'global' && p.platformInfo.apps_supported" class="gcol">
          <div>
            <div class="gt">{{ p.platformInfo.apps_label }}</div>
            <div class="gd">Процессы для тех же режимов.</div>
          </div>
          <div class="tageditor">
            <span v-for="(app, idx) in p.routing.apps" :key="'a' + idx" class="tagchip">
              {{ app }}
              <button class="tag-x" aria-label="Удалить приложение" @click.stop="p.removeApp(idx)">×</button>
            </span>
            <input
              v-model="p.newAppInput"
              type="text"
              name="apps"
              autocomplete="off"
              spellcheck="false"
              :placeholder="p.platformInfo.apps_placeholder + '…'"
              aria-label="Добавить приложение"
              @keyup.enter="p.addApp()"
            />
          </div>
        </div>

        <div class="grow">
          <div>
            <div class="gt">GeoIP-маршрутизация</div>
            <div class="gd">Маршрут трафика по стране IP назначения.</div>
          </div>
          <button
            class="switch"
            role="switch"
            :aria-checked="p.settings.enableGeoip"
            aria-label="GeoIP"
            @click="p.settings.enableGeoip = !p.settings.enableGeoip; p.saveData()"
          />
        </div>
        <div v-if="p.settings.enableGeoip" class="chips">
          <button
            v-for="c in p.POPULAR_COUNTRIES"
            :key="c.code"
            class="chip"
            :data-a="p.settings.geoipRules[c.code] || 'proxy'"
            @click="p.cycleGeo(c.code)"
          >
            {{ c.flag }} {{ c.code }}
            <b>{{ (p.settings.geoipRules[c.code] || 'proxy').toUpperCase() }}</b>
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
