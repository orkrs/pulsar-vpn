//! VPN-движок: бизнес-логика подключения, состояние, метрики.
//!
//! Архитектура:
//! - [`VpnEngine`] — единый API, не зависящий от платформы. Реализуется
//!   через [`PlatformProcess`], который выбирается на этапе компиляции
//!   (`#[cfg(target_os = "…")]`).
//! - На Windows процесс sing-box запускается как дочерний.
//! - На Android движок должен работать через `android.net.VpnService`
//!   (см. заглушку в [`android`]).
//!
//! Все async-методы — `Send + Sync`, чтобы движок можно было шарить
//! между Tauri-командами через `tauri::State`.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use pulsar_shared::vpn::RoutingConfig;

use crate::config_gen;

/// Итоговое состояние, понятное UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VpnState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error(String),
}

impl VpnState {
    pub fn is_active(&self) -> bool {
        matches!(self, VpnState::Connected | VpnState::Connecting)
    }
}

/// Снимок метрик скорости (байт/с).
#[derive(Debug, Clone, Copy, Default)]
pub struct MetricsSnapshot {
    pub download_speed: u64,
    pub upload_speed: u64,
}

/// Запрос на подключение.
#[derive(Debug, Clone)]
pub struct ConnectRequest {
    pub profile_link: String,
    pub routing: RoutingConfig,
}

/// Событие метрик, которое UI получает через подписку.
#[derive(Debug, Clone, Copy)]
pub struct MetricsEvent {
    pub download_speed: u64,
    pub upload_speed: u64,
}

/// Абстракция над "процессом sing-box / VpnService" для текущей платформы.
///
/// Реализация подбирается на этапе компиляции — см. [`process`].
mod process;
pub use process::PlatformProcess;

/// Главный VPN-движок. Держит общее состояние и платформенный процесс.
///
/// Шаблон использования (одинаков на Win и Android):
/// ```no_run
/// # use pulsar_daemon::vpn_engine::{VpnEngine, ConnectRequest, MetricsEvent};
/// # async fn demo(engine: VpnEngine) -> Result<(), String> {
/// engine.connect(ConnectRequest { profile_link: "vless://…".into(), routing: Default::default() }).await?;
/// let mut metrics = engine.subscribe_metrics();
/// while let Some(m) = metrics.recv().await {
///     println!("↓{} ↑{}", m.download_speed, m.upload_speed);
/// }
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct VpnEngine {
    inner: Arc<Inner>,
}

struct Inner {
    state: Arc<Mutex<VpnState>>,
    process: Arc<PlatformProcess>,
    metrics_tx: mpsc::Sender<MetricsEvent>,
    metrics_rx: Arc<Mutex<Option<mpsc::Receiver<MetricsEvent>>>>,
}

impl VpnEngine {
    /// Создаёт новый движок. На Windows ищет `sing-box(.exe)` рядом с текущим
    /// бинарём (или в PATH). На Android требует JNI-инициализации (см. android.rs).
    pub fn new() -> Result<Self, String> {
        let (metrics_tx, metrics_rx) = mpsc::channel::<MetricsEvent>(8);
        let process = PlatformProcess::new()
            .map_err(|e| format!("failed to init platform process: {e}"))?;
        Ok(Self {
            inner: Arc::new(Inner {
                state: Arc::new(Mutex::new(VpnState::Disconnected)),
                process: Arc::new(process),
                metrics_tx,
                metrics_rx: Arc::new(Mutex::new(Some(metrics_rx))),
            }),
        })
    }

    pub async fn state(&self) -> VpnState {
        self.inner.state.lock().await.clone()
    }

    /// Подписаться на стрим метрик. Возвращает Receiver, который нужно
    /// периодически опрашивать (например, в tauri::async_runtime::spawn).
    pub async fn subscribe_metrics(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        self.inner.metrics_rx.lock().await.take()
    }

    /// Подключиться к VPN-узлу.
    pub async fn connect(&self, req: ConnectRequest) -> Result<VpnState, String> {
        // Помечаем состояние Connecting атомарно (проверяем, что не подключены).
        {
            let mut s = self.inner.state.lock().await;
            if s.is_active() {
                return Err("уже подключено или идёт подключение".into());
            }
            *s = VpnState::Connecting;
        }

        // Генерируем конфиг sing-box. Это общая логика для обеих платформ.
        let config_json = config_gen::generate_singbox_config(&req.profile_link, &req.routing)
            .map_err(|e| {
                let err = format!("config generation failed: {e:#}");
                // Возвращаем состояние в Disconnected асинхронно, чтобы не держать lock.
                let this = self.clone();
                let err_for_state = err.clone();
                tokio::spawn(async move {
                    let mut s = this.inner.state.lock().await;
                    *s = VpnState::Error(err_for_state);
                    *s = VpnState::Disconnected;
                });
                err
            })?;

        // Запускаем платформенный процесс.
        match self.inner.process.start(&config_json).await {
            Ok(()) => {
                *self.inner.state.lock().await = VpnState::Connected;
                // Запускаем фоновый сборщик метрик.
                let process = self.inner.process.clone();
                let tx = self.inner.metrics_tx.clone();
                tokio::spawn(metrics_loop(process, tx));
                Ok(VpnState::Connected)
            }
            Err(e) => {
                let msg = format!("{e}");
                *self.inner.state.lock().await = VpnState::Error(msg.clone());
                *self.inner.state.lock().await = VpnState::Disconnected;
                Err(msg)
            }
        }
    }

    /// Отключиться от VPN.
    pub async fn disconnect(&self) -> Result<VpnState, String> {
        *self.inner.state.lock().await = VpnState::Disconnecting;
        if let Err(e) = self.inner.process.stop().await {
            tracing::warn!(error = %e, "vpn process stop failed");
        }
        *self.inner.state.lock().await = VpnState::Disconnected;
        Ok(VpnState::Disconnected)
    }
}

impl Default for VpnEngine {
    fn default() -> Self {
        // Для тестов / previews, где не нужен реальный процесс.
        Self::new().unwrap_or_else(|e| {
            tracing::error!("VpnEngine::new() failed: {e}; using noop engine");
            // Запасной вариант — сообщаем об ошибке в любом connect().
            // Безопаснее, чем падать, потому что Tauri-инициализация не должна
            // ломаться, даже если sing-box отсутствует.
            let (metrics_tx, _metrics_rx) = mpsc::channel::<MetricsEvent>(1);
            Self {
                inner: Arc::new(Inner {
                    state: Arc::new(Mutex::new(VpnState::Error(e.clone()))),
                    process: Arc::new(PlatformProcess::noop()),
                    metrics_tx,
                    metrics_rx: Arc::new(Mutex::new(None)),
                }),
            }
        })
    }
}

/// Периодический сбор метрик с платформенного процесса.
/// На Windows это sysinfo + фильтр по TUN-интерфейсу.
/// На Android — чтение счётчиков из VpnService через JNI.
async fn metrics_loop(process: Arc<PlatformProcess>, tx: mpsc::Sender<MetricsEvent>) {
    let mut last = MetricsSnapshot::default();
    let mut initialized = false;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let snap = process.sample_metrics().await;
        let ev = if !initialized {
            last = snap;
            initialized = true;
            MetricsEvent { download_speed: 0, upload_speed: 0 }
        } else {
            let ev = MetricsEvent {
                download_speed: snap.download_speed.saturating_sub(last.download_speed),
                upload_speed: snap.upload_speed.saturating_sub(last.upload_speed),
            };
            last = snap;
            ev
        };
        if tx.send(ev).await.is_err() {
            // Receiver отвалился — UI отписался.
            break;
        }
    }
}
