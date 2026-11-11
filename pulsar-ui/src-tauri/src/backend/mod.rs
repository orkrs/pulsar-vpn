//! Платформенная абстракция VPN-бэкенда для Tauri-приложения.
//!
//! Архитектура:
//! - На **Windows** Tauri-UI поднимает `pulsar-daemon.exe` (отдельный
//!   процесс) и общается с ним по gRPC. Это даёт надёжную изоляцию и
//!   возможность перезапустить демон, не убивая UI.
//! - На **Android** Tauri-UI живёт в одном процессе, поэтому gRPC
//!   не нужен — мы используем `pulsar_daemon::VpnEngine` напрямую.
//!   Метрики и состояние приходят через тот же [`MetricsEvent`] /
//!   `VpnState`, что и в gRPC-варианте, но без сетевых хопов.
//!
//! [`Backend`] — это enum, варианты которого содержат платформенные
//! детали (`VpnServiceClient` для Windows, `VpnEngine` для Android).
//! Мы выбрали enum, а не `dyn Trait`, чтобы:
//!   1. не платить за виртуальные вызовы на горячем пути;
//!   2. не возиться с `async fn in trait` (до стабилизации в Rust 1.75+
//!      это требовало `BoxFuture` или `async-trait`);
//!   3. хранить `Clone` без `Arc<Mutex<...>>` (Tauri-State требует Clone).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pulsar_daemon::vpn_engine::{ConnectRequest, MetricsEvent, VpnState};
use pulsar_shared::vpn::RoutingConfig;

#[allow(unused_imports)]
use crate::logger as _logger_unused;

/// Глобальная ручка для флага "стримим ли логи в webview".
///
/// Доступ к ней нужен из Tauri-команды `set_logs_streaming`, которая
/// не получает `AppState`. Сама ручка хранится в `OnceLock`, чтобы
/// не плодить глобальных `static mut`.
static LOGS_STREAMING_FLAG: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

/// Сохраняет глобальный флаг. Вызывается один раз из `main()`.
pub fn set_logs_streaming_handle(flag: Arc<AtomicBool>) {
    let _ = LOGS_STREAMING_FLAG.set(flag);
}

/// Устанавливает значение глобального флага. На Android — noop.
pub fn set_logs_streaming_global(enable: bool) {
    if let Some(flag) = LOGS_STREAMING_FLAG.get() {
        flag.store(enable, Ordering::Relaxed);
    }
}

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "windows")]
mod windows;

/// Результат `connect`/`disconnect` в виде простого `(state, message)`,
/// который легко превращается в gRPC `StatusResponse` или в HTTP/JSON
/// ответ для Android (если в будущем понадобится).
#[derive(Debug, Clone)]
pub struct ToggleOutcome {
    pub state: String,
    pub message: String,
}

impl ToggleOutcome {
    pub fn from_state(s: &VpnState) -> Self {
        match s {
            VpnState::Disconnected => Self {
                state: "Disconnected".into(),
                message: "Closed".into(),
            },
            VpnState::Connecting => Self {
                state: "Connecting".into(),
                message: "OK".into(),
            },
            VpnState::Connected => Self {
                state: "Connected".into(),
                message: "OK".into(),
            },
            VpnState::Disconnecting => Self {
                state: "Disconnecting".into(),
                message: "OK".into(),
            },
            VpnState::Error(e) => Self {
                state: "Error".into(),
                message: e.clone(),
            },
        }
    }
}

/// Что мы делаем по `toggle_vpn(connect=true|false)` — на уровне API
/// бэкенда это один метод, потому что и connect, и disconnect —
/// естественные операции на VPN-движке.
#[derive(Debug, Clone)]
pub struct ToggleRequest {
    pub connect: bool,
    pub profile_link: String,
    pub routing: RoutingConfig,
}

/// Платформенная обёртка. Содержит внутри enum с деталями, потому что
/// разные платформы имеют принципиально разные поля (gRPC client vs
/// in-process engine).
#[derive(Clone)]
pub struct Backend {
    inner: Arc<BackendKind>,
}

enum BackendKind {
    #[cfg(target_os = "windows")]
    Windows(windows::WindowsBackend),
    #[cfg(target_os = "android")]
    Android(android::AndroidBackend),
}

impl Backend {
    /// Создаёт бэкенд для текущей платформы.
    ///
    /// Принимает `&tauri::App`, потому что:
    /// - На Windows нужен `resource_dir()` (где лежит `pulsar-daemon.exe`)
    ///   и `AppHandle` (для эмита `vpn_log`).
    /// - На Android — параметр игнорируется.
    pub fn for_tauri(_app: &tauri::App, logs_streaming: Arc<AtomicBool>) -> Result<Self, String> {
        #[cfg(target_os = "windows")]
        {
            tracing::info!("backend: Windows gRPC mode");
            windows::WindowsBackend::attach_app(_app, logs_streaming).map(|b| Self {
                inner: Arc::new(BackendKind::Windows(b)),
            })
        }
        #[cfg(target_os = "android")]
        {
            tracing::info!("backend: Android in-process mode");
            android::AndroidBackend::new().map(|b| Self {
                inner: Arc::new(BackendKind::Android(b)),
            })
        }
    }

    /// Возвращает `true`, если на этой платформе бэкенд — отдельный
    /// процесс. На Android это всегда `false`. Используется UI-стороной
    /// для отображения индикатора "демон" в настройках.
    #[allow(dead_code)]
    pub fn is_external_process(&self) -> bool {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(_) => true,
            #[cfg(target_os = "android")]
            BackendKind::Android(_) => false,
        }
    }

    /// Подключиться или отключиться. Возвращает `(state, message)`,
    /// который UI может показать в нотификации.
    pub async fn toggle(&self, req: ToggleRequest) -> Result<ToggleOutcome, String> {
        let connect_req = ConnectRequest {
            profile_link: req.profile_link,
            routing: req.routing,
        };
        let res = match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => {
                if req.connect {
                    b.connect(connect_req).await
                } else {
                    b.disconnect().await
                }
            }
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => {
                if req.connect {
                    b.connect(connect_req).await
                } else {
                    b.disconnect().await
                }
            }
        };
        res.map(|s| ToggleOutcome::from_state(&s))
    }

    /// Текущее состояние — для редких опросов из UI.
    #[allow(dead_code)]
    pub async fn state(&self) -> VpnState {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => b.state().await,
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => b.state().await,
        }
    }

    /// Подписка на поток метрик. UI вызывает один раз при старте;
    /// в Receiver приходят `MetricsEvent` со скоростями.
    /// `None` — если подписка уже была выдана (второй вызов).
    pub async fn subscribe_metrics(&self) -> Option<tokio::sync::mpsc::Receiver<MetricsEvent>> {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => b.subscribe_metrics().await,
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => b.subscribe_metrics().await,
        }
    }

    /// Корректно остановить VPN и (если применимо) убить демон.
    /// Вызывается на выходе из приложения.
    pub async fn shutdown(&self) {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => b.shutdown().await,
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => b.shutdown().await,
        }
    }
}

// Внутренние модули экспортируются через `mod` выше. main.rs с ними
// напрямую не работает — только через `Backend`.
