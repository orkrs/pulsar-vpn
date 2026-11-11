# Ревью бэкенда Pulsar VPN: маршрутизация, GeoIP, исключения

Проверен пайплайн: UI (`App.vue`) → Tauri (`lib.rs::toggle_vpn`) → gRPC/in-process → `VpnEngine::connect` → `config_gen.rs` → конфиг Xray → запуск `xray.exe` (TUN).

## Вердикт коротко

| Механизм | Статус |
|---|---|
| GeoIP-правила | ✅ Работают корректно |
| Исключения по сайтам (домены) | ⚠️ Работают, но wildcard `*.` сломан |
| Исключения по приложениям | ✅ Работают (process-matching на Windows поддерживается ядром) |
| Режимы Global / Bypass / Proxy | ❌ Bypass неотличим от Global — два из трёх режимов дают одинаковый конфиг |
| Выбор DNS-типа (HTTPS/HTTP/UDP) | ❌ Поле протаскивается, но игнорируется генератором |
| «Разрешить небезопасный TLS» | ⚠️ Действует только на Hysteria2, для VLESS захардкожен `false` |
| IPv6-переключатель | ⚠️ Реализован наполовину + потенциальная утечка v6 |

---

## 1. Режимы: Global ≡ Bypass (баг)

`config_gen.rs:15–16`:

```rust
let selected_tag = if routing.mode == 2 { "proxy" } else { "direct" };
let default_tag  = if routing.mode == 2 { "direct" } else { "proxy" };
```

Enum Mode: `GLOBAL=0, BYPASS=1, PROXY=2`. Для режимов 0 и 1 теги идентичны → генерируется одинаковый конфиг («всё через туннель, список напрямую»). Пользователь переключает три режима, получает два поведения.

**Фикс** — развести семантику:

```rust
let (list_tag, default_tag, apply_lists) = match routing.mode {
    0 => ("proxy",  "proxy",  false), // GLOBAL: весь трафик в туннель, списки игнорируем
    1 => ("direct", "proxy",  true),  // BYPASS: туннель минус исключения
    _ => ("proxy",  "direct", true),  // PROXY: только исключения в туннеле
};
// правила доменов/apps заворачивать только если apply_lists
```

Тест `uses_current_xray_tun_mask_and_explicit_default_route` (mode=1, последний rule→proxy) остаётся валидным; добавить кейс на различие 0/1/2.

## 2. dns_type мёртвое поле

UI собирает `DnsType` (`lib.rs`: `Some("http") => DnsHttp…`), proto тащит его до движка — и дальше ни одного чтения в `config_gen.rs` (grep по файлу: 0 использований). Сервер всегда `https+local://1.1.1.1/dns-query`.

**Фикс:**

```rust
let dns_server = match DnsType::try_from(routing.dns_type) {
    Ok(DnsType::DnsUdp)  => "1.1.1.1".to_string(),
    Ok(DnsType::DnsHttp) => "http://1.1.1.1/dns-query".to_string(),
    _                    => "https+local://1.1.1.1/dns-query".to_string(),
};
"dns": { "servers": [dns_server], ... }
```

## 3. allow_insecure не действует на VLESS

HY2: передаётся (`config_gen.rs:10,137`) ✓. VLESS: `vless_outbound(&url)` не принимает флаг, внутри захардкожено:

```rust
} else if security == "tls" {
    stream["tlsSettings"] = json!({ ..., "allowInsecure": false });
```

**Фикс:** `vless_outbound(&url, routing.allow_insecure)` и подставлять флаг. Для reality не нужен, для ws/grpc+tls — нужен тем же местом.

## 4. Wildcard-домены молча не работают

Домены превращаются в `domain:{d}` (строка 28 области правил). В Xray префикс `domain:` уже означает «домен и все поддомены», а литеральная звезда `*.cdn.net` как подстрока не сматчится никогда → правило-пустышка.

**Фикс при построении списка:**

```rust
let d = d.strip_prefix("*.").unwrap_or(d);
domains.push(format!("domain:{d}"));
```

(Опционально: поддержать `full:` для точных имён и `regexp:` по явному префиксу `re:`.)

Сами списки доезжают целиком: `toggle_vpn` пересылает `domains`/`apps` без изменений ✓; сниффинг с `routeOnly:true` + `destOverride [http,tls,quic]` настроен правильно, чтобы доменные правила видели SNI сквозь TUN ✓.

## 5. Приложения: ок

`{"type":"field","process":[...]}` — Xray поддерживает process-matching на Windows; имена вида `chrome.exe` совпадают с суффикс-логикой UI (`apps_append_suffix`). Правило добавляется только при непустом списке ✓.

## 6. GeoIP: работает

- Правила `{ ip:["geoip:{cc}"], outboundTag }` добавляются только при `enable_geoip=true` ✓
- `domainStrategy` корректно переключается `AsIs → IPIfNonMatch` именно в этом случае ✓
- Двойная валидация кодов стран (Tauri `lib.rs`: 2 буквы + действие ∈ {proxy,direct,block}; повторная проверка действия в `config_gen`) ✓
- `geoip.dat`/`geosite.dat` лежат рядом с `xray.exe` и бандлятся в ресурсы ✓; тесты дополнительно гоняют сгенерированный конфиг через реальный `xray run -test` с `XRAY_LOCATION_ASSET` ✓

Единственное: порядок правил из HashMap недетерминирован — здесь безвредно (правила независимы).

## 7. use_ipv6: полуфабрикат + риск утечки

Влияет только на `queryStrategy UseIP/UseIPv4`. Но TUN поднимается только с IPv4 (`gateway ["172.19.0.1/30"]`, v6-маршрутов нет). При `use_ipv6=true` приложения получат AAAA и пойдут по физическому интерфейсу **мимо туннеля** — утечка.

Минимальный анти-лик без полной v6-реализации:

```rust
if !routing.use_ipv6 {
    rules.push(json!({"type":"field","ip":["::/0"],"outboundTag":"block"}));
}
```
(и держать `queryStrategy: UseIPv4` независимо от флага, пока v6-туннель не реализован).

## 8. Блокирующие вызовы в async-контексте

`VpnEngine::connect` (async) вызывает `generate_xray_config` синхронно, а тот делает:
- `resolve_host`: `(host,443).to_socket_addrs()` — блокирующий системный резолвер;
- fallback DoH через `reqwest::blocking` — дроп клиента внутри рантайма tokio может паниковать («Cannot drop a runtime in a context where blocking is not allowed»);
- `windows_default_interface()` — спавнит `powershell.exe Get-NetRoute` (сотни мс) на каждом коннекте; падение PowerShell = неудавшийся коннект.

**Фикс:**

```rust
let cfg = tokio::task::spawn_blocking(move || {
    config_gen::generate_xray_config(&link, &routing)
}).await.map_err(|e| format!("join: {e}"))??;
```

Плюс: кэшировать интерфейс по-стартеру или заменить на `GetBestRoute` FFI; добавить fallback, если PS недоступен.

## 9. Что сделано правильно

- Явный catch-all `{"network":"tcp,udp"} → default_tag` (не полагается на первый outbound) ✓
- `geoip:private → direct` первым правилом ✓
- Токен-аутентификация gRPC (`x-pulsar-token` interceptor) + биндинг строго на 127.0.0.1 ✓
- VLESS: `flow` пробрасывается (`xtls-rprx-vision` работает), reality-параметры pbk/sid/spx/fp на месте, encryption по умолчанию none ✓
- HY2: auth из username, sni/obfs-password из query, ALPN h3 ✓ (заметка: порты-hopping `mport` не парсится — если серверы его используют, допилить)
- Конфиг пишется во временный файл и скармливается xray; тесты валидируют схему против реального бинарника (`xray run -test`) — сильная защита от регрессий формата ✓
- Метрики: дельта по кумулятивным счётчикам с `saturating_sub` и ресетом на сессию ✓

## 10. Мелочи (не блокирует)

- `kill_ghosts()` убивает ВСЕ процессы по имени образа — заденет чужие инстансы.
- Токен демона виден в command line процесса (для localhost-модели угроз приемлемо; можно перейти на env/handle).
- Ошибка коннекта: состояние `Error` мгновенно перезаписывается `Disconnected` — статус «Error» почти не наблюдается через GetStatus (сообщение ошибки при этом возвращается в ответе ToggleConnection).
- `fetch_subscription` создаёт новый `reqwest::Client` на вызов — микрокост.

## Порядок фиксов

1. spawn_blocking вокруг generate_xray_config (+кэш интерфейса) — стабильность.
2. Анти-утечка `::/0 → block` при выключенном v6 — безопасность.
3. Развести Global/Bypass/Proxy — функциональность.
4. dns_type → серверы DNS; allow_insecure → VLESS; strip `*.` у доменов — добивание фич.
5. Обновить юнит-тесты под новую матрицу режимов.
