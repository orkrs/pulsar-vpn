//! Android-бэкенд: использует `pulsar_daemon::VpnEngine` напрямую.
//!
//! На Android нет смысла (и невозможно) запускать отдельный процесс
//! `pulsar-daemon` — Tauri-UI и есть приложение. Поэтому:
//! - Создаём [`VpnEngine`] один раз при старте.
//! - `connect`/`disconnect` — прямые вызовы движка.
//! - Метрики идут через тот же `mpsc::Receiver<MetricsEvent>`,
//!   что и на Windows, но без gRPC-стрима и без реконнектов.
//!
//! ВАЖНО: реальная имплементация VpnService + NDK sing-box пока не
//! сделана (см. `pulsar-daemon/src/vpn_engine/process/android.rs`).
//! Сейчас `connect()` возвращает осмысленную ошибку, а UI может
//! рендерить состояние "Not implemented on Android yet" в заглушке.

use tokio::sync::mpsc;

use pulsar_daemon::vpn_engine::{ConnectRequest, MetricsEvent, VpnState};

pub struct AndroidBackend {
    engine: VpnEngine,
    metrics_rx: tokio::sync::Mutex<Option<mpsc::Receiver<MetricsEvent>>>,
}

impl AndroidBackend {
    pub fn new() -> Result<Self, String> {
        let engine = VpnEngine::new()?;
        Ok(Self {
            engine,
            metrics_rx: tokio::sync::Mutex::new(None),
        })
    }

    pub async fn connect(&self, req: ConnectRequest) -> Result<VpnState, String> {
        self.engine.connect(req).await
    }

    pub async fn disconnect(&self) -> Result<VpnState, String> {
        self.engine.disconnect().await
    }

    pub async fn state(&self) -> VpnState {
        self.engine.state().await
    }

    pub async fn subscribe_metrics(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        self.engine.subscribe_metrics().await
    }

    pub async fn shutdown(&self) {
        let _ = self.engine.disconnect().await;
    }
}

// Используется через Backend::state() (enum-dispatch)
#![allow(dead_code)]

use pulsar_daemon::vpn_engine::VpnEngine;
