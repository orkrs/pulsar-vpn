# План оптимизации производительности Pulsar VPN

Симптом: клиент стабильно держит ~50% одного ядра i5-12400F.
Диагностика по коду выявила **два основных источника** и несколько второстепенных.

---

## 1. Диагноз: куда уходит ядро

### 1.1. Сфера three.js рендерит на полном FPS постоянно (главный виновник в простое)

**Где:** `pulsar-ui/src/components/PulsarSphere.vue`

| Проблема | Строка | Почему дорого |
|---|---|---|
| `preserveDrawingBuffer: true` | 134 | Принуждает сохранять framebuffer после каждого кадра — лишняя копия памяти на каждом кадре. Нужен только для скриншотов canvas, здесь не используется |
| `antialias: true` + rAF без ограничения FPS | 134, startLoop | Цикл `requestAnimationFrame` крутится на полной частоте монитора (60–144 кадр/с) всё время, пока открыта вкладка Connect — **даже когда VPN выключен**. `MeshPhysicalMaterial` + пульсация `PointLight` (`sin(elapsed*10)*10`) заставляют пересчитывать освещение каждый кадр |
| Нет паузы при скрытии окна / трее | — | `document.hidden` не обрабатывается |

Остановка цикла есть только при уходе с дашборда (`props.active`, строки 248/265).

**Фиксы (этап 1):**
1. `preserveDrawingBuffer: false`.
2. FPS cap до 30 через аккумулятор времени внутри `animate()` — визуально неотличимо для медленного вращения.
3. Полная остановка цикла в idle: когда `isConnected === false` и переходный lerp завершился (`|stateLerp − target| < 0.01`) → `stopLoop()`, канвас остаётся статичным кадром = 0% GPU/CPU.
4. Слушатель `visibilitychange`: окно скрыто (трей, свернуто) → `stopLoop()`, снова видно → `startLoop()`.

### 1.2. sysinfo создаёт полный снапшот системы ДВАЖДЫ в секунду (главный виновник под нагрузкой)

**Где:** `pulsar-daemon/src/vpn_engine/process/windows.rs:156–171`

```rust
fn refresh_list(&self) {          // вызывается из sample_metrics() каждый тик
    let mut s = System::new_all(); // ← строка 163: ПОЛНЫЙ снапшот (все процессы, диски, сеть)
    s.refresh_networks_list();     // ← ре-энумерация всех адаптеров каждый раз
}
fn refresh_data(&self) -> MetricsSnapshot {
    let mut s = System::new_all(); // ← строка 170: второй полный снапшот
    s.refresh_networks();
}
```

`System::new_all()` — тяжёлая операция (десятки мс, перечисление всех процессов системы).
Пока VPN подключён, это выполняется **2 раза в секунду**.

**Фикс:** хранить один `sysinfo::System` в состоянии структуры; `refresh_networks_list()` — однократно при старте; в цикле метрик — только лёгкий `refresh_networks()` на существующем инстансе. Ожидаемо: с ~10–25% ядра до <1%.

### 1.3. Xray: болтливый лог и лишний sniffing

**Где:** `pulsar-daemon/src/config_gen.rs`
- `"loglevel": "info"` → xray пишет каждую строку в stdout → два потока чтения (`backend/windows.rs:387,410`) → батчер каждые 200 мс (`windows.rs:337,341`) → события `vpn_log` в webview. Конвейер работает даже когда панель логов закрыта.
- `sniffing.destOverride: ["http", "tls", "quic"]` — sniffing QUIC заметно дороже TLS/HTTP.

**Фиксы:** `loglevel: "warning"`; убрать `"quic"` из destOverride (domain-routing работает по TLS/HTTP); эмитить `vpn_log` только при включённом стриминге логов (флаг уже есть — `set_logs_streaming`).

### 1.4. Мёртвая зависимость TresJS

**Где:** `pulsar-ui/src/main.ts:2,9` — `@tresjs/core` регистрируется глобально (`app.use(Tres)`), но сфера написана на чистом `three` (`PulsarSphere.vue:3`). `@tresjs/cientos` вообще нигде не импортируется.
**Фикс:** удалить оба пакета и регистрацию → меньше парсинга JS при старте webview.

### 1.5. backdrop-filter поверх WebGL

**Где:** `pulsar-ui/src/App.vue:1420,1522` (`blur(24px)`), `1568` (`blur(16px)`) — стек композитинга blur поверх анимированного WebGL-канваса дорог для GPU/CPU WebView2.
**Фикс:** заменить на полупрозрачные заливки + 1px бордеры (в редизайне `redesign.html` так и сделано), либо хотя бы отключать blur на панели поверх сферы.

### 1.6. Буфер логов UI без потолка

Логи дописываются через rAF-flush (`App.vue:344`) — массив растёт неограниченно при долгой сессии.
**Фикс:** кап массива ~500 строк со сдвигом.

## 2. Этапы внедрения

### Этап 1 — быстрые фиксы (~1–2 часа, ожидаемо −70…90% CPU)
- [ ] `PulsarSphere.vue`: `preserveDrawingBuffer: false`
- [ ] `PulsarSphere.vue`: FPS cap 30 + остановка цикла в idle + пауза по `visibilitychange`
- [ ] `process/windows.rs`: один переиспользуемый `System`, `refresh_networks_list()` однократно, убрать второй `new_all()`
- [ ] `config_gen.rs`: `loglevel: "warning"`, убрать quic из sniffing
- [ ] Удалить `@tresjs/core` / `@tresjs/cientos` из `main.ts` и `package.json`

### Этап 2 — структурный
- [ ] Заменить three.js-сферу на canvas2d-реализацию — готовый референс в **`redesign.html`**: ~200 строк, 240 частиц, 30 fps cap, render-on-demand в idle, пауза в hidden, ноль зависимостей. Либо минимум: `MeshPhysicalMaterial` → `MeshStandardMaterial`, убрать пульсацию point-light
- [ ] Убрать все `backdrop-filter` (паттерн — в `redesign.html`)
- [ ] Кап буфера логов; `vpn_log` только при включённом стриминге

### Этап 3 — проверка
- [ ] Замер Task Manager по процессам до/после: `pulsar-ui.exe` (+ `msedgewebview2.exe`), `pulsar-daemon.exe`, `xray.exe`
- [ ] Цели: простой (VPN off, окно видно) < 3% ядра; подключено без трафика < 5%; xray — зависит от трафика
- [ ] `cargo clippy` + `cargo xtask lint` после правок

## 3. Что НЕ является проблемой (проверено, не трогаем)

- `metrics_loop` в daemon — 1 Гц, спит 999 мс из секунды: норм.
- gRPC-стрим метрик — 1 сообщение/с: норм.
- `check_ping` — ручной TCP connect, не циклится: норм.
- Токен-gRPC, лог-батчинг 200 мс — разумны.
