//! VPN-движок: бизнес-логика подключения, состояние, метрики.
//!
//! Архитектура:
//! - [`VpnEngine`] — единый API, не зависящий от платформы. Реализуется
//!   через [`PlatformProcess`], который выбирается на этапе компиляции
//!   (`#[cfg(target_os = "…")]`).
//! - На Windows процесс Xray запускается как дочерний.
//! - На Android движок должен работать через `android.net.VpnService`.

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use pulsar_shared::vpn::RoutingConfig;

use crate::config_gen;

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

#[derive(Debug, Clone, Copy, Default)]
pub struct MetricsSnapshot {
    pub download_speed: u64,
    pub upload_speed: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectRequest {
    pub profile_link: String,
    pub routing: RoutingConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricsEvent {
    pub download_speed: u64,
    pub upload_speed: u64,
}

mod process;
pub use process::PlatformProcess;

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
    pub fn new() -> Result<Self, String> {
        let (metrics_tx, metrics_rx) = mpsc::channel::<MetricsEvent>(8);
        let process =
            PlatformProcess::new().map_err(|e| format!("failed to init platform process: {e}"))?;
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

    pub async fn subscribe_metrics(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        self.inner.metrics_rx.lock().await.take()
    }

    pub async fn connect(&self, req: ConnectRequest) -> Result<VpnState, String> {
        {
            let mut s = self.inner.state.lock().await;
            if s.is_active() {
                return Err("Уже подключено или идёт подключение".into());
            }
            *s = VpnState::Connecting;
        }

        let config_json = config_gen::generate_xray_config(&req.profile_link, &req.routing)
            .map_err(|e| {
                let err = format!("Config generation failed: {e:#}");
                let this = self.clone();
                let err_for_state = err.clone();
                tokio::spawn(async move {
                    let mut s = this.inner.state.lock().await;
                    *s = VpnState::Error(err_for_state);
                    *s = VpnState::Disconnected;
                });
                err
            })?;

        match self.inner.process.start(&config_json).await {
            Ok(()) => {
                *self.inner.state.lock().await = VpnState::Connected;
                let process = self.inner.process.clone();
                let tx = self.inner.metrics_tx.clone();
                let state = self.inner.state.clone();
                // ЗАПУСКАЕМ ЦИКЛ С ПРОВЕРКОЙ ЖИВУЧЕСТИ ПРОЦЕССА
                tokio::spawn(metrics_loop(process, tx, state));
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
        Self::new().unwrap_or_else(|e| {
            tracing::error!("VpnEngine::new() failed: {e}; using noop engine");
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

async fn metrics_loop(
    process: Arc<PlatformProcess>,
    tx: mpsc::Sender<MetricsEvent>,
    state: Arc<Mutex<VpnState>>,
) {
    let mut last = MetricsSnapshot::default();
    let mut initialized = false;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // ПРОВЕРКА НА ПАДЕНИЕ XRAY (EXIT CODE 23 И ДР.)
        if !process.is_running().await {
            tracing::error!("Xray process crashed unexpectedly!");
            let mut s = state.lock().await;
            if s.is_active() {
                *s =
                    VpnState::Error("Xray crashed. Check geoip.dat/geosite.dat and config.".into());
                *s = VpnState::Disconnected;
            }
            break;
        }

        let snap = process.sample_metrics().await;
        let ev = if !initialized {
            last = snap;
            initialized = true;
            MetricsEvent {
                download_speed: 0,
                upload_speed: 0,
            }
        } else {
            let ev = MetricsEvent {
                download_speed: snap.download_speed.saturating_sub(last.download_speed),
                upload_speed: snap.upload_speed.saturating_sub(last.upload_speed),
            };
            last = snap;
            ev
        };
        if tx.send(ev).await.is_err() {
            break;
        }
    }
}
