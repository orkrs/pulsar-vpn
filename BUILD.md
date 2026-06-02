# Сборка Pulsar VPN

## TL;DR — одна команда

```powershell
cargo xtask build            # debug-профиль, быстрая сборка
cargo xtask build --release  # релиз-профиль + MSI/NSIS инсталлятор
```

`xtask` — это встроенный в воркспейс оркестратор (крейт `xtask/`).
Он сам собирает `pulsar-daemon`, копирует его в `pulsar-ui/src-tauri/binaries/`,
ставит зависимости UI, билдит фронтенд и запускает Tauri-бандлер.
Больше никакой ручной возни с копированием бинарников между шагами.

## Требования

| Что | Минимум | Зачем |
| --- | --- | --- |
| Rust | 1.85+ (`rustup update stable`) | `edition = "2024"` |
| Bun | latest | `bun install`, `bun run tauri build` |
| Windows SDK | актуальный | требует Tauri 2 для WebView2 |
| sing-box | любой актуальный билд | кладётся в `pulsar-ui/src-tauri/binaries/sing-box.exe` |

`xtask` ничего дополнительного не требует — это просто ещё один крейт в воркспейсе.

## Команды `xtask`

| Команда | Что делает |
| --- | --- |
| `cargo xtask build [--release]` | Полная prod-сборка для **текущей платформы** (Windows installer / etc.) |
| `cargo xtask build-android [--release]` | Сборка Android APK (см. [Android](#android)) |
| `cargo xtask build-all [--release]` | Сборка под текущую платформу + Android (если SDK/NDK доступны) |
| `cargo xtask dev` | Поднимает debug-демон + `tauri dev` с hot-reload фронта |
| `cargo xtask lint` | `cargo check` + `cargo clippy -D warnings` + `vue-tsc` |
| `cargo xtask clean [--deep]` | Чистит `target/`, `dist/`. С `--deep` ещё и `node_modules/` |
| `cargo xtask info` | Показывает пути, версии инструментов, статус бинарников |
| `cargo xtask help` | Краткая справка |

### Прямой аналог на старый лад (если не хочется `xtask`)

`.\build.ps1` — обратно-совместимая обёртка, прокидывающая аргументы в `cargo xtask`:

```powershell
.\build.ps1                 # = cargo xtask build
.\build.ps1 -Release        # = cargo xtask build --release
.\build.ps1 -Dev            # = cargo xtask dev
.\build.ps1 -Clean -Deep    # = cargo xtask clean --deep
.\build.ps1 -Lint           # = cargo xtask lint
```

### `just` (опционально)

Если у вас установлен [just](https://github.com/casey/just), есть короткие алиасы:

```powershell
just build          # cargo xtask build
just dev            # cargo xtask dev
just lint           # cargo xtask lint
just clean deep=yes # cargo xtask clean --deep
```

## Что происходит под капотом `cargo xtask build`

```
[1/5] cargo build -p pulsar-daemon --release
       └─ target/release/pulsar-daemon.exe
[2/5] copy → pulsar-ui/src-tauri/binaries/pulsar-daemon.exe
[3/5] bun install --frozen-lockfile (в pulsar-ui/)
[4/5] bun run build  (vue-tsc --noEmit && vite build)
[5/5] bun run tauri build  (Rust src-tauri + MSI/NSIS)
```

`sing-box.exe` НЕ собирается — это внешний бинарь, его нужно положить вручную в
`pulsar-ui/src-tauri/binaries/sing-box.exe`. `xtask build` проверит наличие
и напишет предупреждение, если его нет.

## Android

`pulsar-ui` собирается под Android через Tauri 2 mobile. Архитектурно
приложение работает иначе, чем на Windows:

| | Windows | Android |
| --- | --- | --- |
| VPN-бэкенд | отдельный `pulsar-daemon.exe` (gRPC) | `pulsar_daemon::VpnEngine` **в процессе** приложения |
| Доставка sing-box | внешний `.exe` в `binaries/` | (на будущее) NDK-билд в `.so` через JNI |
| Системный доступ | TUN от sing-box | `android.net.VpnService` (см. stub в `pulsar-daemon/src/vpn_engine/process/android.rs`) |
| UI | десктоп (sidebar, 3D-сфера) | адаптивный (bottom-nav, без 3D для экономии батареи) |

Платформенная абстракция живёт в `pulsar-ui/src-tauri/src/backend/{mod,windows,android}.rs`:
`enum BackendKind` подбирается на этапе компиляции (`#[cfg(target_os = "…")]`).

### Требования для Android-сборки

| Что | Минимум | Где взять |
| --- | --- | --- |
| `cargo tauri` CLI | 2.0+ | `cargo install tauri-cli --version "^2.0"` |
| JDK | 17+ | [Adoptium](https://adoptium.net/) или Android Studio → Settings → Build, Execution, Deployment → Build Tools → Gradle → Gradle JDK |
| Android Studio | Hedgehog (2023.1.1) или новее | https://developer.android.com/studio |
| Android SDK | Platform 34, Build-Tools 34.0.0 | Android Studio → SDK Manager |
| NDK | r26+ (Side by side) | Android Studio → SDK Manager → SDK Tools → NDK (Side by side) |
| Rust targets | хотя бы `aarch64-linux-android` | `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android` |

Переменные окружения:

```powershell
# Windows PowerShell
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:JAVA_HOME = "C:\Program Files\Android Studio\jbr"  # или путь к JDK 17+
```

### Одноразовая инициализация

```bash
cd pulsar-ui
cargo tauri android init
```

Эта команда создаст `pulsar-ui/gen/android/` с Gradle-проектом. Папка
`.gitignore`-ится (регенерируется), но локальные правки (например, разрешения
в `AndroidManifest.xml`) можно коммитить рядом.

### Сборка

```bash
# Debug APK (быстрая итерация, неоптимизированная)
cargo xtask build-android

# Release APK/AAB (оптимизированный, готовый к публикации)
cargo xtask build-android --release
```

Или напрямую через Tauri CLI:

```bash
cd pulsar-ui
cargo tauri android build          # release APK + AAB
cargo tauri android debug          # debug APK
```

`xtask build-android` автоматически проверяет все pre-requisites и пишет
чёткие ошибки, если чего-то не хватает.

### Split tunneling (по итогам Phase 5)

Реализован в `pulsar-daemon/src/config_gen.rs` + UI в `pulsar-ui/src/App.vue`.

**Три режима:**
- **Global** — весь трафик через туннель.
- **Bypass** (по умолчанию) — пользовательские домены/приложения идут
  напрямую, остальное через прокси.
- **Only VPN** — пользовательские домены/приложения идут через прокси,
  остальное блокируется.

**Домены** — `domain_suffix` match (`google.com` покрывает `mail.google.com`),
зеркалятся в DNS-правила sing-box (защита от DNS-утечки).

**Per-app routing:**
- **Windows** — `process_name` rule (sing-box Windows). В исключения также
  добавлены `pulsar-ui.exe` / `pulsar-daemon.exe` / `sing-box.exe`, чтобы
  убийство VPN не убивало сам VPN.
- **Linux/macOS** — аналогично, без `.exe`.
- **Android** — `process_name` НЕ генерируется (sing-box на Android не имеет
  доступа к `/proc`). Per-app routing на Android требует
  `VpnService.Builder.addAllowedApplication/addDisallowedApplication` на
  Kotlin-стороне; до реализации UI скрывает Apps-секцию через флаг
  `apps_supported: false` из `get_platform_info`.

**GeoIP routing:**
- Коды стран нормализуются в lowercase (SagerNet sing-geoip хранит файлы
  в lowercase: `geoip-ru.db`, `geoip-us.db`).
- Действия: `proxy` / `direct` / `block`. Для `direct` и `block`
  зеркалятся DNS-правила, чтобы избежать утечки DNS.

**Security toggles (UI):**
- DNS Hijack — always on, информационная строка.
- Kill Switch — помечен `Planned` (netsh advfirewall на Windows / iptables
  на Android). Реализация — отдельная задача.

### Известные ограничения

- **VPN-бэкенд на Android — заглушка.** `pulsar_daemon::VpnEngine` на Android
  возвращает `Err("Android VPN-бэкенд ещё не реализован")`. Реальная
  имплементация требует:
  1. Kotlin-обёртки `VpnService` + JNI-моста;
  2. NDK-сборки `libsingbox.so` под `aarch64-linux-android`;
  3. Передачи `FileDescriptor` из `VpnService.Builder.establish()` в sing-box
      через FFI;
  4. Передачи списка apps в `VpnService.Builder.addAllowedApplication()`
      из Kotlin, чтобы per-app routing на Android работал.
  Это многоэтапная задача, см. комментарии в
  `pulsar-daemon/src/vpn_engine/process/android.rs`.
- На Android `tauri-plugin-{tray-icon,autostart}` не используются (нет
  иконки в трее и автостарта в смысле Windows). UI скрывает соответствующие
  элементы автоматически через `isMobile` ref.
- `xtask build-all` запускает обе сборки подряд; если Android-окружение
  недоступно, десктоп-артефакты всё равно готовы, и общий exit code = 0
  (с предупреждением).

## Раннее обнаружение ошибок

`pulsar-ui/src-tauri/build.rs` проверяет наличие `pulsar-daemon.exe` ДО
запуска Tauri-бандлера. Если вы случайно собираете UI-крейт без предварительного
`cargo xtask build`, увидите:

```
  pulsar-daemon binary not found at: …\pulsar-ui\src-tauri\binaries\pulsar-daemon.exe

  Запустите `cargo xtask build` (или `cargo xtask dev`) из корня воркспейса —
  он соберёт демона и положит его в нужное место.
```

Чтобы обойти (например, для `cargo check` в CI): `PULSAR_SKIP_DAEMON_CHECK=1`.

## Воркспейс и метаданные

Корневой `Cargo.toml` содержит общие метаданные (`[workspace.package]`):
- `version = "0.1.0"`
- `edition = "2024"`
- `rust-version = "1.85"`

Если нужно бампнуть версию, достаточно поменять одно место в корневом
`Cargo.toml` и синхронизировать с `pulsar-ui/package.json` и
`pulsar-ui/src-tauri/tauri.conf.json` (Tauri читает версию из своего
`Cargo.toml`, Vite — из `package.json`).

### Профили сборки

`[profile.release]` в корневом `Cargo.toml`:
- `lto = "thin"` — дешёвая по времени линковка, заметный выигрыш по размеру
- `codegen-units = 1` — лучше оптимизации, медленнее компиляция
- `strip = "symbols"` — убирает debug-символы из бинарников (~15-20% размера)

`[profile.dev]`: `opt-level = 1` — даёт 30-40% прироста по скорости vs.
`opt-level = 0`, инкрементальная пересборка остаётся быстрой.

## Структура воркспейса

```
pulsar-vpn/
├── Cargo.toml                 # корень воркспейса, [workspace.package], [profile.release]
├── xtask/                     # оркестратор сборки (`cargo xtask …`)
│   ├── Cargo.toml
│   └── src/main.rs
├── pulsar-shared/             # protobuf-контракты (VpnService)
├── pulsar-daemon/             # lib+bin: VPN-движок (config_gen, vpn_engine)
│   ├── Cargo.toml             # [lib] name = "pulsar_daemon", [[bin]] pulsar-daemon
│   └── src/
│       ├── lib.rs             # публичный API: config_gen, logger, vpn_engine
│       ├── main.rs            # Windows-only: gRPC-сервер-обёртка
│       ├── config_gen.rs
│       ├── logger.rs
│       └── vpn_engine/
│           ├── mod.rs         # VpnEngine: state, connect, metrics
│           ├── process.rs     # platform-абстракция
│           └── process/
│               ├── windows.rs # дочерний sing-box + sysinfo
│               └── android.rs # заглушка (см. Android секцию)
└── pulsar-ui/                 # Tauri + Vue фронтенд
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    ├── src/                   # Vue/TS код
    │   ├── App.vue            # адаптивный layout (isMobile + media-queries)
    │   ├── main.ts
    │   └── components/
    │       ├── PulsarSphere.vue  # 3D-сцена, отключается на мобильном
    │       └── BottomNav.vue     # bottom navigation (<768px)
    └── src-tauri/             # Rust-бэкенд Tauri
        ├── Cargo.toml
        ├── build.rs           # проверяет наличие демона
        ├── tauri.conf.json    # bundle.android.{minSdkVersion,autoIncrementVersionCode}
        └── src/
            ├── main.rs        # Tauri-команды, делегируют в Backend
            ├── lib.rs
            ├── logger.rs
            └── backend/       # платформенная абстракция
                ├── mod.rs
                ├── windows.rs # gRPC + spawn демона
                └── android.rs # in-process VpnEngine
```

## Troubleshooting

### `error: linker 'link.exe' not found`
Visual Studio Build Tools с C++ workload: https://visualstudio.microsoft.com/visual-cpp-build-tools/

### `error: WebView2 not found`
Установите Evergreen WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/

### `cargo xtask build` валится на шаге Tauri
Чаще всего — отсутствует `sing-box.exe` или нет интернета для загрузки зависимостей.
Запустите `cargo xtask info` для диагностики путей.

### Хочется собирать только daemon
```bash
cargo build -p pulsar-daemon --release
```
или
```bash
cargo xtask build --release  # сделает это за вас + всё остальное
```

### `cargo tauri android` ругается "SDK location not found"
`ANDROID_HOME` не задан. На Windows: `System Properties → Environment Variables → New → ANDROID_HOME = C:\Users\<you>\AppData\Local\Android\Sdk`. Перезапустите терминал.

### `cargo tauri android build` падает на линковке NDK
Скорее всего, не установлен нужный Rust target. Выполните:
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```
