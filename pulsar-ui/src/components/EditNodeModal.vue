<script setup lang="ts">
import { X } from 'lucide-vue-next'
import { usePulsar } from '../composables/usePulsar'

const p = usePulsar()
</script>

<template>
  <div v-if="p.editModalOpen" class="overlay" @click.self="p.editModalOpen = false">
    <div class="modal" role="dialog" aria-modal="true" aria-label="Настройки узла">
      <div class="m-head">
        <span class="m-title">Настройки узла</span>
        <button class="m-close" aria-label="Закрыть" @click="p.editModalOpen = false"><X :size="15" /></button>
      </div>
      <div class="grow" style="padding:0;gap:12px">
        <div style="flex:1">
          <div class="gt">DNS-транспорт</div>
          <div class="gd">DoH — шифрованный резолв.</div>
        </div>
        <div class="seg dns-seg" style="height:36px">
          <button :class="{ on: p.editedDnsType === 'https' }" @click="p.editedDnsType = 'https'">HTTPS</button>
          <button :class="{ on: p.editedDnsType === 'http' }" @click="p.editedDnsType = 'http'">HTTP</button>
          <button :class="{ on: p.editedDnsType === 'udp' }" @click="p.editedDnsType = 'udp'">UDP</button>
        </div>
      </div>
      <div class="grow" style="padding:12px 0;border-top:1px solid var(--line)">
        <div>
          <div class="gt">Allow insecure</div>
          <div class="gd">Пропустить проверку TLS.</div>
        </div>
        <button
          class="switch"
          role="switch"
          :aria-checked="p.editedAllowInsecure"
          aria-label="Allow insecure"
          @click="p.editedAllowInsecure = !p.editedAllowInsecure"
        />
      </div>
      <div class="grow" style="padding:12px 0;border-top:1px solid var(--line)">
        <div>
          <div class="gt">IPv6</div>
          <div class="gd">Выкл: только IPv4. Вкл: разрешить IPv6.</div>
        </div>
        <button
          class="switch"
          role="switch"
          :aria-checked="p.editedUseIpv6"
          aria-label="IPv6"
          @click="p.editedUseIpv6 = !p.editedUseIpv6"
        />
      </div>
      <div style="display:flex;gap:10px;margin-top:8px">
        <button class="btn-ghost" style="flex:1;height:40px" @click="p.editModalOpen = false">Отмена</button>
        <button class="btn-accent" style="flex:1;height:40px" @click="p.saveProfileSettings()">Сохранить</button>
      </div>
    </div>
  </div>
</template>
