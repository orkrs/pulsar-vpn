```
Cargo.toml
```
[workspace]
members = [
    "pulsar-shared",
    "pulsar-daemon",
    "pulsar-ui/src-tauri",
    "xtask",
]
# РџРѕ СѓРјРѕР»С‡Р°РЅРёСЋ cargo build/check/clippy СЂР°Р±РѕС‚Р°СЋС‚ С‚РѕР»СЊРєРѕ СЃ РїСЂРѕРґСѓРєС‚РѕРІС‹РјРё РєСЂРµР№С‚Р°РјРё.
# `xtask` РїРѕРґРєР»СЋС‡Р°РµС‚СЃСЏ СЏРІРЅРѕ: `cargo build -p xtask` РёР»Рё `cargo xtask <sub>`.
default-members = [
    "pulsar-shared",
    "pulsar-daemon",
    "pulsar-ui/src-tauri",
]
resolver = "2"

# РћР±С‰РёРµ РјРµС‚Р°РґР°РЅРЅС‹Рµ вЂ” СЃРј. https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table
[workspace.package]
version      = "0.1.0"
authors      = ["yann1n"]
license      = "MIT"
repository   = ""
edition      = "2024"
rust-version = "1.85"

[profile.release]
opt-level     = 3
lto           = "thin"
codegen-units = 1
strip         = "symbols"
# panic = "abort"  СЂР°СЃРєРѕРјРјРµРЅС‚РёСЂСѓР№С‚Рµ, РµСЃР»Рё РіРѕС‚РѕРІС‹ РїРѕР¶РµСЂС‚РІРѕРІР°С‚СЊ СЂР°СЃРєСЂСѓС‚РєРѕР№ СЃС‚РµРєР° СЂР°РґРё ~5% СЂР°Р·РјРµСЂР°.

[profile.dev]
opt-level = 1
# Р’ dev РїСЂРѕС„РёР»Рµ LTO РѕС‚РєР»СЋС‡С‘РЅ РїРѕ СѓРјРѕР»С‡Р°РЅРёСЋ вЂ” РѕСЃС‚Р°РІР»СЏРµРј С‚Р°Рє, С‡С‚РѕР±С‹ РёРЅРєСЂРµРјРµРЅС‚Р°Р»СЊРЅС‹Рµ СЃР±РѕСЂРєРё
# РѕСЃС‚Р°РІР°Р»РёСЃСЊ Р±С‹СЃС‚СЂС‹РјРё. Р’ release РІС‹С€Рµ вЂ” LTO thin.
```
pulsar-daemon\Cargo.toml
```
[package]
name = "pulsar-daemon"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Pulsar VPN backend daemon (gRPC server on Windows, in-process engine on Android)"

[lib]
name = "pulsar_daemon"
path = "src/lib.rs"

[[bin]]
name = "pulsar-daemon"
path = "src/main.rs"

[features]
# Р’РєР»СЋС‡Р°РµС‚СЃСЏ, РєРѕРіРґР° Р±РёР±Р»РёРѕС‚РµРєР° РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ РІ Android-СЃР±РѕСЂРєРµ.
# РћС‚РєР»СЋС‡Р°РµС‚ gRPC-СЃРїРµС†РёС„РёС‡РЅС‹Рµ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё (tokio full + tonic РІСЃС‘ СЂР°РІРЅРѕ
# РѕСЃС‚Р°СЋС‚СЃСЏ, РїРѕС‚РѕРјСѓ С‡С‚Рѕ VpnEngine async, РЅРѕ СЌРєРѕРЅРѕРјРёС‚ РјРµСЃС‚Рѕ РЅР° РјРѕР±РёР»СЊРЅРѕР№ СЃР±РѕСЂРєРµ).
default = []

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
tonic = "0.10"
sysinfo = "0.29"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
url = "2.4"
anyhow = "1.0"
thiserror = "1.0"
base64 = "0.21"
reqwest = { version = "0.11", default-features = false, features = ["blocking", "json", "rustls-tls"] }
pulsar-shared = { path = "../pulsar-shared" }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "time"] }
tracing-appender = "0.2"
time = { version = "0.3", features = ["formatting"] }

[target.'cfg(target_os = "android")'.dependencies]
# РќР° Android Р±СѓРґРµРј Р·РІР°С‚СЊ android.net.VpnService Рё TrafficStats С‡РµСЂРµР· JNI.
# ndk-context РїРѕР·РІРѕР»СЏРµС‚ РґРѕСЃС‚Р°С‚СЊ JavaVM РёР· NativeActivity.
# РљРѕРіРґР° РїСЂРёРґС‘С‚ РІСЂРµРјСЏ СЂРµР°Р»СЊРЅРѕР№ РёРјРїР»РµРјРµРЅС‚Р°С†РёРё, РґРѕР±Р°РІРёРј СЃСЋРґР° РєРѕРЅРєСЂРµС‚РЅС‹Рµ РєСЂРµР№С‚С‹.
android_logger = "0.14"
jni = "0.21"
ndk-context = "0.1"
```
pulsar-daemon\src\lib.rs
```
//! Pulsar VPN daemon вЂ” РїРµСЂРµРёСЃРїРѕР»СЊР·СѓРµРјРѕРµ СЏРґСЂРѕ.
//!
//! Р­С‚РѕС‚ crate РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµС‚ Р±РёР·РЅРµСЃ-Р»РѕРіРёРєСѓ VPN РІ РІРёРґРµ Р±РёР±Р»РёРѕС‚РµРєРё, РєРѕС‚РѕСЂР°СЏ
//! РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ РІ РґРІСѓС… СЃС†РµРЅР°СЂРёСЏС…:
//!
//! 1. **Windows**: Р±РёРЅР°СЂСЊ `pulsar-daemon` РїРѕРґРЅРёРјР°РµС‚ gRPC-СЃРµСЂРІРµСЂ РЅР°
//!    `127.0.0.1:50051` Рё РѕР±РѕСЂР°С‡РёРІР°РµС‚ [`VpnEngine`] РІ tonic-СЃРµСЂРІРёСЃ.
//!    Tauri-РїСЂРёР»РѕР¶РµРЅРёРµ РЅР° Windows РѕР±С‰Р°РµС‚СЃСЏ СЃ РЅРёРј РїРѕ gRPC.
//!
//! 2. **Android**: Tauri-РїСЂРёР»РѕР¶РµРЅРёРµ Р»РёРЅРєСѓРµС‚СЃСЏ СЃ СЌС‚РѕР№ Р±РёР±Р»РёРѕС‚РµРєРѕР№
//!    РЅР°РїСЂСЏРјСѓСЋ (С‡РµСЂРµР· cargo feature `android-engine`) Рё РёСЃРїРѕР»СЊР·СѓРµС‚
//!    [`VpnEngine`] in-process, Р±РµР· gRPC. РќР° Android РґРІРёР¶РѕРє СЂР°Р±РѕС‚Р°РµС‚
//!    С‡РµСЂРµР· Android `VpnService` + РЅР°С‚РёРІРЅС‹Р№ Xray, СЃРј.
//!    [`vpn_engine::android`].
//!
//! РџСѓР±Р»РёС‡РЅС‹Р№ API Р±РёР±Р»РёРѕС‚РµРєРё РЅР°РјРµСЂРµРЅРЅРѕ СѓР·РєРёР№:
//! - [`config_gen`] вЂ” РіРµРЅРµСЂР°С†РёСЏ JSON-РєРѕРЅС„РёРіР° Xray
//! - [`vpn_engine`] вЂ” РѕСЃРЅРѕРІРЅРѕР№ РґРІРёР¶РѕРє (СЃРѕСЃС‚РѕСЏРЅРёРµ, РїСЂРѕС†РµСЃСЃ, РјРµС‚СЂРёРєРё)
//! - [`logger`] вЂ” РёРЅРёС†РёР°Р»РёР·Р°С†РёСЏ Р»РѕРіРёСЂРѕРІР°РЅРёСЏ (РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ Р±РёРЅР°СЂС‘Рј)
//!
//! Р­С‚РёС… С‚СЂС‘С… РјРѕРґСѓР»РµР№ РґРѕСЃС‚Р°С‚РѕС‡РЅРѕ Рё gRPC-РѕР±С‘СЂС‚РєРµ, Рё Р±СѓРґСѓС‰РµРјСѓ Android-РёРЅС‚РµСЂС„РµР№СЃСѓ.

pub mod config_gen;
pub mod logger;
pub mod vpn_engine;
```
pulsar-daemon\src\logger.rs
```
use std::path::PathBuf;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// РРЅРёС†РёР°Р»РёР·Р°С†РёСЏ Р»РѕРіРёСЂРѕРІР°РЅРёСЏ РґР»СЏ РґРµРјРѕРЅР°
///
/// РќР°СЃС‚СЂР°РёРІР°РµС‚:
/// - Р РѕС‚Р°С†РёСЋ Р»РѕРіРѕРІ РїРѕ РґРЅСЏРј РІ %LOCALAPPDATA%\PulsarVPN\logs\
/// - Р¤РѕСЂРјР°С‚: timestamp (РјСЃ), СѓСЂРѕРІРµРЅСЊ, РјРѕРґСѓР»СЊ, С„Р°Р№Р»:СЃС‚СЂРѕРєР°, thread, СЃРѕРѕР±С‰РµРЅРёРµ
/// - РЈСЂРѕРІРµРЅСЊ РёР· PULSAR_LOG_LEVEL РёР»Рё РїРѕ СѓРјРѕР»С‡Р°РЅРёСЋ INFO
/// - РћС‡РёСЃС‚РєСѓ Р»РѕРіРѕРІ СЃС‚Р°СЂС€Рµ 30 РґРЅРµР№
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    // Р РѕС‚Р°С†РёСЏ РїРѕ РґРЅСЏРј: С„Р°Р№Р»С‹ РІРёРґР° pulsar-daemon-2025-02-22.log
    let file_appender = rolling::daily(&log_dir, "pulsar-daemon");
    let (non_blocking_appender, _guard) = non_blocking(file_appender);

    // Р¤РѕСЂРјР°С‚: timestamp (РјСЃ), СѓСЂРѕРІРµРЅСЊ, РјРѕРґСѓР»СЊ, С„Р°Р№Р»:СЃС‚СЂРѕРєР°, thread, СЃРѕРѕР±С‰РµРЅРёРµ
    let file_layer = fmt::Layer::default()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE)
        .boxed();

    // РљРѕРЅСЃРѕР»СЊРЅС‹Р№ РІС‹РІРѕРґ (РґР»СЏ РѕС‚Р»Р°РґРєРё)
    let console_layer = fmt::Layer::default()
        .with_writer(std::io::stdout)
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .boxed();

    // РЈСЂРѕРІРµРЅСЊ Р»РѕРіРёСЂРѕРІР°РЅРёСЏ РёР· РїРµСЂРµРјРµРЅРЅРѕР№ РѕРєСЂСѓР¶РµРЅРёСЏ РёР»Рё РїРѕ СѓРјРѕР»С‡Р°РЅРёСЋ INFO
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    Registry::default()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    // Р—Р°РїСѓСЃРєР°РµРј РѕС‡РёСЃС‚РєСѓ СЃС‚Р°СЂС‹С… Р»РѕРіРѕРІ РІ С„РѕРЅРµ
    let log_dir_clone = log_dir.clone();
    tokio::spawn(async move {
        cleanup_old_logs(&log_dir_clone).await;
    });

    tracing::info!("Logger initialized. Logs directory: {:?}", log_dir);
    Ok(())
}

fn get_log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    Ok(PathBuf::from(local_app_data).join("PulsarVPN").join("logs"))
}

/// РћС‡РёСЃС‚РєР° Р»РѕРіРѕРІ СЃС‚Р°СЂС€Рµ 30 РґРЅРµР№
async fn cleanup_old_logs(log_dir: &PathBuf) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let thirty_days_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (30 * 24 * 60 * 60);

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(modified_secs) = modified.duration_since(UNIX_EPOCH) {
                        if modified_secs.as_secs() < thirty_days_ago {
                            if let Err(e) = std::fs::remove_file(entry.path()) {
                                tracing::warn!(
                                    "Failed to delete old log file {:?}: {}",
                                    entry.path(),
                                    e
                                );
                            } else {
                                tracing::debug!("Deleted old log file: {:?}", entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
}
```
pulsar-daemon\src\main.rs
```
//! Pulsar VPN daemon вЂ” Р±РёРЅР°СЂСЊ-РѕР±С‘СЂС‚РєР° РІРѕРєСЂСѓРі gRPC-СЃРµСЂРІРёСЃР° (С‚РѕР»СЊРєРѕ Windows).
//!
//! РќР° Windows `pulsar-daemon.exe` РїРѕРґРЅРёРјР°РµС‚ gRPC-СЃРµСЂРІРµСЂ РЅР° 127.0.0.1:50051
//! Рё РѕР±РѕСЂР°С‡РёРІР°РµС‚ [`VpnEngine`] РёР· `pulsar_daemon` РІ tonic-СЃРµСЂРІРёСЃ.
//!
//! РќР° Android СЌС‚РѕС‚ Р±РёРЅР°СЂСЊ РЅРµ РєРѕРјРїРёР»РёСЂСѓРµС‚СЃСЏ (`#[cfg(target_os = "windows")]`),
//! Рё Tauri-РїСЂРёР»РѕР¶РµРЅРёРµ РёСЃРїРѕР»СЊР·СѓРµС‚ Р±РёР±Р»РёРѕС‚РµРєСѓ РЅР°РїСЂСЏРјСѓСЋ.

#![cfg(target_os = "windows")]

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, transport::Server};

use pulsar_daemon::vpn_engine::{ConnectRequest, VpnEngine, VpnState};
use pulsar_shared::vpn::vpn_service_server::{VpnService, VpnServiceServer};
use pulsar_shared::vpn::{ConnectionRequest, Empty, Metrics, StatusResponse};

use pulsar_daemon::logger;
use pulsar_daemon::vpn_engine::MetricsEvent;

/// gRPC-РѕР±С‘СЂС‚РєР° РЅР°Рґ [`VpnEngine`].
pub struct MyVpnService {
    engine: VpnEngine,
    expected_token: Option<Arc<str>>,
    shutdown_tx: mpsc::Sender<()>,
}

fn check_token<T>(request: &Request<T>, expected: Option<&Arc<str>>) -> Result<(), Status> {
    let Some(token) = expected else { return Ok(()) };
    let got = request
        .metadata()
        .get("x-pulsar-token")
        .and_then(|v| v.to_str().ok());
    if got != Some(token.as_ref()) {
        return Err(Status::unauthenticated("invalid token"));
    }
    Ok(())
}

fn state_to_proto(s: &VpnState) -> (String, String) {
    match s {
        VpnState::Disconnected => ("Disconnected".into(), "OK".into()),
        VpnState::Connecting => ("Connecting".into(), "OK".into()),
        VpnState::Connected => ("Connected".into(), "OK".into()),
        VpnState::Disconnecting => ("Disconnecting".into(), "OK".into()),
        VpnState::Error(e) => ("Error".into(), e.clone()),
    }
}

#[tonic::async_trait]
impl VpnService for MyVpnService {
    type StreamMetricsStream = ReceiverStream<Result<Metrics, Status>>;

    async fn toggle_connection(
        &self,
        request: Request<ConnectionRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let req = request.into_inner();
        let routing = req.routing_config.unwrap_or_default();
        let engine_req = ConnectRequest {
            profile_link: req.profile_link,
            routing,
        };
        let res = if req.connect {
            self.engine.connect(engine_req).await
        } else {
            self.engine.disconnect().await
        };
        let (state, msg) = match res {
            Ok(s) => state_to_proto(&s),
            Err(e) => ("Error".into(), e),
        };
        Ok(Response::new(StatusResponse {
            state,
            message: msg,
        }))
    }

    async fn get_status(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<StatusResponse>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let (state, msg) = state_to_proto(&self.engine.state().await);
        Ok(Response::new(StatusResponse {
            state,
            message: msg,
        }))
    }

    async fn stream_metrics(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::StreamMetricsStream>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let (tx, rx) = mpsc::channel::<Result<Metrics, Status>>(4);
        let engine = self.engine.clone();
        tokio::spawn(async move {
            // РџРѕРґРїРёСЃС‹РІР°РµРјСЃСЏ РЅР° СЃРѕР±С‹С‚РёСЏ РјРµС‚СЂРёРє.
            let Some(mut metrics_rx) = engine.subscribe_metrics().await else {
                tracing::warn!("stream_metrics: no subscriber available");
                return;
            };
            while let Some(ev) = metrics_rx.recv().await {
                let MetricsEvent {
                    download_speed,
                    upload_speed,
                } = ev;
                let m = Metrics {
                    download_speed,
                    upload_speed,
                };
                if tx.send(Ok(m)).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn shutdown(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let _ = self.engine.disconnect().await;
        let _ = self.shutdown_tx.send(()).await;
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init_logger()?;
    let addr = "127.0.0.1:50051".parse()?;
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
    let expected_token = std::env::args().nth(1).map(Arc::from);

    let engine = VpnEngine::new()?;
    let service = MyVpnService {
        engine,
        expected_token,
        shutdown_tx,
    };

    tracing::info!("pulsar-daemon listening on {addr}");
    Server::builder()
        .add_service(VpnServiceServer::new(service))
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.recv().await;
            tracing::info!("pulsar-daemon shutting down");
        })
        .await?;
    Ok(())
}
```
pulsar-daemon\src\vpn_engine\mod.rs
```
//! VPN-РґРІРёР¶РѕРє: Р±РёР·РЅРµСЃ-Р»РѕРіРёРєР° РїРѕРґРєР»СЋС‡РµРЅРёСЏ, СЃРѕСЃС‚РѕСЏРЅРёРµ, РјРµС‚СЂРёРєРё.
//!
//! РђСЂС…РёС‚РµРєС‚СѓСЂР°:
//! - [`VpnEngine`] вЂ” РµРґРёРЅС‹Р№ API, РЅРµ Р·Р°РІРёСЃСЏС‰РёР№ РѕС‚ РїР»Р°С‚С„РѕСЂРјС‹. Р РµР°Р»РёР·СѓРµС‚СЃСЏ
//!   С‡РµСЂРµР· [`PlatformProcess`], РєРѕС‚РѕСЂС‹Р№ РІС‹Р±РёСЂР°РµС‚СЃСЏ РЅР° СЌС‚Р°РїРµ РєРѕРјРїРёР»СЏС†РёРё
//!   (`#[cfg(target_os = "вЂ¦")]`).
//! - РќР° Windows РїСЂРѕС†РµСЃСЃ Xray Р·Р°РїСѓСЃРєР°РµС‚СЃСЏ РєР°Рє РґРѕС‡РµСЂРЅРёР№.
//! - РќР° Android РґРІРёР¶РѕРє РґРѕР»Р¶РµРЅ СЂР°Р±РѕС‚Р°С‚СЊ С‡РµСЂРµР· `android.net.VpnService`
//!   (СЃРј. Р·Р°РіР»СѓС€РєСѓ РІ [`android`]).
//!
//! Р’СЃРµ async-РјРµС‚РѕРґС‹ вЂ” `Send + Sync`, С‡С‚РѕР±С‹ РґРІРёР¶РѕРє РјРѕР¶РЅРѕ Р±С‹Р»Рѕ С€Р°СЂРёС‚СЊ
//! РјРµР¶РґСѓ Tauri-РєРѕРјР°РЅРґР°РјРё С‡РµСЂРµР· `tauri::State`.

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use pulsar_shared::vpn::RoutingConfig;

use crate::config_gen;

/// РС‚РѕРіРѕРІРѕРµ СЃРѕСЃС‚РѕСЏРЅРёРµ, РїРѕРЅСЏС‚РЅРѕРµ UI.
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

/// РЎРЅРёРјРѕРє РјРµС‚СЂРёРє СЃРєРѕСЂРѕСЃС‚Рё (Р±Р°Р№С‚/СЃ).
#[derive(Debug, Clone, Copy, Default)]
pub struct MetricsSnapshot {
    pub download_speed: u64,
    pub upload_speed: u64,
}

/// Р—Р°РїСЂРѕСЃ РЅР° РїРѕРґРєР»СЋС‡РµРЅРёРµ.
#[derive(Debug, Clone)]
pub struct ConnectRequest {
    pub profile_link: String,
    pub routing: RoutingConfig,
}

/// РЎРѕР±С‹С‚РёРµ РјРµС‚СЂРёРє, РєРѕС‚РѕСЂРѕРµ UI РїРѕР»СѓС‡Р°РµС‚ С‡РµСЂРµР· РїРѕРґРїРёСЃРєСѓ.
#[derive(Debug, Clone, Copy)]
pub struct MetricsEvent {
    pub download_speed: u64,
    pub upload_speed: u64,
}

/// РђР±СЃС‚СЂР°РєС†РёСЏ РЅР°Рґ "РїСЂРѕС†РµСЃСЃРѕРј Xray / VpnService" РґР»СЏ С‚РµРєСѓС‰РµР№ РїР»Р°С‚С„РѕСЂРјС‹.
///
/// Р РµР°Р»РёР·Р°С†РёСЏ РїРѕРґР±РёСЂР°РµС‚СЃСЏ РЅР° СЌС‚Р°РїРµ РєРѕРјРїРёР»СЏС†РёРё вЂ” СЃРј. [`process`].
mod process;
pub use process::PlatformProcess;

/// Р“Р»Р°РІРЅС‹Р№ VPN-РґРІРёР¶РѕРє. Р”РµСЂР¶РёС‚ РѕР±С‰РµРµ СЃРѕСЃС‚РѕСЏРЅРёРµ Рё РїР»Р°С‚С„РѕСЂРјРµРЅРЅС‹Р№ РїСЂРѕС†РµСЃСЃ.
///
/// РЁР°Р±Р»РѕРЅ РёСЃРїРѕР»СЊР·РѕРІР°РЅРёСЏ (РѕРґРёРЅР°РєРѕРІ РЅР° Win Рё Android):
/// ```no_run
/// # use pulsar_daemon::vpn_engine::{VpnEngine, ConnectRequest, MetricsEvent};
/// # async fn demo(engine: VpnEngine) -> Result<(), String> {
/// engine.connect(ConnectRequest { profile_link: "vless://вЂ¦".into(), routing: Default::default() }).await?;
/// let Some(mut metrics) = engine.subscribe_metrics().await else { return Ok(()) };
/// while let Some(m) = metrics.recv().await {
///     println!("в†“{} в†‘{}", m.download_speed, m.upload_speed);
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
    /// РЎРѕР·РґР°С‘С‚ РЅРѕРІС‹Р№ РґРІРёР¶РѕРє. РќР° Windows РёС‰РµС‚ `xray(.exe)` СЂСЏРґРѕРј СЃ С‚РµРєСѓС‰РёРј
    /// Р±РёРЅР°СЂС‘Рј (РёР»Рё РІ PATH). РќР° Android С‚СЂРµР±СѓРµС‚ JNI-РёРЅРёС†РёР°Р»РёР·Р°С†РёРё (СЃРј. android.rs).
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

    /// РџРѕРґРїРёСЃР°С‚СЊСЃСЏ РЅР° СЃС‚СЂРёРј РјРµС‚СЂРёРє. Р’РѕР·РІСЂР°С‰Р°РµС‚ Receiver, РєРѕС‚РѕСЂС‹Р№ РЅСѓР¶РЅРѕ
    /// РїРµСЂРёРѕРґРёС‡РµСЃРєРё РѕРїСЂР°С€РёРІР°С‚СЊ (РЅР°РїСЂРёРјРµСЂ, РІ tauri::async_runtime::spawn).
    pub async fn subscribe_metrics(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        self.inner.metrics_rx.lock().await.take()
    }

    /// РџРѕРґРєР»СЋС‡РёС‚СЊСЃСЏ Рє VPN-СѓР·Р»Сѓ.
    pub async fn connect(&self, req: ConnectRequest) -> Result<VpnState, String> {
        // РџРѕРјРµС‡Р°РµРј СЃРѕСЃС‚РѕСЏРЅРёРµ Connecting Р°С‚РѕРјР°СЂРЅРѕ (РїСЂРѕРІРµСЂСЏРµРј, С‡С‚Рѕ РЅРµ РїРѕРґРєР»СЋС‡РµРЅС‹).
        {
            let mut s = self.inner.state.lock().await;
            if s.is_active() {
                return Err("СѓР¶Рµ РїРѕРґРєР»СЋС‡РµРЅРѕ РёР»Рё РёРґС‘С‚ РїРѕРґРєР»СЋС‡РµРЅРёРµ".into());
            }
            *s = VpnState::Connecting;
        }

        // Р“РµРЅРµСЂРёСЂСѓРµРј РєРѕРЅС„РёРі Xray. Р­С‚Рѕ РѕР±С‰Р°СЏ Р»РѕРіРёРєР° РґР»СЏ РѕР±РµРёС… РїР»Р°С‚С„РѕСЂРј.
        let config_json = config_gen::generate_xray_config(&req.profile_link, &req.routing)
            .map_err(|e| {
                let err = format!("config generation failed: {e:#}");
                // Р’РѕР·РІСЂР°С‰Р°РµРј СЃРѕСЃС‚РѕСЏРЅРёРµ РІ Disconnected Р°СЃРёРЅС…СЂРѕРЅРЅРѕ, С‡С‚РѕР±С‹ РЅРµ РґРµСЂР¶Р°С‚СЊ lock.
                let this = self.clone();
                let err_for_state = err.clone();
                tokio::spawn(async move {
                    let mut s = this.inner.state.lock().await;
                    *s = VpnState::Error(err_for_state);
                    *s = VpnState::Disconnected;
                });
                err
            })?;

        // Р—Р°РїСѓСЃРєР°РµРј РїР»Р°С‚С„РѕСЂРјРµРЅРЅС‹Р№ РїСЂРѕС†РµСЃСЃ.
        match self.inner.process.start(&config_json).await {
            Ok(()) => {
                *self.inner.state.lock().await = VpnState::Connected;
                // Р—Р°РїСѓСЃРєР°РµРј С„РѕРЅРѕРІС‹Р№ СЃР±РѕСЂС‰РёРє РјРµС‚СЂРёРє.
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

    /// РћС‚РєР»СЋС‡РёС‚СЊСЃСЏ РѕС‚ VPN.
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
        // Р”Р»СЏ С‚РµСЃС‚РѕРІ / previews, РіРґРµ РЅРµ РЅСѓР¶РµРЅ СЂРµР°Р»СЊРЅС‹Р№ РїСЂРѕС†РµСЃСЃ.
        Self::new().unwrap_or_else(|e| {
            tracing::error!("VpnEngine::new() failed: {e}; using noop engine");
            // Р—Р°РїР°СЃРЅРѕР№ РІР°СЂРёР°РЅС‚ вЂ” СЃРѕРѕР±С‰Р°РµРј РѕР± РѕС€РёР±РєРµ РІ Р»СЋР±РѕРј connect().
            // Р‘РµР·РѕРїР°СЃРЅРµРµ, С‡РµРј РїР°РґР°С‚СЊ, РїРѕС‚РѕРјСѓ С‡С‚Рѕ Tauri-РёРЅРёС†РёР°Р»РёР·Р°С†РёСЏ РЅРµ РґРѕР»Р¶РЅР°
            // Р»РѕРјР°С‚СЊСЃСЏ, РґР°Р¶Рµ РµСЃР»Рё Xray РѕС‚СЃСѓС‚СЃС‚РІСѓРµС‚.
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

/// РџРµСЂРёРѕРґРёС‡РµСЃРєРёР№ СЃР±РѕСЂ РјРµС‚СЂРёРє СЃ РїР»Р°С‚С„РѕСЂРјРµРЅРЅРѕРіРѕ РїСЂРѕС†РµСЃСЃР°.
/// РќР° Windows СЌС‚Рѕ sysinfo + С„РёР»СЊС‚СЂ РїРѕ TUN-РёРЅС‚РµСЂС„РµР№СЃСѓ.
/// РќР° Android вЂ” С‡С‚РµРЅРёРµ СЃС‡С‘С‚С‡РёРєРѕРІ РёР· VpnService С‡РµСЂРµР· JNI.
async fn metrics_loop(process: Arc<PlatformProcess>, tx: mpsc::Sender<MetricsEvent>) {
    let mut last = MetricsSnapshot::default();
    let mut initialized = false;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
            // Receiver РѕС‚РІР°Р»РёР»СЃСЏ вЂ” UI РѕС‚РїРёСЃР°Р»СЃСЏ.
            break;
        }
    }
}
```
pulsar-daemon\src\vpn_engine\process.rs
```
//! РџР»Р°С‚С„РѕСЂРјРµРЅРЅР°СЏ Р°Р±СЃС‚СЂР°РєС†РёСЏ РЅР°Рґ VPN-РїСЂРѕС†РµСЃСЃРѕРј.
//!
//! РќР° Windows: РїРѕСЂРѕР¶РґР°РµС‚ РґРѕС‡РµСЂРЅРёР№ РїСЂРѕС†РµСЃСЃ `xray(.exe)`, С‡РёС‚Р°РµС‚ РµРіРѕ
//! stdout/stderr, РѕРїСЂР°С€РёРІР°РµС‚ СЃРµС‚РµРІС‹Рµ РёРЅС‚РµСЂС„РµР№СЃС‹ С‡РµСЂРµР· `sysinfo`.
//!
//! РќР° Android: РЅР°С‚РёРІРЅС‹Р№ Xray Р·Р°РїСѓСЃРєР°РµС‚СЃСЏ РІ Р°РґСЂРµСЃРЅРѕРј РїСЂРѕСЃС‚СЂР°РЅСЃС‚РІРµ
//! СЃР°РјРѕРіРѕ РїСЂРёР»РѕР¶РµРЅРёСЏ (С‡РµСЂРµР· JNI), Р° С‚СЂР°С„РёРє РїРµСЂРµС…РІР°С‚С‹РІР°РµС‚СЃСЏ С‡РµСЂРµР·
//! Android `VpnService` (СЃРј. `android.rs`). Р­С‚РѕС‚ РјРѕРґСѓР»СЊ РЅР° Android
//! СЃРµР№С‡Р°СЃ вЂ” Р·Р°РіР»СѓС€РєР°, Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅРЅР°СЏ РІ [`android`].

use std::path::PathBuf;
use std::sync::Arc;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
pub use android::PlatformProcessImpl;
#[cfg(target_os = "windows")]
pub use windows::PlatformProcessImpl;

/// РўРёРї-РѕР±С‘СЂС‚РєР°: РЅР° РєР°Р¶РґРѕР№ РїР»Р°С‚С„РѕСЂРјРµ СЃРІРѕСЏ СЂРµР°Р»РёР·Р°С†РёСЏ, РїРѕРґРєР»СЋС‡Р°РµРјР°СЏ С‡РµСЂРµР· `mod`.
/// РСЃРїРѕР»СЊР·СѓРµРј РЅРѕРІС‹Р№-С‚РёРї РїРѕРІРµСЂС… `Arc<PlatformProcessImpl>`, С‡С‚РѕР±С‹ СЂРµР°Р»РёР·Р°С†РёРё
/// РјРѕРіР»Рё РёРјРµС‚СЊ СЂР°Р·РЅС‹Рµ РЅР°Р±РѕСЂС‹ РїРѕР»РµР№ Рё РѕР±РѕСЂР°С‡РёРІР°С‚СЊСЃСЏ РІ Arc РµРґРёРЅРѕРѕР±СЂР°Р·РЅРѕ.
pub struct PlatformProcess(Arc<PlatformProcessImpl>);

impl std::ops::Deref for PlatformProcess {
    type Target = PlatformProcessImpl;
    fn deref(&self) -> &PlatformProcessImpl {
        &self.0
    }
}

impl Clone for PlatformProcess {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PlatformProcess {
    pub fn new() -> Result<Self, String> {
        Ok(Self(Arc::new(PlatformProcessImpl::new()?)))
    }

    /// Р—Р°РіР»СѓС€РєР° РґР»СЏ СЃР»СѓС‡Р°СЏ, РєРѕРіРґР° `VpnEngine::new()` РЅРµ СЃРјРѕРі РЅР°Р№С‚Рё
    /// Xray / VpnService. Р’СЃРµ РІС‹Р·РѕРІС‹ РІРѕР·РІСЂР°С‰Р°СЋС‚ РѕС€РёР±РєРё.
    pub fn noop() -> Self {
        Self(Arc::new(PlatformProcessImpl::noop()))
    }

    /// Р“РґРµ Р»РµР¶РёС‚ Xray РЅР° С‚РµРєСѓС‰РµР№ РїР»Р°С‚С„РѕСЂРјРµ (РґР»СЏ UI-РґРёР°РіРЅРѕСЃС‚РёРєРё).
    pub fn binary_path(&self) -> Option<PathBuf> {
        self.0.binary_path()
    }
}
```
pulsar-daemon\src\vpn_engine\process\windows.rs
```
//! Windows implementation: child Xray process plus network metrics.

use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::{NetworkExt, System, SystemExt};
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::vpn_engine::MetricsSnapshot;

/// РћС€РёР±РєР° РїР»Р°С‚С„РѕСЂРјРµРЅРЅРѕР№ РѕРїРµСЂР°С†РёРё.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Xray not found: {0}")]
    XrayMissing(PathBuf),
    #[error("failed to start Xray: {0}")]
    Spawn(String),
    #[error("РѕС€РёР±РєР° РІРІРѕРґР°-РІС‹РІРѕРґР°: {0}")]
    Io(#[from] std::io::Error),
    #[error("Xray exited with code {0}")]
    Exited(i32),
}

/// Р РµР°Р»СЊРЅР°СЏ СЂРµР°Р»РёР·Р°С†РёСЏ РґР»СЏ Windows. Р”РµСЂР¶РёС‚ РґРѕС‡РµСЂРЅРёР№ `Child` РїРѕРґ `Mutex`,
/// С‡С‚РѕР±С‹ РєРѕРјР°РЅРґС‹ РєРѕРЅРєСѓСЂРµРЅС‚РЅРѕ РЅРµ РїР»РѕРґРёР»Рё РїСЂРѕС†РµСЃСЃС‹.
pub struct PlatformProcessImpl {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    child: Option<Child>,
    xray_path: PathBuf,
}

impl PlatformProcessImpl {
    pub fn new() -> Result<Self, String> {
        let xray_path = find_xray()
            .ok_or_else(|| "xray.exe not found next to the daemon or in PATH".to_string())?;
        Ok(Self {
            inner: Arc::new(Mutex::new(Inner {
                child: None,
                xray_path,
            })),
        })
    }

    pub fn noop() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                child: None,
                xray_path: PathBuf::new(),
            })),
        }
    }

    pub fn binary_path(&self) -> Option<PathBuf> {
        let g = self.inner.try_lock().ok()?;
        if g.xray_path.as_os_str().is_empty() {
            None
        } else {
            Some(g.xray_path.clone())
        }
    }

    pub async fn start(&self, config_json: &str) -> Result<(), PlatformError> {
        if !is_elevated() {
            return Err(PlatformError::Spawn(
                "Windows administrator rights are required to install the VPN routes".into(),
            ));
        }
        let mut g = self.inner.lock().await;
        if g.child.is_some() {
            return Ok(());
        } // СѓР¶Рµ Р·Р°РїСѓС‰РµРЅ

        // Р—Р°РїРёСЃС‹РІР°РµРј РєРѕРЅС„РёРі РІРѕ РІСЂРµРјРµРЅРЅС‹Р№ С„Р°Р№Р».
        let cfg_dir = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("PulsarVPN");
        let _ = std::fs::create_dir_all(&cfg_dir);
        let cfg_path = cfg_dir.join("config.json");
        tokio::fs::write(&cfg_path, config_json).await?;

        let mut cmd = Command::new(&g.xray_path);
        cmd.arg("run")
            .arg("-config")
            .arg(&cfg_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if let Some(directory) = g.xray_path.parent() {
            cmd.current_dir(directory);
        }

        #[cfg(target_os = "windows")]
        {
            // CREATE_NO_WINDOW вЂ” С‡С‚РѕР±С‹ РЅРµ РјР°СЏС‡РёР»Рѕ РєРѕРЅСЃРѕР»СЊРЅРѕРµ РѕРєРЅРѕ.
            // tokio::process::Command РЅР° Windows РЅР°С‚РёРІРЅРѕ РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµС‚
            // РјРµС‚РѕРґ creation_flags (0x08000000), РѕС‚РґРµР»СЊРЅС‹Р№ use РЅРµ РЅСѓР¶РµРЅ.
            cmd.creation_flags(0x08000000);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| PlatformError::Spawn(e.to_string()))?;

        // РџРµСЂРµС…РІР°С‚С‹РІР°РµРј Р»РѕРіРё РІ tracing.
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut lines = tokio::io::BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!(target: "xray", "{line}");
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut lines = tokio::io::BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(target: "xray", "{line}");
                }
            });
        }

        // РќРµР±РѕР»СЊС€Р°СЏ Р·Р°РґРµСЂР¶РєР° вЂ” РµСЃР»Рё РїСЂРѕС†РµСЃСЃ СЃСЂР°Р·Сѓ СѓРїР°Р», Р·Р°РјРµС‚РёРј.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Ok(Some(status)) = child.try_wait() {
            return Err(PlatformError::Exited(status.code().unwrap_or(-1)));
        }
        g.child = Some(child);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), PlatformError> {
        let mut g = self.inner.lock().await;
        if let Some(mut child) = g.child.take() {
            // РЎРЅР°С‡Р°Р»Р° taskkill (СѓР±РёРІР°РµС‚ Рё РґРѕС‡РµСЂРЅРёРµ РїСЂРѕС†РµСЃСЃС‹), Р·Р°С‚РµРј kill().
            #[cfg(target_os = "windows")]
            if let Some(pid) = child.id() {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .output();
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        Ok(())
    }

    pub async fn sample_metrics(&self) -> MetricsSnapshot {
        // Р”РµР»Р°РµРј СЂР°Р· РІ СЃРµРєСѓРЅРґСѓ, refresh_networks() РґРµС€С‘РІС‹Р№ (С‚РѕР»СЊРєРѕ РґР°РЅРЅС‹Рµ
        // РїРѕ СѓР¶Рµ-РёР·РІРµСЃС‚РЅС‹Рј РёРЅС‚РµСЂС„РµР№СЃР°Рј), refresh_networks_list() вЂ” РґРѕСЂРѕРіРѕР№,
        // Р·РѕРІС‘Рј С‚РѕР»СЊРєРѕ РїСЂРё СЃРјРµРЅРµ СЃРѕСЃС‚РѕСЏРЅРёСЏ РїРѕРґРєР»СЋС‡РµРЅРёСЏ.
        // Р—РґРµСЃСЊ РјС‹ РІРЅСѓС‚СЂРё РјРµС‚СЂРёРє-С†РёРєР»Р°, Рё СЃРѕСЃС‚РѕСЏРЅРёРµ РЅР°СЃ РЅРµ РёРЅС‚РµСЂРµСЃСѓРµС‚ вЂ”
        // РµСЃР»Рё TUN РЅРµС‚, РІРµСЂРЅС‘Рј РЅСѓР»Рё (UI РёС… Рё С‚Р°Рє СЂРёСЃСѓРµС‚ РїСЂРё Disconnected).
        use std::sync::atomic::{AtomicBool, Ordering};
        static INIT: AtomicBool = AtomicBool::new(false);
        if !INIT.swap(true, Ordering::Relaxed) {
            self.refresh_list();
        }
        self.refresh_data()
    }

    fn refresh_list(&self) {
        // sysinfo вЂ” sync API, РґРµР»Р°РµРј РІ spawn_blocking, С‡С‚РѕР±С‹ РЅРµ Р±Р»РѕРєРёСЂРѕРІР°С‚СЊ
        // tokio executor (РЅР° Windows refresh_networks_list РґС‘СЂРіР°РµС‚ WMI).
        std::mem::drop(tokio::task::spawn_blocking(|| {
            let mut s = System::new_all();
            s.refresh_networks_list();
        }));
    }

    fn refresh_data(&self) -> MetricsSnapshot {
        // РўРѕР¶Рµ С‡РµСЂРµР· spawn_blocking, РїРѕС‚РѕРјСѓ С‡С‚Рѕ sysinfo вЂ” sync.
        tokio::task::block_in_place(|| {
            let mut s = System::new_all();
            s.refresh_networks();
            let mut td = 0u64;
            let mut tu = 0u64;
            for (name, data) in s.networks() {
                if name.to_lowercase().contains("pulsar") {
                    td += data.total_received();
                    tu += data.total_transmitted();
                }
            }
            MetricsSnapshot {
                download_speed: td,
                upload_speed: tu,
            }
        })
    }
}

/// РС‰РµС‚ Xray СЂСЏРґРѕРј СЃ С‚РµРєСѓС‰РёРј Р±РёРЅР°СЂС‘Рј, РїРѕС‚РѕРј РІ PATH.
fn find_xray() -> Option<PathBuf> {
    let candidates = [
        // СЂСЏРґРѕРј СЃ pulsar-daemon.exe вЂ” СЃС‚Р°РЅРґР°СЂС‚РЅР°СЏ СЂР°СЃРєР»Р°РґРєР°
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("xray.exe"))),
        Some(PathBuf::from("xray.exe")),
    ];
    for c in candidates.into_iter().flatten() {
        if c.exists() {
            return Some(c);
        }
    }
    which_xray()
}

#[cfg(target_os = "windows")]
fn which_xray() -> Option<PathBuf> {
    // РњРёРЅРёРјР°Р»СЊРЅС‹Р№ which вЂ” Р±РµР· Р»РёС€РЅРµР№ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё.
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let p = dir.join("xray.exe");
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

fn is_elevated() -> bool {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("whoami.exe")
        .args(["/groups", "/fo", "csv"])
        .creation_flags(0x0800_0000)
        .output()
        .map(|output| {
            let groups = String::from_utf8_lossy(&output.stdout);
            groups.contains("S-1-16-12288") || groups.contains("S-1-16-16384")
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn which_xray() -> Option<PathBuf> {
    None
}
```
pulsar-shared\Cargo.toml
```
[package]
name = "pulsar-shared"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true


[dependencies]
tonic = "0.10.2"
prost = "0.12"

[build-dependencies]
tonic-build = "0.10.2"
```
pulsar-shared\proto\vpn.proto
```
syntax = "proto3";
package vpn;

service VpnService {
  rpc ToggleConnection (ConnectionRequest) returns (StatusResponse);
  rpc GetStatus (Empty) returns (StatusResponse);
  rpc StreamMetrics (Empty) returns (stream Metrics);

  // РђР РҐРРўР•РљРўРЈР РќРћР• РРЎРџР РђР’Р›Р•РќРР• (РџСѓРЅРєС‚ 4):
  // РњРµС‚РѕРґ РґР»СЏ РјСЏРіРєРѕРіРѕ Р·Р°РІРµСЂС€РµРЅРёСЏ РґРµРјРѕРЅР° (Graceful Shutdown).
  // Р’С‹Р·С‹РІР°РµС‚СЃСЏ РёР· Tauri РїСЂРё Р·Р°РєСЂС‹С‚РёРё РѕРєРЅР°, С‡С‚РѕР±С‹ РЅРµ РѕСЃС‚Р°РІР»СЏС‚СЊ Р·РѕРјР±Рё-РїСЂРѕС†РµСЃСЃС‹.
  rpc Shutdown (Empty) returns (Empty);
}

message Empty {}

// Р”РћР‘РђР’Р›Р•РќРћ: РџРµСЂРµС‡РёСЃР»РµРЅРёРµ С‚РёРїРѕРІ DNS РґР»СЏ РёРЅРґРёРІРёРґСѓР°Р»СЊРЅРѕР№ РЅР°СЃС‚СЂРѕР№РєРё РїСЂРѕС„РёР»РµР№
enum DnsType {
  DNS_HTTPS = 0;
  DNS_HTTP = 1;
  DNS_UDP = 2;
}

message RoutingConfig {
  enum Mode {
    GLOBAL = 0;
    BYPASS = 1;
    PROXY = 2;
  }
  Mode mode = 1;
  repeated string domains = 2;
  repeated string apps = 3;

  // Р”РћР‘РђР’Р›Р•РќРћ: РќР°СЃС‚СЂРѕР№РєРё GeoIP-РјР°СЂС€СЂСѓС‚РёР·Р°С†РёРё (РџСѓРЅРєС‚ 7)
  bool enable_geoip = 4;
  map<string, string> geoip_rules = 5; // РљР»СЋС‡: РєРѕРґ СЃС‚СЂР°РЅС‹ ("ru", "us"), Р—РЅР°С‡РµРЅРёРµ: РјР°СЂС€СЂСѓС‚ ("proxy", "direct", "block")

  // Р”РћР‘РђР’Р›Р•РќРћ: РРЅРґРёРІРёРґСѓР°Р»СЊРЅС‹Р№ С‚РёРї DNS
  DnsType dns_type = 6;

  // Р”РћР‘РђР’Р›Р•РќРћ: Р Р°Р·СЂРµС€РёС‚СЊ РЅРµР±РµР·РѕРїР°СЃРЅРѕРµ СЃРѕРµРґРёРЅРµРЅРёРµ (insecure TLS)
  bool allow_insecure = 7;

  // Р•СЃР»Рё false вЂ” С‚РѕР»СЊРєРѕ IPv4 (TUN, DNS, СЂРµР·РѕР»РІ СЃРµСЂРІРµСЂР°, dial). РџРѕ СѓРјРѕР»С‡Р°РЅРёСЋ РІС‹РєР».
  bool use_ipv6 = 8;
}

message ConnectionRequest {
  bool connect = 1;
  string profile_link = 2;
  RoutingConfig routing_config = 3;
}

message StatusResponse {
  string state = 1;
  string message = 2;
}

// РРЎРџР РђР’Р›Р•РќРР•: Р Р°Р·РґРµР»СЏРµРј РјРµС‚СЂРёРєРё РЅР° Down Рё Up
message Metrics {
  uint64 download_speed = 1; // РІ Р±Р°Р№С‚Р°С… РІ СЃРµРєСѓРЅРґСѓ
  uint64 upload_speed = 2;   // РІ Р±Р°Р№С‚Р°С… РІ СЃРµРєСѓРЅРґСѓ
}```
pulsar-shared\src\lib.rs
```
pub mod vpn {
    tonic::include_proto!("vpn");
}
```
pulsar-ui\index.html
```
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover" />
    <meta name="theme-color" content="#121212" />
    <title>Pulsar VPN</title>
  </head>

  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```
pulsar-ui\package.json
```
{
  "name": "pulsar-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite --port 1420",
    "build": "vue-tsc --noEmit && vite build",
    "typecheck": "vue-tsc --noEmit",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-autostart": "^2.5.1",
    "@tauri-apps/plugin-opener": "^2",
    "@tauri-apps/plugin-store": "~2",
    "@tresjs/cientos": "^5.4.0",
    "@tresjs/core": "^5.5.0",
    "@vueuse/core": "^14.2.1",
    "@vueuse/motion": "^3.0.3",
    "lucide-vue-next": "^0.575.0",
    "pinia": "^3.0.4",
    "three": "^0.183.0",
    "vue": "^3.5.13"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.2.0",
    "@tauri-apps/cli": "^2",
    "@vitejs/plugin-vue": "^5.2.1",
    "autoprefixer": "^10.4.24",
    "postcss": "^8.5.6",
    "tailwindcss": "^4.2.0",
    "typescript": "~5.6.2",
    "vite": "^6.0.3",
    "vue-tsc": "^2.1.10"
  }
}
```
pulsar-ui\src\App.vue
```
<script setup lang="ts">
import {ref, computed, onMounted, watch} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'
import {load} from '@tauri-apps/plugin-store'
import {enable, disable, isEnabled} from '@tauri-apps/plugin-autostart'
import {
  ShieldCheck, Globe2, Settings, ArrowDownToLine, ArrowUpFromLine,
  Lock, Zap, Server, Plus, Link, Trash2, CheckCircle2,
  Power, ShieldAlert, Search, Loader2, X, Monitor, Globe,
  Terminal, Trash, MonitorCheck, RefreshCcw,
  ArrowDownWideNarrow, ArrowUpNarrowWide, ChevronDown, ChevronRight,
  Sparkles, ArrowRight, Check, Settings2, ClipboardPaste
} from 'lucide-vue-next'
import PulsarSphere from './components/PulsarSphere.vue'
import BottomNav from './components/BottomNav.vue'
import {logger} from './utils/logger'

// --- РРќРўР•Р Р¤Р•Р™РЎР« ---
interface Subscription {
  id: string
  url: string
  name: string
  expire?: number
  lastUpdated: number
  nodes: Profile[]
}

interface Profile {
  id: number
  name: string
  protocol: string
  ping: number // 0 = Pinging, -1 = Timeout, >0 = ms
  active: boolean
  rawLink: string
  flag: string
  country: string
  subscriptionId?: string
  dnsType?: 'https' | 'http' | 'udp'
  allowInsecure?: boolean // Р”РћР‘РђР’Р›Р•РќРћ
  /** Р’РєР»СЋС‡РёС‚СЊ IPv6 РІ TUN/DNS/dial; РїРѕ СѓРјРѕР»С‡Р°РЅРёСЋ РІС‹РєР». (С‚РѕР»СЊРєРѕ IPv4). */
  useIpv6?: boolean
}

interface RoutingConfig {
  mode: 'global' | 'bypass' | 'proxy'
  domains: string[]
  apps: string[]
}

// --- РЎРћРЎРўРћРЇРќРР• ---
const isConnected = ref(false)
const isImporting = ref(false)
const isPinging = ref(false)

const currentMetrics = ref({download: 0, upload: 0})

const activeTab = ref<'dashboard' | 'nodes' | 'settings'>('dashboard')
const showLogs = ref(false)
const toast = ref<string | null>(null)
let toastTimer: number | null = null
const showToast = (msg: string) => {
  toast.value = msg
  if (toastTimer) window.clearTimeout(toastTimer)
  toastTimer = window.setTimeout(() => { toast.value = null }, 2200)
}

// WIZARD STATE
const showWizard = ref(false)
const wizardStep = ref(1)

let store: Awaited<ReturnType<typeof load>> | null = null

const importUrl = ref('')
const searchQuery = ref('')

// РЎРћР РўРР РћР’РљРђ
const sortBy = ref<'name' | 'ping'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')

const subscriptions = ref<Subscription[]>([])
const standaloneProfiles = ref<Profile[]>([])
const expandedSubs = ref<Set<string>>(new Set())

// РЎРћРЎРўРћРЇРќРР• РњРћР”РђР›Р¬РќРћР“Рћ РћРљРќРђ РќРђРЎРўР РћР•Рљ РџР РћР¤РР›РЇ
const editModalOpen = ref(false)
const editingProfileId = ref<number | null>(null)
const editedDnsType = ref<'https' | 'http' | 'udp'>('https')
const editedAllowInsecure = ref(false) // Р”РћР‘РђР’Р›Р•РќРћ
const editedUseIpv6 = ref(false)

const logs = ref<string[]>([
  '[SYSTEM] Pulsar Engine Initialized',
  '[SYSTEM] Waiting for connection protocol...'
])
const logsContainer = ref<HTMLElement | null>(null)

// --- РќРћР’Р«Р™ РЎРџРРЎРћРљ РЎРўР РђРќ Р”Р›РЇ GEOIP ---
const popularCountries =[
  { code: 'RU', name: 'Russia', flag: 'рџ‡·рџ‡є' },
  { code: 'US', name: 'United States', flag: 'рџ‡єрџ‡ё' },
  { code: 'DE', name: 'Germany', flag: 'рџ‡©рџ‡Є' },
  { code: 'GB', name: 'United Kingdom', flag: 'рџ‡¬рџ‡§' },
  { code: 'FR', name: 'France', flag: 'рџ‡«рџ‡·' },
  { code: 'NL', name: 'Netherlands', flag: 'рџ‡ірџ‡±' },
  { code: 'CA', name: 'Canada', flag: 'рџ‡Ёрџ‡¦' },
  { code: 'AU', name: 'Australia', flag: 'рџ‡¦рџ‡є' },
  { code: 'JP', name: 'Japan', flag: 'рџ‡Їрџ‡µ' },
  { code: 'CN', name: 'China', flag: 'рџ‡Ёрџ‡і' },
  { code: 'BR', name: 'Brazil', flag: 'рџ‡§рџ‡·' },
  { code: 'IN', name: 'India', flag: 'рџ‡®рџ‡і' },
  { code: 'IT', name: 'Italy', flag: 'рџ‡®рџ‡№' },
  { code: 'ES', name: 'Spain', flag: 'рџ‡Єрџ‡ё' },
  { code: 'SE', name: 'Sweden', flag: 'рџ‡ёрџ‡Є' },
  { code: 'NO', name: 'Norway', flag: 'рџ‡ірџ‡ґ' },
  { code: 'FI', name: 'Finland', flag: 'рџ‡«рџ‡®' },
  { code: 'DK', name: 'Denmark', flag: 'рџ‡©рџ‡°' },
  { code: 'PL', name: 'Poland', flag: 'рџ‡µрџ‡±' },
  { code: 'UA', name: 'Ukraine', flag: 'рџ‡єрџ‡¦' },
];

const settings = ref({
  autoStart: false,
  minimizeToTray: true,
  setupWizardCompleted: false,
  enableGeoip: false,
  geoipRules: {} as Record<string, 'proxy'|'direct'|'block'>
})

// --- РџР›РђРўР¤РћР РњР•РќРќРђРЇ РРќР¤Рћ ---
// get_platform_info (Tauri-РєРѕРјР°РЅРґР°) СЃРѕРѕР±С‰Р°РµС‚ UI, РєР°РєРёРµ РћРЎ-СЃРїРµС†РёС„РёС‡РЅС‹Рµ
// С„РёС‡Рё Р°РєС‚РёРІРЅС‹: apps-list СЂР°Р±РѕС‚Р°РµС‚ С‚РѕР»СЊРєРѕ РЅР° Windows/Linux/macOS
// (РЅР° Android вЂ” С‚РѕР»СЊРєРѕ РєРѕРіРґР° Р±СѓРґРµС‚ СЂРµР°Р»РёР·РѕРІР°РЅ VpnService.Builder).
interface PlatformInfo {
  os: string
  apps_label: string
  apps_placeholder: string
  apps_append_suffix: boolean
  apps_supported: boolean
}
const platformInfo = ref<PlatformInfo>({
  os: 'unknown',
  apps_label: 'Applications',
  apps_placeholder: 'process name',
  apps_append_suffix: false,
  apps_supported: false
})

const routing = ref<RoutingConfig>({
  mode: 'bypass',
  domains: [],
  apps:[]
})

const newDomainInput = ref('')
const newAppInput = ref('')

// --- COMPUTED ---
const profiles = computed(() => {
  const subsNodes = subscriptions.value.flatMap(s => s.nodes)
  return[...subsNodes, ...standaloneProfiles.value]
})

const downloadSpeed = computed(() => ((currentMetrics.value.download * 8) / (1024 * 1024)).toFixed(1))
const uploadSpeed = computed(() => ((currentMetrics.value.upload * 8) / (1024 * 1024)).toFixed(1))

const activeProfile = computed(() =>
        profiles.value.find(p => p.active) || {
          name: 'No Node Selected',
          protocol: 'None',
          flag: 'рџЊЌ',
          dnsType: 'https',
          allowInsecure: false,
          useIpv6: false
        }
)

// --- Р›РћР“РРљРђ Р¤РР›Р¬РўР РђР¦РР Р РЎРћР РўРР РћР’РљР ---
const getFilteredAndSortedNodes = (nodes: Profile[]) => {
  if (!nodes) return[]
  let result = nodes
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = nodes.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.protocol.toLowerCase().includes(q) ||
        p.country.toLowerCase().includes(q)
    )
  }
  return result.sort((a, b) => {
    let modifier = sortOrder.value === 'asc' ? 1 : -1
    if (sortBy.value === 'name') {
      return a.name.localeCompare(b.name) * modifier
    } else if (sortBy.value === 'ping') {
      const getPingWeight = (p: number) => {
        if (p > 0) return p
        if (p === 0) return 999998
        if (p === -1) return 999999
        return 999999
      }
      return (getPingWeight(a.ping) - getPingWeight(b.ping)) * modifier
    }
    return 0
  })
}

const toggleSort = (field: 'name' | 'ping') => {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}

watch(activeTab, () => {
  searchQuery.value = ''
})

// --- РђР”РђРџРўРР’РќРћРЎРўР¬ ---
// РќР° РјРѕР±РёР»СЊРЅС‹С… (<768px) РїСЂСЏС‡РµРј СЃР°Р№РґР±Р°СЂ, РїРѕРєР°Р·С‹РІР°РµРј BottomNav, РІС‹РєР»СЋС‡Р°РµРј 3D-СЃС„РµСЂСѓ
// РґР»СЏ СЌРєРѕРЅРѕРјРёРё Р±Р°С‚Р°СЂРµРё Рё CPU.
const isMobile = ref(false)
let mqMobile: MediaQueryList | null = null
function applyMq(e: MediaQueryListEvent | MediaQueryList) {
  isMobile.value = e.matches
}

// --- INIT ---
onMounted(async () => {
  logger.info('App', 'Vue app mounting');

  // Р РµРіРёСЃС‚СЂРёСЂСѓРµРј СЃР»СѓС€Р°С‚РµР»СЊ media-query РґР»СЏ Р°РґР°РїС‚РёРІРЅРѕСЃС‚Рё.
  mqMobile = window.matchMedia('(max-width: 767px)')
  applyMq(mqMobile)
  mqMobile.addEventListener('change', applyMq)

  // РџР»Р°С‚С„РѕСЂРјРµРЅРЅР°СЏ РёРЅС„РѕСЂРјР°С†РёСЏ (apps placeholder, СЃСѓС„С„РёРєСЃС‹ Рё С‚.Рґ.)
  try {
    const info = await invoke<PlatformInfo>('get_platform_info')
    platformInfo.value = info
    const vpnState = await invoke<string>('get_vpn_state')
    isConnected.value = vpnState === 'Connected' || vpnState === 'Connecting'
    logger.info('App', `Platform: ${info.os}, apps_supported=${info.apps_supported}`)
  } catch (e) {
    logger.warn('App', 'get_platform_info failed, using defaults', e)
  }

  try {
    store = await load('store.json', {autoSave: false, defaults: {}})

    const savedSubs = await store.get('subscriptions')
    if (savedSubs) subscriptions.value = savedSubs as Subscription[]

    const savedStandalone = await store.get('standaloneProfiles')
    if (savedStandalone) standaloneProfiles.value = savedStandalone as Profile[]

    const oldProfiles = await store.get('profiles')
    if (oldProfiles && (!savedStandalone || (savedStandalone as Profile[]).length === 0)) {
      standaloneProfiles.value = oldProfiles as Profile[]
      await store.delete('profiles')
    }

    // РњРР“Р РђР¦РРЇ Р”РђРќРќР«РҐ Р”Р›РЇ DNS TYPE Р ALLOW INSECURE
    standaloneProfiles.value = standaloneProfiles.value.map(p => ({
      ...p,
      dnsType: p.dnsType || 'https',
      allowInsecure: p.allowInsecure ?? false, // Р”РћР‘РђР’Р›Р•РќРћ
      useIpv6: p.useIpv6 ?? false
    }))

    subscriptions.value.forEach(sub => {
      sub.nodes = sub.nodes.map(p => ({
        ...p,
        dnsType: p.dnsType || 'https',
        allowInsecure: p.allowInsecure ?? false, // Р”РћР‘РђР’Р›Р•РќРћ
        useIpv6: p.useIpv6 ?? false
      }))
    })

    const savedRouting = await store.get('routing')
    if (savedRouting) routing.value = savedRouting as RoutingConfig

    const savedSettings = await store.get('settings')
    if (savedSettings) {
      settings.value = savedSettings as any
      // РЈР±РµРґРёРјСЃСЏ, С‡С‚Рѕ geoipRules СЃСѓС‰РµСЃС‚РІСѓРµС‚
      if (!settings.value.geoipRules) settings.value.geoipRules = {}
    }
    settings.value.autoStart = await isEnabled();
    await invoke('set_minimize_to_tray', {enable: settings.value.minimizeToTray});

    // Check Wizard Status
    if (!settings.value.setupWizardCompleted) {
      showWizard.value = true
    }

  } catch (e) {
    logger.error('App', 'Store init error', e)
  }

  if (profiles.value.length > 0) refreshPings()

  if ((window as any).__TAURI_INTERNALS__) {
    try {
      await listen<{ download: number, upload: number }>('vpn_metrics', e => {
        currentMetrics.value = e.payload
      })
      await listen<string>('vpn_log', e => addLog(`[CORE] ${e.payload}`))

      const maxAttempts = 60
      for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
          await invoke('start_metrics_stream')
          break
        } catch (_) {
          if (attempt < maxAttempts - 1) await new Promise(r => setTimeout(r, 2000))
        }
      }
    } catch (e) {
      console.error('Failed to init streams:', e)
    }
  }
})

// --- LOGIC ---
// РљРѕР°Р»РµСЃРёРЅРі Р·Р°РїРёСЃРµР№ РІ Р¶СѓСЂРЅР°Р»: РЅРµ РґС‘СЂРіР°РµРј СЂРµР°РєС‚РёРІРЅРѕСЃС‚СЊ Рё DOM РЅР° РєР°Р¶РґСѓСЋ
// СЃС‚СЂРѕРєСѓ РёР· Р±Р°С‚С‡Р°, Р° РѕРґРёРЅ СЂР°Р· Р·Р° РєР°РґСЂ. РЎРёР»СЊРЅРѕ СЃРЅРёР¶Р°РµС‚ РЅР°РіСЂСѓР·РєСѓ, РєРѕРіРґР°
// РёР· Rust-СЃС‚СЂРёРјР° РїСЂРёР»РµС‚Р°РµС‚ РїР°РєРµС‚ РІ РґРµСЃСЏС‚РєРё СЃС‚СЂРѕРє СЂР°Р·РѕРј.
let pendingLogs: string[] = []
let logFlushScheduled = false
const flushLogs = () => {
  logFlushScheduled = false
  if (pendingLogs.length === 0) return
  logs.value.push(...pendingLogs)
  pendingLogs = []
  if (logs.value.length > 100) logs.value.splice(0, logs.value.length - 100)
  if (logsContainer.value) logsContainer.value.scrollTop = logsContainer.value.scrollHeight
}
const addLog = (msg: string) => {
  pendingLogs.push(msg)
  if (!logFlushScheduled) {
    logFlushScheduled = true
    requestAnimationFrame(flushLogs)
  }
}

// РЈРІРµРґРѕРјР»СЏРµРј Rust Рѕ С‚РѕРј, РѕС‚РєСЂС‹С‚Р° Р»Рё РїР°РЅРµР»СЊ Р»РѕРіРѕРІ: РїРѕРєР° Р·Р°РєСЂС‹С‚Р° вЂ”
// Р±СЌРєРµРЅРґ РЅРµ С€Р»С‘С‚ "vpn_log" СЃРѕР±С‹С‚РёСЏ РІ webview (СЃРј. set_logs_streaming РІ main.rs).
watch(showLogs, async (open) => {
  try { await invoke('set_logs_streaming', { enable: open }) } catch {}
})
onMounted(async () => {
  // РќР° СЃР»СѓС‡Р°Р№ РµСЃР»Рё РїР°РЅРµР»СЊ Р»РѕРіРѕРІ СѓР¶Рµ РѕС‚РєСЂС‹С‚Р° РґРѕ РїРѕРґРєР»СЋС‡РµРЅРёСЏ IPC
  try { await invoke('set_logs_streaming', { enable: showLogs.value }) } catch {}
})

const toggleAutoStart = async () => {
  try {
    if (settings.value.autoStart) {
      await disable();
      settings.value.autoStart = false;
    } else {
      await enable();
      settings.value.autoStart = true;
    }
    await saveData();
  } catch (e) {
    console.error(e);
    alert('Failed to toggle autostart: ' + e);
  }
}

const toggleTray = async () => {
  settings.value.minimizeToTray = !settings.value.minimizeToTray;
  await invoke('set_minimize_to_tray', {enable: settings.value.minimizeToTray});
  await saveData();
}

const refreshPings = async () => {
  if (isPinging.value) return
  isPinging.value = true
  addLog('[SYSTEM] Started latency check...')

  profiles.value.forEach(p => p.ping = 0)

  const promises = profiles.value.map(async (p) => {
    try {
      const ms = await invoke<number>('check_ping', {link: p.rawLink})
      p.ping = ms
    } catch (e) {
      p.ping = -1
    }
  })

  await Promise.allSettled(promises)
  await saveData()
  isPinging.value = false
}

const toggleVpn = async () => {
  if (profiles.value.length === 0) {
    showToast('Add a node first')
    activeTab.value = 'nodes'
    return
  }
  const active = profiles.value.find(p => p.active)
  if (!active || !active.rawLink) return

  const targetState = !isConnected.value

  try {
    addLog(`[SYSTEM] Attempting to ${targetState ? 'connect' : 'disconnect'}...`)

    const res = await invoke<string>('toggle_vpn', {
      payload: {
        connect: targetState,
        link: active.rawLink,
        routing_config: {
          mode: routing.value.mode,
          domains: routing.value.domains,
          apps: routing.value.apps,
          enable_geoip: settings.value.enableGeoip,
          geoip_rules: settings.value.geoipRules,
          dns_type: active.dnsType || 'https',
          allow_insecure: active.allowInsecure ?? false, // Р”РћР‘РђР’Р›Р•РќРћ
          use_ipv6: active.useIpv6 ?? false
        }
      }
    })

    isConnected.value = res === 'Connected'
    addLog(`[SYSTEM] ${res}`)
  } catch (e) {
    addLog(`[ERROR] ${e}`);
    isConnected.value = false;
  }
}

const saveData = async () => {
  if (store) {
    await store.set('subscriptions', subscriptions.value)
    await store.set('standaloneProfiles', standaloneProfiles.value)
    await store.set('routing', routing.value)
    await store.set('settings', settings.value)
    await store.save()
  }
}

// РЈРїСЂР°РІР»РµРЅРёРµ РјРѕРґР°Р»СЊРЅС‹Рј РѕРєРЅРѕРј РЅР°СЃС‚СЂРѕР№РєРё РїСЂРѕС„РёР»СЏ
const openEditModal = (profile: Profile) => {
  editingProfileId.value = profile.id
  editedDnsType.value = profile.dnsType || 'https'
  editedAllowInsecure.value = profile.allowInsecure ?? false // Р”РћР‘РђР’Р›Р•РќРћ
  editedUseIpv6.value = profile.useIpv6 ?? false
  editModalOpen.value = true
}

const saveProfileSettings = async () => {
  if (editingProfileId.value === null) return

  const updateProfile = (nodes: Profile[]) => {
    const p = nodes.find(n => n.id === editingProfileId.value)
    if (p) {
      p.dnsType = editedDnsType.value
      p.allowInsecure = editedAllowInsecure.value // Р”РћР‘РђР’Р›Р•РќРћ
      p.useIpv6 = editedUseIpv6.value
    }
  }

  updateProfile(standaloneProfiles.value)
  subscriptions.value.forEach(sub => updateProfile(sub.nodes))

  await saveData()
  editModalOpen.value = false
  editingProfileId.value = null
}

// Wizard Logic
const completeWizard = async () => {
  settings.value.setupWizardCompleted = true
  await saveData()
  showWizard.value = false
}

const handleWizardImport = async () => {
  await handleImport()
  if (profiles.value.length > 0) {
    wizardStep.value = 3
  }
}

const addDomain = () => {
  if (newDomainInput.value.trim()) {
    routing.value.domains.push(newDomainInput.value.trim());
    newDomainInput.value = '';
    saveData()
  }
}
const removeDomain = (idx: number) => {
  routing.value.domains.splice(idx, 1);
  saveData()
}
const addApp = () => {
  if (newAppInput.value.trim()) {
    let app = newAppInput.value.trim();
    // РќР° Windows РїСЂРёРЅСЏС‚Рѕ СѓРєР°Р·С‹РІР°С‚СЊ РёРјСЏ СЃ .exe (РЅР°РїСЂРёРјРµСЂ, chrome.exe).
    // РќР° РґСЂСѓРіРёС… РїР»Р°С‚С„РѕСЂРјР°С… (Linux: chrome, macOS: Google Chrome,
    // Android: com.android.chrome) .exe РЅРµ РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ вЂ” РґРѕР±Р°РІР»СЏРµРј
    // СЃСѓС„С„РёРєСЃ С‚РѕР»СЊРєРѕ РµСЃР»Рё РїР»Р°С‚С„РѕСЂРјР° РµРіРѕ С‚СЂРµР±СѓРµС‚.
    if (platformInfo.value.apps_append_suffix && !app.toLowerCase().endsWith('.exe')) {
      app += '.exe';
    }
    routing.value.apps.push(app);
    newAppInput.value = '';
    saveData()
  }
}
const removeApp = (idx: number) => {
  routing.value.apps.splice(idx, 1);
  saveData()
}

const setActiveProfile = async (id: number) => {
  subscriptions.value.forEach(s => s.nodes.forEach(p => p.active = (p.id === id)))
  standaloneProfiles.value.forEach(p => p.active = (p.id === id))
  await saveData()
}

const deleteProfile = async (id: number) => {
  const wasActive = profiles.value.find(p => p.id === id)?.active
  standaloneProfiles.value = standaloneProfiles.value.filter(p => p.id !== id)
  subscriptions.value.forEach(s => {
    s.nodes = s.nodes.filter(p => p.id !== id)
  })

  if (wasActive) {
    if (profiles.value.length > 0) profiles.value[0].active = true
    else isConnected.value = false
  }
  await saveData()
}

const deleteSubscription = async (id: string) => {
  const sub = subscriptions.value.find(s => s.id === id)
  if (sub && sub.nodes.some(n => n.active)) isConnected.value = false
  subscriptions.value = subscriptions.value.filter(s => s.id !== id)
  expandedSubs.value.delete(id)
  await saveData()
}

const parseLocationFromName = (name: string) => {
  const emojiRegex = /[\u{1F1E6}-\u{1F1FF}]{2}/u
  const match = name.match(emojiRegex)
  let country = 'Unknown'
  if (name.includes('RU')) country = 'Russia'
  else if (name.includes('DE')) country = 'Germany'
  else if (name.includes('NL')) country = 'Netherlands'
  else if (name.includes('US')) country = 'USA'
  else if (name.includes('PL')) country = 'Poland'
  return {flag: match ? match[0] : 'рџЊЌ', country}
}

const createProfileObj = (url: string, subId?: string): Profile | null => {
  let protocol = 'Custom', name = 'Node';
  if (url.startsWith('vless://')) protocol = 'VLESS';
  else if (url.startsWith('wireguard://')) protocol = 'WireGuard';
  else if (url.startsWith('warp://')) protocol = 'WARP';
  else if (url.startsWith('ss://')) protocol = 'Shadowsocks';
  else if (url.startsWith('trojan://')) protocol = 'Trojan';
  else if (url.startsWith('hysteria2://') || url.startsWith('hy2://')) protocol = 'Hysteria2';

  const hashIndex = url.indexOf('#')
  if (hashIndex !== -1) {
    try {
      name = decodeURIComponent(url.substring(hashIndex + 1))
    } catch (e) {}
  }
  const {flag, country} = parseLocationFromName(name)

  return {
    id: Date.now() + Math.random(),
    name,
    protocol,
    ping: 0,
    active: false,
    rawLink: url,
    flag,
    country,
    subscriptionId: subId,
    dnsType: 'https',
    allowInsecure: false, // Р”РћР‘РђР’Р›Р•РќРћ
    useIpv6: false
  }
}

const handleImport = async () => {
  if (!importUrl.value) return
  const input = importUrl.value.trim()
  importUrl.value = ''
  isImporting.value = true

  try {
    if (input.startsWith('http')) {
      addLog(`[SYSTEM] Fetching subscription...`)
      const res = await invoke<{nodes: string[], userinfo: string | null}>('fetch_subscription', {url: input})
      const subId = Date.now().toString()
      const newNodes = res.nodes.map(link => createProfileObj(link, subId)).filter(p => p !== null) as Profile[]

      let expireTime = undefined
      if (res.userinfo) {
        const match = res.userinfo.match(/expire=(\d+)/)
        if (match) expireTime = parseInt(match[1]) * 1000
      }

      if (profiles.value.length === 0 && newNodes.length > 0) newNodes[0].active = true

      subscriptions.value.push({
        id: subId, url: input, name: new URL(input).hostname, lastUpdated: Date.now(), expire: expireTime, nodes: newNodes
      })
      addLog(`[SYSTEM] Imported subscription with ${newNodes.length} nodes`)
    } else {
      const p = createProfileObj(input)
      if (p) {
        if (profiles.value.length === 0) p.active = true
        standaloneProfiles.value.push(p)
        addLog(`[SYSTEM] Imported standalone profile: ${p.name}`)
      }
    }
    await saveData()
    refreshPings()
  } catch (e) {
    addLog(`[ERROR] Import failed: ${e}`)
  } finally {
    isImporting.value = false
  }
}

const refreshSubscription = async (sub: Subscription) => {
  addLog(`[SYSTEM] Updating subscription ${sub.name}...`)
  try {
    const res = await invoke<{nodes: string[], userinfo: string | null}>('fetch_subscription', { url: sub.url })
    const activeLink = profiles.value.find(p => p.active)?.rawLink
    const newNodes = res.nodes.map(link => createProfileObj(link, sub.id)).filter(p => p !== null) as Profile[]

    if (activeLink) {
      const found = newNodes.find(n => n.rawLink === activeLink)
      if (found) found.active = true
    }

    sub.nodes = newNodes
    sub.lastUpdated = Date.now()
    if (res.userinfo) {
      const match = res.userinfo.match(/expire=(\d+)/)
      if (match) sub.expire = parseInt(match[1]) * 1000
    }

    await saveData()
    refreshPings()
    addLog(`[SYSTEM] Subscription updated. Total nodes: ${newNodes.length}`)
  } catch(e) {
    addLog(`[ERROR] Failed to update subscription: ${e}`)
  }
}

const toggleSubExpanded = (id: string) => {
  if (expandedSubs.value.has(id)) expandedSubs.value.delete(id)
  else expandedSubs.value.add(id)
  // trigger reactivity for Set
  expandedSubs.value = new Set(expandedSubs.value)
}

interface NodeGroup {
  id: string
  name: string
  sub: Subscription | null
  nodes: Profile[]
}
const nodeGroups = computed<NodeGroup[]>(() => {
  const groups: NodeGroup[] = []
  for (const sub of subscriptions.value) {
    groups.push({ id: sub.id, name: sub.name, sub, nodes: sub.nodes })
  }
  if (standaloneProfiles.value.length > 0) {
    groups.push({
      id: '__standalone__',
      name: 'Standalone Nodes',
      sub: null,
      nodes: standaloneProfiles.value
    })
  }
  return groups
})

const quickAdd = async () => {
  try {
    let text = ''
    try {
      if (platformInfo.value?.os === 'android') {
        text = await invoke<string>('read_clipboard')
      } else {
        text = await navigator.clipboard.readText()
      }
    } catch (e) {
      addLog(`[WARN] clipboard read: ${e}`)
      text = ''
    }
    const trimmed = (text || '').trim()
    if (!trimmed) {
      showToast('Clipboard is empty')
      return
    }
    if (trimmed.startsWith('http') || trimmed.startsWith('vless://') || trimmed.startsWith('vmess://')
        || trimmed.startsWith('trojan://') || trimmed.startsWith('ss://') || trimmed.startsWith('hy2://')
        || trimmed.startsWith('hysteria2://') || trimmed.startsWith('wireguard://') || trimmed.startsWith('warp://')) {
      importUrl.value = trimmed
      await handleImport()
      showToast('Imported from clipboard')
    } else {
      importUrl.value = trimmed
      showToast('Pasted to input вЂ” review and add')
    }
  } catch (e) {
    showToast('Quick add failed')
  }
}
</script>

<template>
  <div
      class="fixed inset-0 w-full h-full bg-[#121212] text-[#FDFDFD] overflow-hidden selection:bg-[#9D68FF]/30 font-sans flex">
    <!-- Background -->
    <div class="absolute inset-0 z-0 pointer-events-none">
      <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_0%,#121212_95%)]"></div>
    </div>

      <!-- Pulsar Layer (3D Sphere) -->
    <div
        class="fixed inset-0 z-10 flex items-center justify-center pointer-events-none transition-all duration-700 ease-[cubic-bezier(0.16,1,0.3,1)]"
        :class="activeTab === 'dashboard' ? 'scale-100 opacity-100 blur-none' : 'scale-90 opacity-20 blur-md'"
    >
      <div class="absolute inset-0">
        <PulsarSphere :speed="currentMetrics.download + currentMetrics.upload" :is-connected="isConnected" :active="!isMobile && activeTab === 'dashboard'"/>
      </div>
      <!-- Р‘РѕР»СЊС€Р°СЏ РєРЅРѕРїРєР° РїРѕРґРєР»СЋС‡РµРЅРёСЏ -->
      <button
          v-if="activeTab === 'dashboard'"
          @click="toggleVpn"
          class="connect-orb relative z-[100] w-[220px] h-[220px] md:w-[450px] md:h-[450px] rounded-full flex flex-col items-center justify-center group outline-none cursor-pointer pointer-events-auto active:scale-95 transition-transform"
      >
        <!-- Р’РЅСѓС‚СЂРµРЅРЅРµРµ СЃРІРµС‡РµРЅРёРµ Рё РєРѕР»СЊС†Рѕ (РІСЃРµРіРґР° РІРёРґРЅРѕ РЅР° РјРѕР±РёР»СЊРЅРѕРј, РїРѕСЏРІР»СЏРµС‚СЃСЏ РїСЂРё hover РЅР° РґРµСЃРєС‚РѕРїРµ) -->
        <div
            class="orb-ring absolute inset-0 rounded-full pointer-events-none transition-opacity duration-500"
            :class="isConnected
              ? 'opacity-100 shadow-[0_0_60px_rgba(157,104,255,0.6),inset_0_0_40px_rgba(157,104,255,0.4)]'
              : 'opacity-70 md:opacity-0 md:group-hover:opacity-70 shadow-[0_0_40px_rgba(157,104,255,0.2),inset_0_0_30px_rgba(157,104,255,0.15)]'"
        ></div>
        <!-- Р›РµР№Р±Р»: РІРёРґРµРЅ РІСЃРµРіРґР° РЅР° РјРѕР±РёР»СЊРЅРѕРј, РїРѕ hover РЅР° РґРµСЃРєС‚РѕРїРµ -->
        <div
            class="orb-label md:mt-[280px] mt-0 md:opacity-0 opacity-100 md:group-hover:opacity-100 md:group-hover:-translate-y-6 px-6 py-3 rounded-full bg-[#121212]/90 backdrop-blur-2xl border border-[#9D68FF]/20 shadow-[0_0_30px_rgba(0,0,0,0.5)] transition-all duration-500 ease-out flex items-center gap-3 pointer-events-none">
          <Power class="w-5 h-5 md:w-4 md:h-4 animate-pulse" :class="isConnected ? 'text-[#9D68FF]' : 'text-white'"/>
          <span class="text-xs md:text-[10px] font-black uppercase tracking-[0.4em]"
                :class="isConnected ? 'text-[#9D68FF]' : 'text-white'">
            {{ isConnected ? 'DISCONNECT' : 'INITIALIZE' }}
          </span>
        </div>
      </button>
    </div>

    <!-- MAIN UI WRAPPER -->
    <div class="relative z-20 w-full h-full flex p-6 gap-6 pointer-events-none main-padding">
      <!-- SIDEBAR MENU (desktop) -->
      <nav
          class="glass-panel sidebar w-24 rounded-[2.5rem] flex flex-col items-center py-8 justify-between flex-shrink-0 pointer-events-auto shadow-2xl z-30 relative">
        <div class="flex flex-col items-center gap-10 w-full">
          <div
              class="w-12 h-12 rounded-[1.2rem] bg-[#9D68FF] flex items-center justify-center">
            <Zap class="w-6 h-6 text-[#121212]" fill="currentColor"/>
          </div>
          <div class="flex flex-col gap-4 w-full px-4 mt-4">
            <button @click="activeTab = 'dashboard'" class="nav-btn" :class="{ active: activeTab === 'dashboard' }">
              <ShieldCheck class="w-6 h-6 icon"/>
            </button>
            <button @click="activeTab = 'nodes'" class="nav-btn" :class="{ active: activeTab === 'nodes' }">
              <Globe2 class="w-6 h-6 icon"/>
            </button>
            <button @click="activeTab = 'settings'" class="nav-btn" :class="{ active: activeTab === 'settings' }">
              <Settings class="w-6 h-6 icon"/>
            </button>
          </div>
        </div>
        <button @click="showLogs = !showLogs"
                class="w-12 h-12 rounded-full flex items-center justify-center transition-all"
                :class="showLogs ? 'bg-[#FDFDFD]/10 text-[#FDFDFD]' : 'text-[#FDFDFD]/30 hover:text-[#FDFDFD] hover:bg-[#FDFDFD]/5'">
          <Terminal class="w-5 h-5"/>
        </button>
      </nav>

      <!-- BOTTOM NAV (mobile) -->
      <BottomNav
          v-if="isMobile"
          class="bottom-nav"
          :active-tab="activeTab"
          @navigate="(t: any) => activeTab = t"
          :show-logs="showLogs"
          @toggle-logs="showLogs = !showLogs"
      />

      <!-- CONTENT AREA -->
      <main class="flex-1 flex flex-col justify-between overflow-hidden relative pointer-events-none main-area">
        <header
            class="app-header glass-panel pointer-events-auto shrink-0 z-30">
          <!-- MOBILE HEADER (compact) -->
          <div class="md:hidden flex items-center justify-between h-14 px-3 gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <div class="w-8 h-8 rounded-xl bg-[#9D68FF] flex items-center justify-center shrink-0">
                <Zap class="w-4 h-4 text-[#121212]" fill="currentColor"/>
              </div>
              <div class="min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="text-base font-black tracking-tight">Pulsar</span>
                  <span class="text-base font-black text-[#9D68FF]">VPN</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full"
                   :class="isConnected ? 'bg-[#9D68FF]/15' : 'bg-[#FDFDFD]/5'">
                <div class="w-1.5 h-1.5 rounded-full"
                     :class="isConnected ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/30'"></div>
                <span class="text-[9px] font-black uppercase tracking-widest"
                      :class="isConnected ? 'text-[#9D68FF]' : 'text-[#FDFDFD]/40'">
                  {{ isConnected ? 'ON' : 'OFF' }}
                </span>
              </div>
              <button @click="quickAdd" class="quick-add-btn" aria-label="Quick add from clipboard" title="Paste from clipboard">
                <Plus class="w-5 h-5"/>
              </button>
            </div>
          </div>
          <!-- DESKTOP HEADER (full) -->
          <div class="hidden md:flex justify-between items-center rounded-[2rem] px-8 py-5">
            <div>
              <h1 class="text-3xl font-black tracking-tight flex items-center gap-2">Pulsar<span class="text-[#9D68FF]">VPN</span>
              </h1>
              <p class="text-[10px] text-[#FDFDFD]/50 font-bold uppercase tracking-[0.2em] mt-1 flex items-center gap-2">
                <Lock class="w-3 h-3" :class="isConnected ? 'text-[#9D68FF]' : 'text-[#FDFDFD]/30'"/>
                System {{ isConnected ? 'Encrypted' : 'Exposed' }}
              </p>
            </div>
            <div class="flex items-center gap-3">
              <button @click="quickAdd" class="quick-add-btn" aria-label="Quick add from clipboard" title="Paste from clipboard">
                <ClipboardPaste class="w-4 h-4"/>
                <span class="text-xs font-bold uppercase tracking-widest">Paste</span>
              </button>
              <div class="flex items-center gap-3 px-4 py-2 bg-[#121212]/50 rounded-xl border border-[#FDFDFD]/10">
                <div class="w-2 h-2 rounded-full"
                     :class="isConnected ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/30'"></div>
                <span class="text-xs font-bold tracking-widest uppercase"
                      :class="isConnected ? 'text-[#FDFDFD]' : 'text-[#FDFDFD]/50'">{{
                    isConnected ? 'Protected' : 'Unprotected'
                  }}</span>
              </div>
            </div>
          </div>
        </header>

        <!-- SUB-PAGES -->
        <div class="relative flex-1 overflow-hidden pointer-events-none mt-6">

          <!-- DASHBOARD -->
          <transition name="fade">
            <div v-if="activeTab === 'dashboard'"
                 class="absolute inset-0 flex flex-col justify-end pb-0 pointer-events-none">
              <!-- DESKTOP: 3 metric cards -->
              <div class="hidden md:grid grid-cols-3 gap-6 shrink-0 h-44 pointer-events-auto metrics-grid">
                <div class="glass-panel rounded-[2rem] p-6 flex flex-col justify-between shadow-lg relative overflow-hidden">
                  <div class="flex justify-between items-center relative z-10">
                    <span class="text-[10px] font-bold text-[#FDFDFD]/50 uppercase tracking-widest flex items-center gap-2">
                      <ArrowDownToLine class="w-4 h-4 text-[#9D68FF]"/> Download
                    </span>
                  </div>
                  <div class="flex items-baseline gap-2 relative z-10">
                    <span class="text-5xl font-black font-mono tracking-tighter">{{ isConnected ? downloadSpeed : '0.0' }}</span>
                    <span class="text-xs font-bold text-[#FDFDFD]/30 uppercase">Mbps</span>
                  </div>
                </div>
                <div class="glass-panel rounded-[2rem] p-6 flex flex-col justify-between shadow-lg relative overflow-hidden">
                  <div class="flex justify-between items-center relative z-10">
                    <span class="text-[10px] font-bold text-[#FDFDFD]/50 uppercase tracking-widest flex items-center gap-2">
                      <ArrowUpFromLine class="w-4 h-4 text-[#9D68FF]"/> Upload
                    </span>
                  </div>
                  <div class="flex items-baseline gap-2 relative z-10">
                    <span class="text-5xl font-black font-mono tracking-tighter">{{ isConnected ? uploadSpeed : '0.0' }}</span>
                    <span class="text-xs font-bold text-[#FDFDFD]/30 uppercase">Mbps</span>
                  </div>
                </div>
                <div @click="activeTab = 'nodes'"
                     class="glass-panel rounded-[2rem] p-6 flex flex-col justify-center gap-4 cursor-pointer hover:border-[#9D68FF]/50 transition-all shadow-lg bg-gradient-to-br from-white/[0.02] to-[#9D68FF]/[0.05]">
                  <span class="text-[10px] font-bold text-[#FDFDFD]/50 uppercase tracking-widest flex items-center gap-2">
                    <Server class="w-4 h-4 text-[#FDFDFD]/50"/> Active Tunnel
                  </span>
                  <div class="flex items-center gap-4">
                    <span class="text-4xl drop-shadow-md">{{ activeProfile.flag }}</span>
                    <div class="flex flex-col">
                      <span class="text-xl font-black uppercase tracking-tight truncate w-32">{{ activeProfile.name }}</span>
                      <span class="text-[10px] font-bold text-[#9D68FF] tracking-widest uppercase mt-1">{{ activeProfile.protocol }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <!-- MOBILE: single compact info row -->
              <div class="md:hidden flex items-center gap-3 px-1 pb-2 pointer-events-auto">
                <button @click="activeTab = 'nodes'" class="flex-1 glass-panel rounded-2xl px-4 py-3 flex items-center gap-3 min-w-0 active:scale-[0.98] transition-transform">
                  <span class="text-2xl drop-shadow-sm shrink-0">{{ activeProfile.flag }}</span>
                  <div class="flex-1 min-w-0 text-left">
                    <div class="text-[9px] font-black uppercase tracking-widest text-[#FDFDFD]/40">Active</div>
                    <div class="text-sm font-black truncate">{{ activeProfile.name }}</div>
                  </div>
                  <div v-if="isConnected" class="flex flex-col items-end shrink-0">
                    <span class="text-[8px] font-black uppercase tracking-widest text-[#FDFDFD]/40">в†“</span>
                    <span class="text-sm font-black font-mono text-[#9D68FF]">{{ downloadSpeed }}</span>
                  </div>
                </button>
              </div>
            </div>
          </transition>

          <!-- NODES TAB (subscriptions + standalone, accordion) -->
          <transition name="fade">
            <div v-if="activeTab === 'nodes'"
                 class="absolute inset-0 pointer-events-auto flex flex-col gap-4 custom-scroll overflow-y-auto pb-4">
              <!-- Import Box -->
              <div class="glass-panel p-4 md:p-6 rounded-[1.5rem] md:rounded-[2rem] flex flex-col gap-3 shrink-0">
                <h2 class="text-base md:text-lg font-black uppercase tracking-widest flex items-center gap-2">
                  <Plus class="w-4 h-4 md:w-5 md:h-5 text-[#9D68FF]"/> Import
                </h2>
                <div class="flex gap-2 md:gap-3">
                  <div class="flex-1 relative">
                    <div class="absolute inset-y-0 left-3 md:left-4 flex items-center pointer-events-none">
                      <Link class="w-4 h-4 md:w-5 md:h-5 text-[#FDFDFD]/30"/>
                    </div>
                    <input v-model="importUrl" @keyup.enter="handleImport" type="text"
                           placeholder="Subscription link, vless://, wireguard://..."
                           class="w-full h-11 md:h-14 bg-[#121212]/50 border border-[#FDFDFD]/10 rounded-xl md:rounded-2xl pl-10 md:pl-12 pr-3 md:pr-4 text-xs md:text-sm font-mono focus:outline-none focus:border-[#9D68FF]/50 focus:ring-1 focus:ring-[#9D68FF]/50 transition-all"/>
                  </div>
                  <button @click="handleImport" :disabled="isImporting"
                          class="h-11 md:h-14 px-4 md:px-8 bg-[#9D68FF] text-[#121212] font-black uppercase tracking-widest rounded-xl md:rounded-2xl hover:bg-[#b084ff] transition-all flex items-center justify-center min-w-[80px] md:min-w-[140px] text-xs md:text-sm">
                    <Loader2 v-if="isImporting" class="w-4 h-4 md:w-5 md:h-5 animate-spin"/>
                    <span v-else>ADD</span>
                  </button>
                </div>
              </div>

              <!-- Search + sort + ping -->
              <div class="flex gap-2 md:gap-3 px-1 shrink-0">
                <div class="relative flex-1">
                  <div class="absolute inset-y-0 left-3 flex items-center pointer-events-none">
                    <Search class="w-4 h-4 text-[#FDFDFD]/30"/>
                  </div>
                  <input v-model="searchQuery" type="text" placeholder="Search nodes..."
                         class="w-full h-10 bg-[#121212]/30 border border-[#FDFDFD]/10 rounded-xl pl-9 pr-4 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50 transition-all"/>
                </div>
                <div class="flex gap-1 bg-[#121212]/30 p-1 rounded-xl border border-[#FDFDFD]/10">
                  <button @click="toggleSort('name')" class="px-2.5 h-8 rounded-lg text-[10px] font-bold uppercase tracking-wider flex items-center gap-1 transition-all"
                          :class="sortBy === 'name' ? 'bg-[#9D68FF]/20 text-[#9D68FF]' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    <span class="hidden sm:inline">Name</span>
                    <span class="sm:hidden">Aв†“</span>
                    <component :is="sortOrder === 'asc' ? ArrowDownWideNarrow : ArrowUpNarrowWide" v-if="sortBy === 'name'" class="w-3 h-3"/>
                  </button>
                  <button @click="toggleSort('ping')" class="px-2.5 h-8 rounded-lg text-[10px] font-bold uppercase tracking-wider flex items-center gap-1 transition-all"
                          :class="sortBy === 'ping' ? 'bg-[#9D68FF]/20 text-[#9D68FF]' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    <span class="hidden sm:inline">Ping</span>
                    <span class="sm:hidden">ms</span>
                    <component :is="sortOrder === 'asc' ? ArrowDownWideNarrow : ArrowUpNarrowWide" v-if="sortBy === 'ping'" class="w-3 h-3"/>
                  </button>
                </div>
                <button @click="refreshPings" :disabled="isPinging" aria-label="Ping all"
                        class="w-10 h-10 rounded-xl bg-[#121212]/30 border border-[#FDFDFD]/10 text-[#9D68FF] hover:bg-[#9D68FF]/10 flex items-center justify-center transition-all disabled:opacity-50 shrink-0">
                  <RefreshCcw class="w-4 h-4" :class="isPinging ? 'animate-spin' : ''"/>
                </button>
              </div>

              <!-- Empty state -->
              <div v-if="nodeGroups.length === 0" class="glass-panel rounded-2xl p-8 text-center">
                <Globe2 class="w-12 h-12 mx-auto mb-3 text-[#FDFDFD]/20"/>
                <p class="text-sm text-[#FDFDFD]/50 mb-1">No nodes yet</p>
                <p class="text-xs text-[#FDFDFD]/30">Paste a subscription link or single node URL above to get started.</p>
              </div>

              <!-- Accordion groups -->
              <div class="flex flex-col gap-3">
                <div v-for="group in nodeGroups" :key="group.id" class="glass-panel rounded-2xl overflow-hidden">
                  <!-- Group header (clickable to expand) -->
                  <button @click="toggleSubExpanded(group.id)"
                          class="w-full p-3.5 md:p-4 flex items-center gap-3 text-left hover:bg-[#FDFDFD]/[0.02] transition-colors">
                    <component :is="expandedSubs.has(group.id) ? ChevronDown : ChevronRight"
                               class="w-4 h-4 text-[#FDFDFD]/40 shrink-0 transition-transform"/>
                    <div class="flex-1 min-w-0">
                      <h3 class="font-black text-sm truncate flex items-center gap-2">
                        <span class="truncate">{{ group.name }}</span>
                        <span class="text-[9px] font-mono font-black text-[#9D68FF] bg-[#9D68FF]/10 px-1.5 py-0.5 rounded shrink-0">
                          {{ group.nodes.length }}
                        </span>
                      </h3>
                      <p class="text-[10px] text-[#FDFDFD]/40 mt-0.5 truncate">
                        <span v-if="group.sub">
                          {{ new Date(group.sub.lastUpdated).toLocaleDateString() }}
                          <span v-if="group.sub.expire" :class="group.sub.expire < Date.now() ? 'text-red-400' : 'text-green-400'">
                            В· exp {{ new Date(group.sub.expire).toLocaleDateString() }}
                          </span>
                        </span>
                        <span v-else>Manually added</span>
                      </p>
                    </div>
                    <!-- Refresh (only for subscription groups) -->
                    <button v-if="group.sub" @click.stop="refreshSubscription(group.sub)"
                            class="p-2 bg-[#9D68FF]/10 text-[#9D68FF] rounded-xl hover:bg-[#9D68FF] hover:text-[#121212] transition-colors shrink-0"
                            aria-label="Refresh subscription">
                      <RefreshCcw class="w-3.5 h-3.5"/>
                    </button>
                    <button v-if="group.sub" @click.stop="deleteSubscription(group.id)"
                            class="p-2 text-[#FDFDFD]/30 hover:text-red-400 hover:bg-red-400/10 rounded-xl transition-all shrink-0 hidden sm:flex"
                            aria-label="Delete subscription">
                      <Trash2 class="w-3.5 h-3.5"/>
                    </button>
                  </button>

                  <!-- Group content (expanded) -->
                  <transition name="slide-down">
                    <div v-if="expandedSubs.has(group.id)" class="border-t border-[#FDFDFD]/5">
                      <div v-if="group.nodes.length === 0" class="p-4 text-center text-xs text-[#FDFDFD]/30">
                        No nodes
                      </div>
                      <div v-else class="flex flex-col">
                        <div v-for="profile in getFilteredAndSortedNodes(group.nodes)" :key="profile.id"
                             @click="setActiveProfile(profile.id)"
                             class="node-row p-3 md:p-3.5 flex items-center gap-3 cursor-pointer border-t border-[#FDFDFD]/[0.03] first:border-t-0 transition-colors"
                             :class="profile.active ? 'bg-[#9D68FF]/10' : 'hover:bg-[#FDFDFD]/[0.02]'">
                          <div class="w-8 h-8 shrink-0 rounded-lg flex items-center justify-center transition-colors"
                               :class="profile.active ? 'bg-[#9D68FF] text-[#121212]' : 'bg-[#121212] text-[#FDFDFD]/40'">
                            <CheckCircle2 v-if="profile.active" class="w-4 h-4"/>
                            <span v-else class="text-base leading-none">{{ profile.flag }}</span>
                          </div>
                          <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-1.5">
                              <h4 class="font-bold text-xs md:text-sm truncate">{{ profile.name }}</h4>
                            </div>
                            <div class="flex items-center gap-1.5 mt-0.5">
                              <span class="text-[9px] font-black uppercase tracking-widest text-[#FDFDFD]/40">
                                {{ profile.protocol }}
                              </span>
                              <span v-if="profile.dnsType && profile.dnsType !== 'https'"
                                    class="text-[8px] font-black uppercase tracking-widest px-1 py-0.5 rounded border border-[#FDFDFD]/10 text-[#FDFDFD]/40">
                                {{ profile.dnsType }}
                              </span>
                            </div>
                          </div>
                          <div class="flex items-center gap-2 shrink-0">
                            <span class="text-xs font-mono font-bold w-12 text-right"
                                  :class="profile.ping > 0 && profile.ping < 150 ? 'text-green-400' : (profile.ping > 0 && profile.ping < 300 ? 'text-yellow-400' : (profile.ping === 0 ? 'text-[#FDFDFD]/30' : 'text-red-400'))">
                              <template v-if="profile.ping > 0">{{ profile.ping }}<span class="text-[9px] opacity-60">ms</span></template>
                              <Loader2 v-else-if="profile.ping === 0" class="w-3 h-3 inline animate-spin"/>
                              <span v-else>Г—</span>
                            </span>
                            <button @click.stop="openEditModal(profile)" aria-label="Edit node"
                                    class="p-1.5 text-[#FDFDFD]/30 hover:text-[#9D68FF] hover:bg-[#9D68FF]/10 rounded-lg transition-all">
                              <Settings2 class="w-3.5 h-3.5"/>
                            </button>
                            <button v-if="!group.sub" @click.stop="deleteProfile(profile.id)" aria-label="Delete node"
                                    class="p-1.5 text-[#FDFDFD]/30 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all">
                              <Trash2 class="w-3.5 h-3.5"/>
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  </transition>
                </div>
              </div>
            </div>
          </transition>

          <!-- SETTINGS -->
          <transition name="fade">
            <div v-if="activeTab === 'settings'"
                 class="absolute inset-0 pointer-events-auto overflow-y-auto custom-scroll pr-2 flex flex-col gap-6 pb-6">

              <!-- Security -->
              <div class="glass-panel p-6 md:p-8 rounded-[1.5rem] md:rounded-[2rem] flex flex-col gap-3">
                <h2 class="text-base md:text-lg font-black uppercase tracking-widest flex items-center gap-2 border-b border-[#FDFDFD]/10 pb-3">
                  <ShieldAlert class="w-5 h-5 text-[#9D68FF]"/> Security
                </h2>
                <div class="flex flex-col gap-3 pt-1">
                  <div class="flex items-start gap-3">
                    <CheckCircle2 class="w-4 h-4 text-green-400 mt-0.5 shrink-0"/>
                    <div>
                      <h4 class="font-bold text-sm">DNS Hijack</h4>
                      <p class="text-xs text-[#FDFDFD]/50 mt-0.5">Always on. All port-53 traffic is routed through sing-box (rule: <code class="text-[10px] bg-[#FDFDFD]/5 px-1 py-0.5 rounded">protocol: dns в†’ hijack-dns</code>). DoH/DoT to custom servers is subject to normal routing rules below.</p>
                    </div>
                  </div>
                  <div class="flex items-start gap-3 opacity-60">
                    <Lock class="w-4 h-4 text-[#FDFDFD]/40 mt-0.5 shrink-0"/>
                    <div>
                      <h4 class="font-bold text-sm">Kill Switch <span class="text-[9px] font-black uppercase tracking-widest text-[#FDFDFD]/40 ml-1">Planned</span></h4>
                      <p class="text-xs text-[#FDFDFD]/50 mt-0.5">Drop traffic if VPN disconnects. Will be implemented via <code class="text-[10px] bg-[#FDFDFD]/5 px-1 py-0.5 rounded">netsh advfirewall</code> on Windows / iptables on Android. Tracked separately.</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Split Tunneling -->
              <div class="glass-panel p-8 rounded-[2rem] flex flex-col gap-6">
                <div class="flex items-center justify-between border-b border-[#FDFDFD]/10 pb-4">
                  <h2 class="text-lg font-black uppercase tracking-widest flex items-center gap-2">
                    <ArrowDownToLine class="w-5 h-5 text-[#9D68FF]"/> Split Tunneling
                  </h2>
                </div>
                <div class="flex p-1 bg-[#121212]/50 rounded-xl border border-[#FDFDFD]/5">
                  <button @click="routing.mode = 'global'; saveData()"
                          class="flex-1 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all"
                          :class="routing.mode === 'global' ? 'bg-[#9D68FF] text-[#121212] shadow-md' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    Global
                  </button>
                  <button @click="routing.mode = 'bypass'; saveData()"
                          class="flex-1 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all"
                          :class="routing.mode === 'bypass' ? 'bg-[#9D68FF] text-[#121212] shadow-md' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    Bypass Mode
                  </button>
                  <button @click="routing.mode = 'proxy'; saveData()"
                          class="flex-1 py-2 rounded-lg text-xs font-bold uppercase tracking-wider transition-all"
                          :class="routing.mode === 'proxy' ? 'bg-[#9D68FF] text-[#121212] shadow-md' : 'text-[#FDFDFD]/50 hover:text-[#FDFDFD]'">
                    Only VPN
                  </button>
                </div>
                <div v-if="routing.mode !== 'global'" class="flex flex-col gap-6 mt-2">
                  <div class="flex flex-col gap-3">
                    <label class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50 flex items-center gap-2">
                      <Globe class="w-4 h-4"/> Domains
                      <span class="text-[9px] font-mono normal-case tracking-normal text-[#FDFDFD]/30">(suffix match вЂ” google.com covers mail.google.com)</span>
                    </label>
                    <div class="flex gap-2">
                      <input v-model="newDomainInput" @keyup.enter="addDomain" type="text" placeholder="e.g. google.com"
                             class="flex-1 h-10 bg-[#121212]/30 border border-[#FDFDFD]/10 rounded-xl px-4 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50 transition-all"/>
                      <button @click="addDomain" class="w-10 h-10 rounded-xl bg-[#9D68FF]/10 text-[#9D68FF] hover:bg-[#9D68FF] hover:text-[#121212] flex items-center justify-center transition-all">
                        <Plus class="w-5 h-5"/>
                      </button>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <span v-for="(domain, idx) in routing.domains" :key="idx"
                            class="px-3 py-1.5 rounded-lg bg-[#FDFDFD]/5 border border-[#FDFDFD]/5 text-xs font-mono flex items-center gap-2 group hover:border-red-500/50 transition-colors cursor-default">
                        {{ domain }}<button @click="removeDomain(idx)" class="text-[#FDFDFD]/30 group-hover:text-red-400"><X class="w-3 h-3"/></button>
                      </span>
                    </div>
                  </div>
                  <!-- Per-app routing: РїРѕРєР°Р·С‹РІР°РµРј С‚РѕР»СЊРєРѕ С‚Р°Рј, РіРґРµ СЌС‚Рѕ СЂР°Р±РѕС‚Р°РµС‚. -->
                  <div v-if="platformInfo.apps_supported" class="flex flex-col gap-3">
                    <label class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50 flex items-center gap-2">
                      <Monitor class="w-4 h-4"/> {{ platformInfo.apps_label }}
                    </label>
                    <div class="flex gap-2">
                      <input v-model="newAppInput" @keyup.enter="addApp" type="text" :placeholder="platformInfo.apps_placeholder"
                             class="flex-1 h-10 bg-[#121212]/30 border border-[#FDFDFD]/10 rounded-xl px-4 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50 transition-all"/>
                      <button @click="addApp" class="w-10 h-10 rounded-xl bg-[#9D68FF]/10 text-[#9D68FF] hover:bg-[#9D68FF] hover:text-[#121212] flex items-center justify-center transition-all">
                        <Plus class="w-5 h-5"/>
                      </button>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <span v-for="(app, idx) in routing.apps" :key="idx"
                            class="px-3 py-1.5 rounded-lg bg-[#FDFDFD]/5 border border-[#FDFDFD]/5 text-xs font-mono flex items-center gap-2 group hover:border-red-500/50 transition-colors cursor-default">
                        {{ app }}<button @click="removeApp(idx)" class="text-[#FDFDFD]/30 group-hover:text-red-400"><X class="w-3 h-3"/></button>
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- App Behavior -->
              <div class="glass-panel p-8 rounded-[2rem] flex flex-col gap-6">
                <h2 class="text-lg font-black uppercase tracking-widest flex items-center gap-2 border-b border-[#FDFDFD]/10 pb-4">
                  <MonitorCheck class="w-5 h-5 text-[#9D68FF]"/> App Behavior
                </h2>
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm">Auto-start with System</h4>
                    <p class="text-xs text-[#FDFDFD]/50 mt-1">Launch PulsarVPN silently.</p>
                  </div>
                  <button @click="toggleAutoStart" class="w-14 h-8 rounded-full transition-colors relative"
                          :class="settings.autoStart ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
                    <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                         :class="settings.autoStart ? 'translate-x-6' : 'translate-x-0'"></div>
                  </button>
                </div>
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm">Minimize to Tray on Close</h4>
                    <p class="text-xs text-[#FDFDFD]/50 mt-1">Keep VPN active in background.</p>
                  </div>
                  <button @click="toggleTray" class="w-14 h-8 rounded-full transition-colors relative"
                          :class="settings.minimizeToTray ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
                    <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                         :class="settings.minimizeToTray ? 'translate-x-6' : 'translate-x-0'"></div>
                  </button>
                </div>
              </div>

              <!-- GeoIP Routing -->
              <div class="glass-panel p-8 rounded-[2rem] flex flex-col gap-6">
                <div class="flex items-center justify-between border-b border-[#FDFDFD]/10 pb-4">
                  <h2 class="text-lg font-black uppercase tracking-widest flex items-center gap-2">
                    <Globe2 class="w-5 h-5 text-[#9D68FF]"/> GeoIP Routing
                  </h2>
                </div>
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm">Enable GeoIP</h4>
                    <p class="text-xs text-[#FDFDFD]/50 mt-1">Route traffic based on country IP.</p>
                  </div>
                  <button @click="settings.enableGeoip = !settings.enableGeoip; saveData()"
                          class="w-14 h-8 rounded-full transition-colors relative"
                          :class="settings.enableGeoip ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
                    <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                         :class="settings.enableGeoip ? 'translate-x-6' : 'translate-x-0'"></div>
                  </button>
                </div>
                <div v-if="settings.enableGeoip" class="flex flex-col gap-4 mt-2">
                  <div class="flex justify-between items-center">
                    <span class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50">Country rules</span>
                  </div>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-60 overflow-y-auto custom-scroll pr-2">
                    <div v-for="country in popularCountries" :key="country.code" class="flex items-center justify-between bg-[#121212]/30 p-2 rounded-lg border border-[#FDFDFD]/5">
                      <span class="text-sm flex items-center gap-2"><span class="text-lg">{{ country.flag }}</span> {{ country.name }}</span>
                      <select v-model="settings.geoipRules[country.code]" @change="saveData" class="bg-[#121212] border border-[#FDFDFD]/10 rounded-lg px-2 py-1 text-xs font-mono focus:outline-none focus:border-[#9D68FF]/50">
                        <option value="proxy">Proxy</option>
                        <option value="direct">Direct</option>
                        <option value="block">Block</option>
                      </select>
                    </div>
                  </div>
                  <p class="text-[10px] text-[#FDFDFD]/30 mt-1">Rules are applied after local exceptions. Block will drop traffic.</p>
                </div>
              </div>

            </div>
          </transition>
        </div>
      </main>
    </div>

    <!-- LOGS CONSOLE -->
    <transition name="slide-up">
      <div v-if="showLogs"
           class="absolute bottom-0 left-0 right-0 h-1/3 bg-[#0a0a0a]/95 backdrop-blur-3xl border-t border-[#9D68FF]/20 z-50 flex flex-col shadow-[0_-20px_50px_rgba(0,0,0,0.5)] pointer-events-auto">
        <div class="flex justify-between items-center p-3 border-b border-[#FDFDFD]/10 bg-black/40">
          <span class="text-xs font-black uppercase tracking-widest flex items-center gap-2 text-[#9D68FF]"><Terminal class="w-4 h-4"/> System Logs</span>
          <div class="flex gap-2">
            <button @click="logs =[]" class="p-1.5 rounded-lg text-[#FDFDFD]/30 hover:text-red-400 hover:bg-red-400/10 transition-colors">
              <Trash class="w-4 h-4"/>
            </button>
            <button @click="showLogs = false" class="p-1.5 rounded-lg text-[#FDFDFD]/30 hover:text-white hover:bg-white/10 transition-colors">
              <X class="w-4 h-4"/>
            </button>
          </div>
        </div>
        <div ref="logsContainer" class="flex-1 p-4 overflow-y-auto custom-scroll font-mono text-[10px] leading-relaxed">
          <div v-for="(log, i) in logs" :key="i" class="mb-1" :class="log.includes('ERROR') ? 'text-red-400' : 'text-[#A78BFA]'">{{ log }}</div>
        </div>
      </div>
    </transition>

    <!-- WIZARD OVERLAY -->
    <transition name="fade">
      <div v-if="showWizard" class="fixed inset-0 z-50 bg-[#121212] flex items-center justify-center p-4 sm:p-6 md:p-10 pointer-events-auto">
        <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(157,104,255,0.1)_0%,transparent_70%)]"></div>
        <div class="glass-panel w-full max-w-2xl min-h-[400px] rounded-[3rem] relative overflow-hidden flex flex-col mx-auto">

          <!-- STEP 1: WELCOME -->
          <div v-if="wizardStep === 1" class="flex-1 flex flex-col items-center justify-center gap-6 p-8 sm:p-10 md:p-12 text-center">
            <div class="w-20 h-20 sm:w-24 sm:h-24 rounded-[2rem] bg-[#9D68FF] flex items-center justify-center animate-pulse">
              <Sparkles class="w-10 h-10 sm:w-12 sm:h-12 text-[#121212]" fill="currentColor"/>
            </div>
            <div>
              <h1 class="text-2xl sm:text-3xl md:text-4xl font-black mb-3 sm:mb-4">Welcome to Pulsar<span class="text-[#9D68FF]">VPN</span></h1>
              <p class="text-[#FDFDFD]/50 max-w-md mx-auto leading-relaxed text-sm sm:text-base">
                The next generation tunnel client. Secure, fast, and designed for privacy. Let's get you set up in seconds.
              </p>
            </div>
            <button @click="wizardStep = 2" class="mt-2 sm:mt-4 px-6 sm:px-8 py-3 sm:py-4 bg-[#FDFDFD] text-[#121212] font-black uppercase tracking-widest rounded-2xl hover:bg-[#9D68FF] hover:shadow-[0_0_30px_rgba(157,104,255,0.5)] transition-all flex items-center gap-3">
              Get Started <ArrowRight class="w-4 h-4 sm:w-5 sm:h-5"/>
            </button>
          </div>

          <!-- STEP 2: IMPORT -->
          <div v-if="wizardStep === 2" class="flex-1 flex flex-col items-center justify-center gap-6 p-8 sm:p-10 md:p-12">
            <div class="text-center">
              <h2 class="text-xl sm:text-2xl font-black mb-2">Add Your First Subscription</h2>
              <p class="text-[#FDFDFD]/50 text-sm sm:text-base">Paste your VLESS, Trojan, or Subscription link below.</p>
            </div>
            <div class="w-full max-w-md flex flex-col sm:flex-row gap-4">
              <input v-model="importUrl" @keyup.enter="handleWizardImport" type="text"
                     placeholder="Paste link here..."
                     class="flex-1 h-12 sm:h-14 bg-[#121212]/50 border border-[#FDFDFD]/10 rounded-2xl px-6 text-sm font-mono focus:outline-none focus:border-[#9D68FF]/50 focus:ring-1 focus:ring-[#9D68FF]/50 transition-all"/>
              <button @click="handleWizardImport" :disabled="isImporting"
                      class="h-12 sm:h-14 px-6 sm:px-8 bg-[#9D68FF] text-[#121212] font-black rounded-2xl hover:bg-[#b084ff] transition-all flex items-center justify-center">
                <Loader2 v-if="isImporting" class="w-5 h-5 animate-spin"/>
                <span v-else>IMPORT</span>
              </button>
            </div>
            <button @click="wizardStep = 3" class="text-xs font-bold text-[#FDFDFD]/30 hover:text-[#FDFDFD] uppercase tracking-widest transition-colors">
              Skip for now
            </button>
          </div>

          <!-- STEP 3: READY -->
          <div v-if="wizardStep === 3" class="flex-1 flex flex-col items-center justify-center gap-6 p-8 sm:p-10 md:p-12 text-center">
            <div class="w-16 h-16 sm:w-20 sm:h-20 rounded-full bg-green-500/20 text-green-400 flex items-center justify-center mb-2">
              <Check class="w-8 h-8 sm:w-10 sm:h-10"/>
            </div>
            <div>
              <h2 class="text-2xl sm:text-3xl font-black mb-3 sm:mb-4">You are all set!</h2>
              <p class="text-[#FDFDFD]/50 text-sm sm:text-base">PulsarVPN is ready to connect. Enjoy unrestricted access.</p>
            </div>
            <button @click="completeWizard" class="mt-2 sm:mt-4 px-8 sm:px-10 py-3 sm:py-4 bg-[#9D68FF] text-[#121212] font-black uppercase tracking-widest rounded-2xl hover:shadow-[0_0_30px_rgba(157,104,255,0.4)] transition-all">
              Go to Dashboard
            </button>
          </div>

          <!-- Progress Dots -->
          <div class="absolute bottom-6 left-0 right-0 flex justify-center gap-2">
            <div v-for="step in 3" :key="step"
                 class="w-2 h-2 rounded-full transition-all duration-300"
                 :class="wizardStep === step ? 'w-6 sm:w-8 bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'"></div>
          </div>

        </div>
      </div>
    </transition>

    <!-- EDIT PROFILE MODAL -->
    <transition name="fade">
      <div v-if="editModalOpen" class="fixed inset-0 z-[60] flex items-center justify-center p-4 pointer-events-auto bg-black/60 backdrop-blur-sm">
        <div class="glass-panel w-full max-w-sm rounded-[2rem] p-6 flex flex-col gap-6 shadow-2xl border border-[#9D68FF]/20 relative overflow-hidden">
          <!-- Р”РµРєРѕСЂ -->
          <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-[#9D68FF] to-transparent opacity-50"></div>

          <h3 class="text-lg font-black uppercase tracking-widest flex items-center gap-2">
            <Settings2 class="w-5 h-5 text-[#9D68FF]"/> Node Settings
          </h3>

          <!-- DNS TYPE -->
          <div class="flex flex-col gap-3">
            <label class="text-xs font-bold uppercase tracking-widest text-[#FDFDFD]/50">DNS Resolution Type</label>
            <div class="relative">
              <select v-model="editedDnsType" class="w-full h-12 bg-[#121212]/80 border border-[#FDFDFD]/10 rounded-xl px-4 text-sm focus:outline-none focus:border-[#9D68FF]/50 transition-all appearance-none cursor-pointer">
                <option value="https">HTTPS (DoH) - Encrypted (Default)</option>
                <option value="http">HTTP (Plain) - Not Encrypted</option>
                <option value="udp">UDP - Traditional (Port 53)</option>
              </select>
              <!-- Custom arrow -->
              <div class="absolute inset-y-0 right-4 flex items-center pointer-events-none text-[#FDFDFD]/30">
                <ArrowDownToLine class="w-4 h-4"/>
              </div>
            </div>
            <p class="text-[10px] text-[#FDFDFD]/30 mt-1 leading-relaxed">
              Change this if the server fails to connect due to DNS resolution errors (e.g., standard Hysteria2 instances).
            </p>
          </div>

          <!-- ALLOW INSECURE TOGGLE (Р”РћР‘РђР’Р›Р•РќРћ) -->
          <div class="flex items-center justify-between mt-4 pt-4 border-t border-[#FDFDFD]/10">
            <div>
              <h4 class="font-bold text-sm">Allow Insecure</h4>
              <p class="text-xs text-[#FDFDFD]/50">Skip TLS certificate verification (for selfвЂ‘signed certs).</p>
            </div>
            <button @click="editedAllowInsecure = !editedAllowInsecure"
                    class="w-14 h-8 rounded-full transition-colors relative"
                    :class="editedAllowInsecure ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
              <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                   :class="editedAllowInsecure ? 'translate-x-6' : 'translate-x-0'"></div>
            </button>
          </div>

          <div class="flex items-center justify-between mt-4 pt-2 border-t border-[#FDFDFD]/10">
            <div>
              <h4 class="font-bold text-sm">Use IPv6</h4>
              <p class="text-xs text-[#FDFDFD]/50">Off: IPv4-only (TUN, DNS, server dial). On: allow IPv6 if the network supports it.</p>
            </div>
            <button @click="editedUseIpv6 = !editedUseIpv6"
                    class="w-14 h-8 rounded-full transition-colors relative"
                    :class="editedUseIpv6 ? 'bg-[#9D68FF]' : 'bg-[#FDFDFD]/10'">
              <div class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full transition-transform shadow-md"
                   :class="editedUseIpv6 ? 'translate-x-6' : 'translate-x-0'"></div>
            </button>
          </div>

          <div class="flex gap-3 mt-2">
            <button @click="editModalOpen = false" class="flex-1 h-10 rounded-xl bg-[#FDFDFD]/5 hover:bg-[#FDFDFD]/10 text-xs font-bold uppercase tracking-widest transition-colors">
              Cancel
            </button>
            <button @click="saveProfileSettings" class="flex-1 h-10 rounded-xl bg-[#9D68FF] hover:bg-[#b084ff] text-[#121212] text-xs font-black uppercase tracking-widest transition-all shadow-[0_0_15px_rgba(157,104,255,0.3)]">
              Save
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- TOAST -->
    <transition name="toast">
      <div v-if="toast" class="toast">{{ toast }}</div>
    </transition>

  </div>
</template>

<style scoped>
.glass-panel {
  background: rgba(18, 18, 18, 0.4);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(253, 253, 253, 0.05);
  box-shadow: 0 24px 48px -12px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(253, 253, 253, 0.05);
}
.nav-btn {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 1.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  color: rgba(253, 253, 253, 0.3);
}
.nav-btn:hover {
  background: rgba(253, 253, 253, 0.08);
  color: #fdfdfd;
}
.nav-btn.active {
  background: rgba(157, 104, 255, 0.15);
  color: #9d68ff;
  box-shadow: inset 0 0 0 1px rgba(157, 104, 255, 0.3);
}
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.4s ease, transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}
.slide-up-enter-active, .slide-up-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.slide-up-enter-from, .slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}
.custom-scroll::-webkit-scrollbar {
  width: 4px;
}
.custom-scroll::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(253, 253, 253, 0.1);
  border-radius: 10px;
}
.custom-scroll::-webkit-scrollbar-thumb:hover {
  background: #9d68ff;
}

/* --- РђР”РђРџРўРР’РќРћРЎРўР¬ (mobile < 768px) --- */
@media (max-width: 767px) {
  .sidebar {
    display: none !important;
  }
  .bottom-nav {
    display: flex !important;
  }
  .main-padding {
    padding: 0.75rem !important;
    gap: 0.75rem !important;
  }
  /* РЈРјРµРЅСЊС€Р°РµРј Р»РѕРі-РїР°РЅРµР»СЊ РЅР° РјРѕР±РёР»СЊРЅРѕРј */
  .logs-panel {
    width: 100% !important;
  }
  /* Main area: РѕСЃС‚Р°РІРёС‚СЊ РјРµСЃС‚Рѕ РїРѕРґ BottomNav + safe-area-inset-bottom */
  .main-area {
    padding-bottom: calc(64px + env(safe-area-inset-bottom, 0px) + 0.5rem) !important;
  }
}
/* Desktop: СЃРєСЂС‹С‚СЊ BottomNav */
@media (min-width: 768px) {
  .bottom-nav {
    display: none !important;
  }
  .app-header {
    margin: 0 0 1.5rem 0;
  }
}

/* --- APP HEADER --- */
.app-header {
  display: block;
  border-radius: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  margin: 0 0 0.75rem 0;
}
@media (min-width: 768px) {
  .app-header {
    border-radius: 2rem;
    background: rgba(18, 18, 18, 0.4);
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    border: 1px solid rgba(253, 253, 253, 0.05);
    box-shadow: 0 24px 48px -12px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(253, 253, 253, 0.05);
  }
}
.quick-add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  height: 36px;
  min-width: 36px;
  padding: 0 0.6rem;
  border-radius: 12px;
  background: rgba(157, 104, 255, 0.15);
  color: #9d68ff;
  border: 1px solid rgba(157, 104, 255, 0.2);
  transition: all 0.2s;
  cursor: pointer;
  font: inherit;
}
.quick-add-btn:hover {
  background: rgba(157, 104, 255, 0.25);
  color: #c0a0ff;
}
.quick-add-btn:active {
  transform: scale(0.95);
}

/* --- TOAST --- */
.toast {
  position: fixed;
  left: 50%;
  bottom: calc(80px + env(safe-area-inset-bottom, 0px));
  transform: translateX(-50%);
  z-index: 100;
  padding: 0.65rem 1.25rem;
  background: rgba(18, 18, 18, 0.95);
  border: 1px solid rgba(157, 104, 255, 0.3);
  border-radius: 999px;
  color: #fdfdfd;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4), 0 0 20px rgba(157, 104, 255, 0.2);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  white-space: nowrap;
  max-width: 90vw;
  pointer-events: none;
}
.toast-enter-active, .toast-leave-active {
  transition: opacity 0.25s, transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-enter-from, .toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(8px);
}

/* --- ACCORDION SLIDE --- */
.slide-down-enter-active, .slide-down-leave-active {
  transition: opacity 0.25s ease, max-height 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  overflow: hidden;
}
.slide-down-enter-from, .slide-down-leave-to {
  opacity: 0;
  max-height: 0;
}
.slide-down-enter-to, .slide-down-leave-from {
  opacity: 1;
  max-height: 4000px;
}
.node-row {
  user-select: none;
}
.node-row:active {
  background: rgba(157, 104, 255, 0.05);
}

/* --- FADE for tab content --- */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.4s ease, transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}
.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

/* Connect orb ring (РІСЃРµРіРґР° РІРёРґРµРЅ РЅР° РјРѕР±РёР»СЊРЅРѕРј, С‡С‚РѕР±С‹ РєРЅРѕРїРєР° РЅРµ Р±С‹Р»Р° РЅРµРІРёРґРёРјРѕР№) */
.orb-ring {
  border: 1.5px solid rgba(157, 104, 255, 0.4);
  background: radial-gradient(circle, rgba(157, 104, 255, 0.08) 0%, transparent 70%);
  animation: orb-pulse 3s ease-in-out infinite;
}
.orb-ring:hover {
  border-color: rgba(157, 104, 255, 0.7);
}
@keyframes orb-pulse {
  0%, 100% { box-shadow: 0 0 40px rgba(157, 104, 255, 0.2), inset 0 0 30px rgba(157, 104, 255, 0.15); }
  50% { box-shadow: 0 0 60px rgba(157, 104, 255, 0.35), inset 0 0 40px rgba(157, 104, 255, 0.2); }
}
@media (min-width: 768px) {
  .orb-ring {
    animation: none;
  }
}
</style>
```
pulsar-ui\src\components\BottomNav.vue
```
<script setup lang="ts">
import {
  ShieldCheck, Globe2, Settings, Terminal,
} from 'lucide-vue-next'

const props = defineProps<{
  activeTab: string
  showLogs: boolean
}>()

const emit = defineEmits<{
  (e: 'navigate', tab: string): void
  (e: 'toggle-logs'): void
}>()

const items = [
  { id: 'dashboard', label: 'Shield', icon: ShieldCheck },
  { id: 'nodes', label: 'Nodes', icon: Globe2 },
  { id: 'settings', label: 'Setup', icon: Settings },
] as const
</script>

<template>
  <nav class="bottom-nav glass-panel">
    <button
        v-for="item in items"
        :key="item.id"
        class="nav-item"
        :class="{ active: props.activeTab === item.id }"
        @click="emit('navigate', item.id)"
    >
      <component :is="item.icon" class="w-5 h-5"/>
      <span class="label">{{ item.label }}</span>
    </button>
    <button
        class="nav-item"
        :class="{ active: props.showLogs }"
        @click="emit('toggle-logs')"
    >
      <Terminal class="w-5 h-5"/>
      <span class="label">Logs</span>
    </button>
  </nav>
</template>

<style scoped>
.bottom-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: calc(64px + env(safe-area-inset-bottom, 0px));
  padding-bottom: env(safe-area-inset-bottom, 0px);
  z-index: 50;
  display: flex;
  align-items: stretch;
  justify-content: space-around;
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  border-top-left-radius: 0;
  border-top-right-radius: 0;
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
  border-top: 1px solid rgba(253, 253, 253, 0.05);
  background: rgba(18, 18, 18, 0.85);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  pointer-events: auto;
}
.nav-item {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  color: rgba(253, 253, 253, 0.4);
  transition: color 0.2s, transform 0.15s;
  min-width: 44px;
  min-height: 44px;
  background: transparent;
  border: 0;
  padding: 0;
  cursor: pointer;
  pointer-events: auto;
  -webkit-tap-highlight-color: transparent;
  font: inherit;
  color: rgba(253, 253, 253, 0.4);
}
.nav-item:active {
  transform: scale(0.92);
}
.nav-item.active {
  color: #9d68ff;
}
.label {
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
</style>
```
pulsar-ui\src\components\PulsarSphere.vue
```
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import * as THREE from 'three'

const props = defineProps<{ speed: number; isConnected: boolean; active?: boolean }>()
const containerRef = ref<HTMLElement | null>(null)

let renderer: THREE.WebGLRenderer
let scene: THREE.Scene
let camera: THREE.PerspectiveCamera
let animationFrameId: number
let clock: THREE.Clock
let resizeObserver: ResizeObserver

let group: THREE.Group
let core: THREE.Mesh
let innerCore1: THREE.Points
let innerCore2: THREE.Points
let wireframe: THREE.Mesh
let particles: THREE.Points
let ambient: THREE.Points
let pointLightMain: THREE.PointLight

let coreMaterial: THREE.MeshPhysicalMaterial
let innerMaterial: THREE.PointsMaterial
let wireMaterial: THREE.MeshBasicMaterial
let particleMaterial: THREE.PointsMaterial
let ambientMaterial: THREE.PointsMaterial

let currentVisualSpeed = 0
let stateLerp = 0
let isRunning = false

// РџРµСЂРµРёСЃРїРѕР»СЊР·СѓРµРјС‹Рµ РѕР±СЉРµРєС‚С‹ (Р°Р»Р»РѕРєР°С†РёРё РІ onMounted, РЅРµ РІ animation loop)
const tmpColor = new THREE.Color()
const cOnCore = new THREE.Color(0x2e1065)
const cOnEmissive = new THREE.Color(0x6d28d9)
const cOnInner = new THREE.Color(0xf3e8ff)
const cOnWire = new THREE.Color(0xc084fc)
const cOnParts = new THREE.Color(0xe9d5ff)
const cOnLight = new THREE.Color(0xa855f7)
const cOffCore = new THREE.Color(0x111111)
const cOffEmissive = new THREE.Color(0x000000)
const cOffInner = new THREE.Color(0x222222)
const cOffWire = new THREE.Color(0x333333)
const cOffParts = new THREE.Color(0x444444)
const cOffLight = new THREE.Color(0x4c1d95)

function startLoop() {
  if (isRunning) return
  isRunning = true
  clock.start()
  const animate = () => {
    if (!isRunning) return
    animationFrameId = requestAnimationFrame(animate)

    const delta = Math.min(clock.getDelta(), 0.05)
    const elapsed = clock.getElapsedTime()

    const targetState = props.isConnected ? 1 : 0
    stateLerp = THREE.MathUtils.lerp(stateLerp, targetState, delta * 3.0)

    const targetSpeed = props.isConnected ? (props.speed / 1024 / 1024) : 0
    currentVisualSpeed = THREE.MathUtils.lerp(currentVisualSpeed, targetSpeed, delta * 2.0)

    const activityMult = 0.2 + (stateLerp * 0.8) + (currentVisualSpeed * 0.05)

    group.rotation.y += delta * activityMult * 0.4
    group.rotation.z = Math.sin(elapsed * 0.3) * 0.1

    wireframe.rotation.x -= delta * activityMult * 0.3
    wireframe.rotation.y -= delta * activityMult * 0.2
    wireframe.scale.setScalar(1.05 + Math.cos(elapsed * 2) * (0.02 + stateLerp * 0.03))

    particles.rotation.y += delta * activityMult * 0.6
    particles.rotation.z = Math.cos(elapsed * 0.4) * 0.15
    particles.scale.setScalar(1.0 + stateLerp * 0.2 + Math.sin(elapsed * 5) * (0.02 * currentVisualSpeed))

    ambient.rotation.y -= delta * 0.02
    ambient.rotation.x = Math.sin(elapsed * 0.1) * 0.05

    const breathSpeed = 2 + stateLerp * 3 + currentVisualSpeed
    const breathAmp = 0.01 + stateLerp * 0.04 + (currentVisualSpeed * 0.01)
    core.scale.setScalar(1 + Math.sin(elapsed * breathSpeed) * breathAmp)
    core.rotation.y -= delta * activityMult * 0.5
    core.rotation.x += delta * activityMult * 0.3

    const plasmaSpeed = delta * (1 + stateLerp * 3 + currentVisualSpeed * 2)
    innerCore1.rotation.y += plasmaSpeed
    innerCore1.rotation.x += plasmaSpeed * 0.5

    innerCore2.rotation.y -= plasmaSpeed * 1.2
    innerCore2.rotation.z += plasmaSpeed * 0.8

    // Lerp c РїРµСЂРµРёСЃРїРѕР»СЊР·РѕРІР°РЅРёРµРј tmpColor вЂ” РЅРёРєР°РєРёС… new THREE.Color() РІ РіРѕСЂСЏС‡РµРј С†РёРєР»Рµ
    tmpColor.copy(cOffCore).lerp(cOnCore, stateLerp); coreMaterial.color.copy(tmpColor)
    tmpColor.copy(cOffEmissive).lerp(cOnEmissive, stateLerp); coreMaterial.emissive.copy(tmpColor)
    tmpColor.copy(cOffInner).lerp(cOnInner, stateLerp); innerMaterial.color.copy(tmpColor)
    tmpColor.copy(cOffWire).lerp(cOnWire, stateLerp); wireMaterial.color.copy(tmpColor)
    tmpColor.copy(cOffParts).lerp(cOnParts, stateLerp); particleMaterial.color.copy(tmpColor)

    coreMaterial.emissiveIntensity = (0.2 + stateLerp * 2.0) + (Math.sin(elapsed * 12) * 0.3 * stateLerp)
    innerMaterial.opacity = (stateLerp * 0.9) + (Math.sin(elapsed * 8) * 0.2 * stateLerp)
    wireMaterial.opacity = 0.1 + (stateLerp * 0.3)
    particleMaterial.opacity = 0.1 + (stateLerp * 0.6)

    tmpColor.copy(cOffLight).lerp(cOnLight, stateLerp); pointLightMain.color.copy(tmpColor)
    pointLightMain.intensity = (5 + stateLerp * 35) + (Math.sin(elapsed * 10) * 10 * stateLerp)

    renderer.render(scene, camera)
  }
  animationFrameId = requestAnimationFrame(animate)
}

function stopLoop() {
  if (!isRunning) return
  isRunning = false
  if (animationFrameId) cancelAnimationFrame(animationFrameId)
  animationFrameId = 0
}

onMounted(() => {
  if (!containerRef.value) return

  scene = new THREE.Scene()
  clock = new THREE.Clock(false)

  const width = containerRef.value.clientWidth
  const height = containerRef.value.clientHeight

  camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 100)
  camera.position.z = 7

  renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, preserveDrawingBuffer: true, powerPreference: 'low-power' })
  renderer.setSize(width, height)
  // devicePixelRatio cap = 1.5 РґР°С‘С‚ 2-4x СѓСЃРєРѕСЂРµРЅРёРµ РЅР° 2K/4K РјРѕРЅРёС‚РѕСЂР°С…,
  // РІРёР·СѓР°Р»СЊРЅРѕ СЂР°Р·РЅРёС†Р° РґР»СЏ additive-blended С‡Р°СЃС‚РёС† РЅРµР·Р°РјРµС‚РЅР°.
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5))
  containerRef.value.appendChild(renderer.domElement)

  const ambientLight = new THREE.AmbientLight(0xffffff, 0.3)
  scene.add(ambientLight)

  pointLightMain = new THREE.PointLight(0x9D68FF, 0, 20)
  pointLightMain.position.set(2, 2, 5)
  scene.add(pointLightMain)

  const pointLightSub = new THREE.PointLight(0x4c1d95, 2)
  pointLightSub.position.set(-2, -2, -5)
  scene.add(pointLightSub)

  group = new THREE.Group()
  scene.add(group)

  const coreGeo = new THREE.IcosahedronGeometry(0.8, 2)
  coreMaterial = new THREE.MeshPhysicalMaterial({
    color: 0x111111,
    emissive: 0x000000,
    roughness: 0.1,
    metalness: 0.9,
    transparent: true,
    opacity: 0.7,
    transmission: 0.5,
    wireframe: false,
  })
  core = new THREE.Mesh(coreGeo, coreMaterial)
  group.add(core)

  const createInnerCore = (count: number, radius: number, _size: number) => {
    const geo = new THREE.BufferGeometry()
    const pos = new Float32Array(count * 3)
    for (let i = 0; i < count; i++) {
      const r = Math.pow(Math.random(), 1/3) * radius
      const theta = Math.random() * 2 * Math.PI
      const phi = Math.acos(2 * Math.random() - 1)
      pos[i * 3] = r * Math.sin(phi) * Math.cos(theta)
      pos[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta)
      pos[i * 3 + 2] = r * Math.cos(phi)
    }
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3))
    return geo
  }

  innerMaterial = new THREE.PointsMaterial({
    size: 0.025,
    color: 0x333333,
    transparent: true,
    opacity: 0.0,
    blending: THREE.AdditiveBlending,
    depthWrite: false
  })

  innerCore1 = new THREE.Points(createInnerCore(600, 0.6, 0.03), innerMaterial)
  innerCore2 = new THREE.Points(createInnerCore(400, 0.4, 0.04), innerMaterial)
  core.add(innerCore1)
  core.add(innerCore2)

  const wireGeo = new THREE.IcosahedronGeometry(1.05, 2)
  wireMaterial = new THREE.MeshBasicMaterial({
    color: 0x333333,
    wireframe: true,
    transparent: true,
    opacity: 0.1
  })
  wireframe = new THREE.Mesh(wireGeo, wireMaterial)
  group.add(wireframe)

  const pCount = 2000
  const pGeo = new THREE.BufferGeometry()
  const pPos = new Float32Array(pCount * 3)
  for (let i = 0; i < pCount; i++) {
    const r = 1.6 + Math.random() * 1.5
    const theta = Math.random() * Math.PI * 2
    const phi = Math.acos(2 * Math.random() - 1)
    pPos[i * 3] = r * Math.sin(phi) * Math.cos(theta)
    pPos[i * 3 + 1] = (r * Math.sin(phi) * Math.sin(theta)) * 0.2
    pPos[i * 3 + 2] = r * Math.cos(phi)
  }
  pGeo.setAttribute('position', new THREE.BufferAttribute(pPos, 3))
  particleMaterial = new THREE.PointsMaterial({
    size: 0.03,
    color: 0x444444,
    transparent: true,
    opacity: 0.2,
    sizeAttenuation: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false
  })
  particles = new THREE.Points(pGeo, particleMaterial)
  group.add(particles)

  const dCount = 800
  const dGeo = new THREE.BufferGeometry()
  const dPos = new Float32Array(dCount * 3)
  for (let i = 0; i < dCount * 3; i++) dPos[i] = (Math.random() - 0.5) * 15
  dGeo.setAttribute('position', new THREE.BufferAttribute(dPos, 3))
  ambientMaterial = new THREE.PointsMaterial({
    size: 0.02,
    color: 0x333333,
    transparent: true,
    opacity: 0.2,
    depthWrite: false
  })
  ambient = new THREE.Points(dGeo, ambientMaterial)
  scene.add(ambient)

  // Р—Р°РїСѓСЃРєР°РµРј Р°РЅРёРјР°С†РёСЋ С‚РѕР»СЊРєРѕ РєРѕРіРґР° РІРєР»Р°РґРєР° Р°РєС‚РёРІРЅР°.
  if (props.active !== false) startLoop()

  resizeObserver = new ResizeObserver(() => {
    if (!containerRef.value) return
    const newWidth = containerRef.value.clientWidth
    const newHeight = containerRef.value.clientHeight
    if (newWidth === 0 || newHeight === 0) return
    camera.aspect = newWidth / newHeight
    camera.updateProjectionMatrix()
    renderer.setSize(newWidth, newHeight)
  })
  resizeObserver.observe(containerRef.value)
})

// Р РµР°РєС‚РёРІРЅР°СЏ РѕСЃС‚Р°РЅРѕРІРєР°/РІРѕР·РѕР±РЅРѕРІР»РµРЅРёРµ rAF РїСЂРё СЃРјРµРЅРµ РІРєР»Р°РґРєРё.
// РџСЂРѕРї active = true РЅР° dashboard, РёРЅР°С‡Рµ false. Р‘РµР· СЂРµРЅРґРµСЂРёРЅРіР° РІРЅРµ dashboard
// WebView2 dispatcher РЅРµ РїРѕР»СѓС‡Р°РµС‚ РїРѕСЃС‚РѕСЏРЅРЅС‹С… invalidate-СЃРѕРѕР±С‰РµРЅРёР№.
watch(() => props.active, (v) => {
  if (v) startLoop(); else stopLoop()
})

onBeforeUnmount(() => {
  stopLoop()
  if (resizeObserver && containerRef.value) resizeObserver.unobserve(containerRef.value)

  if (renderer) {
    renderer.dispose()
    renderer.forceContextLoss()
  }

  const disposeMesh = (mesh: THREE.Mesh | THREE.Points) => {
    if (mesh) {
      mesh.geometry.dispose()
      ;(mesh.material as THREE.Material).dispose()
    }
  }

  disposeMesh(core)
  disposeMesh(wireframe)
  disposeMesh(particles)
  disposeMesh(ambient)
  disposeMesh(innerCore1)
  disposeMesh(innerCore2)
})
</script>

<template>
  <div ref="containerRef" class="w-full h-full pointer-events-none absolute inset-0"></div>
</template>
```
pulsar-ui\src\main.ts
```
import { createApp } from 'vue'
import Tres from '@tresjs/core' // <-- РРњРџРћР Рў 3D Р”Р’РР–РљРђ
import './style.css'
import App from './App.vue'

const app = createApp(App)

// РџРѕРґРєР»СЋС‡Р°РµРј TresJS Рє РЅР°С€РµРјСѓ РїСЂРёР»РѕР¶РµРЅРёСЋ (Р‘Р•Р— Р­РўРћР“Рћ 3D РќР• Р РђР‘РћРўРђР•Рў)
app.use(Tres)

app.mount('#app')```
pulsar-ui\src\style.css
```
@import "tailwindcss"; /* РќРѕРІС‹Р№ СЃС‚Р°РЅРґР°СЂС‚ Tailwind v4 */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;800;900&family=JetBrains+Mono:wght@800&display=swap');

/* Р–Р•Р›Р•Р—РћР‘Р•РўРћРќРќР«Р™ Р¤РРљРЎ Р­РљР РђРќРђ */
html, body, #app {
    margin: 0;
    padding: 0;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: #121212; /* РўРІРѕР№ С†РІРµС‚ С„РѕРЅР° */
    color: #FDFDFD;            /* РўРІРѕР№ С†РІРµС‚ С‚РµРєСЃС‚Р° */
}

body {
    font-family: 'Inter', sans-serif;
    user-select: none;
    -webkit-user-select: none;
}

/* РЎРєСЂРѕР»Р»Р±Р°СЂ */
.custom-scroll::-webkit-scrollbar { width: 6px; }
.custom-scroll::-webkit-scrollbar-track { background: transparent; }
.custom-scroll::-webkit-scrollbar-thumb {
    background: rgba(253, 253, 253, 0.1);
    border-radius: 10px;
}
.custom-scroll::-webkit-scrollbar-thumb:hover { background: #9D68FF; }```
pulsar-ui\src\utils\logger.ts
```
/**
 * Р›РѕРіРёСЂРѕРІР°РЅРёРµ РґР»СЏ Vue frontend
 * РћС‚РїСЂР°РІР»СЏРµС‚ Р»РѕРіРё РІ Tauri backend РґР»СЏ Р·Р°РїРёСЃРё РІ С„Р°Р№Р»
 */

export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
  FATAL = 5,
}

const LOG_LEVEL_NAMES: Record<LogLevel, string> = {
  [LogLevel.TRACE]: 'TRACE',
  [LogLevel.DEBUG]: 'DEBUG',
  [LogLevel.INFO]: 'INFO',
  [LogLevel.WARN]: 'WARN',
  [LogLevel.ERROR]: 'ERROR',
  [LogLevel.FATAL]: 'FATAL',
};

class Logger {
  private minLevel: LogLevel = LogLevel.INFO;
  private requestId: string | null = null;

  constructor() {
    // РЈСЂРѕРІРµРЅСЊ РёР· РїРµСЂРµРјРµРЅРЅРѕР№ РѕРєСЂСѓР¶РµРЅРёСЏ РёР»Рё localStorage
    const envLevel = import.meta.env.VITE_LOG_LEVEL;
    const storedLevel = localStorage.getItem('pulsar_log_level');
    const levelStr = envLevel || storedLevel || 'info';
    
    this.minLevel = this.parseLevel(levelStr);
    
    // Р“РµРЅРµСЂРёСЂСѓРµРј request-id РґР»СЏ СЃРµСЃСЃРёРё
    this.requestId = this.generateRequestId();
  }

  private parseLevel(level: string): LogLevel {
    const normalized = level.toLowerCase();
    switch (normalized) {
      case 'trace': return LogLevel.TRACE;
      case 'debug': return LogLevel.DEBUG;
      case 'info': return LogLevel.INFO;
      case 'warn': return LogLevel.WARN;
      case 'error': return LogLevel.ERROR;
      case 'fatal': return LogLevel.FATAL;
      default: return LogLevel.INFO;
    }
  }

  private generateRequestId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.minLevel;
  }

  private formatMessage(level: LogLevel, module: string, message: string, ...args: any[]): string {
    const timestamp = new Date().toISOString();
    const levelName = LOG_LEVEL_NAMES[level];
    const formattedMessage = args.length > 0 
      ? `${message} ${args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ')}`
      : message;
    
    return `[${timestamp}] [${levelName}] [${module}] ${formattedMessage}`;
  }

  private async sendToBackend(level: LogLevel, module: string, message: string, ...args: any[]): Promise<void> {
    if (!this.shouldLog(level)) return;

    try {
      const formatted = this.formatMessage(level, module, message, ...args);
      
      // РћС‚РїСЂР°РІР»СЏРµРј РІ Tauri backend РґР»СЏ Р·Р°РїРёСЃРё РІ С„Р°Р№Р»
      if ((window as any).__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('log_message', {
          level: LOG_LEVEL_NAMES[level],
          module,
          message: formatted,
          requestId: this.requestId,
        }).catch(() => {
          // Fallback РЅР° console РµСЃР»Рё backend РЅРµРґРѕСЃС‚СѓРїРµРЅ
          console.log(formatted);
        });
      } else {
        // Р’ Р±СЂР°СѓР·РµСЂРµ РїСЂРѕСЃС‚Рѕ РІ console
        const consoleMethod = level >= LogLevel.ERROR ? console.error 
          : level >= LogLevel.WARN ? console.warn
          : level >= LogLevel.INFO ? console.info
          : console.debug;
        consoleMethod(formatted);
      }
    } catch (e) {
      console.error('Logger error:', e);
    }
  }

  trace(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.TRACE, module, message, ...args);
  }

  debug(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.DEBUG, module, message, ...args);
  }

  info(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.INFO, module, message, ...args);
  }

  warn(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.WARN, module, message, ...args);
  }

  error(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.ERROR, module, message, ...args);
  }

  fatal(module: string, message: string, ...args: any[]): void {
    this.sendToBackend(LogLevel.FATAL, module, message, ...args);
  }

  setLevel(level: LogLevel): void {
    this.minLevel = level;
    localStorage.setItem('pulsar_log_level', LOG_LEVEL_NAMES[level].toLowerCase());
  }

  getRequestId(): string | null {
    return this.requestId;
  }
}

export const logger = new Logger();
```
pulsar-ui\src\vite-env.d.ts
```
/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
```
pulsar-ui\src-tauri\capabilities\default.json
```
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": [
    "main"
  ],
  "permissions": [
    "core:default",
    "opener:default",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled",
    "store:default"
  ]
}```
pulsar-ui\src-tauri\Cargo.toml
```
[package]
name = "pulsar-ui"
version.workspace = true
description = "Pulsar VPN desktop client (Tauri + Vue)"
authors.workspace = true
license.workspace = true
edition.workspace = true

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
# The `_lib` suffix may seem redundant but it is necessary
# to make the lib name unique and wouldn't conflict with the bin name.
# This seems to be only an issue on Windows, see https://github.com/rust-lang/cargo/issues/8519
name = "pulsar_ui_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "custom-protocol", "image-ico", "image-png"] }
tauri-plugin-opener = "2.0.0"
tauri-plugin-store = "2.0.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0.148"
tokio = { version = "1", features = ["full"] }
tonic = "0.10"
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
base64 = "0.21"
pulsar-shared = { path = "../../pulsar-shared" }
pulsar-daemon = { path = "../../pulsar-daemon" }
tauri-plugin-autostart = "2.0.0"
url = "2.5.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "time"] }
time = { version = "0.3", features = ["formatting"] }
tracing-appender = "0.2"
uuid = { version = "1.0", features = ["v4"] }
tauri-plugin-notification = "2.0"

# РќР° Windows РґРµСЃРєС‚РѕРїРЅС‹Р№ Р±РёР»Рґ РЅРµ РёСЃРїРѕР»СЊР·СѓРµС‚ pulsar-daemon РєР°Рє Р±РёР±Р»РёРѕС‚РµРєСѓ вЂ”
# РЅРѕ РєРѕРјРїРёР»РёСЂРѕРІР°С‚СЊ РµРіРѕ РІСЃС‘ СЂР°РІРЅРѕ РїСЂРёРґС‘С‚СЃСЏ (workspace dep). РќР° Android вЂ”
# РЅР°РѕР±РѕСЂРѕС‚, С‚РѕР»СЊРєРѕ in-process. РњРѕР¶РЅРѕ РїРѕС‚РѕРј Р·Р°РєСЂС‹С‚СЊ feature-С„Р»Р°РіРѕРј, РµСЃР»Рё
# СЂР°Р·РґСѓРµС‚ Р±РёРЅР°СЂСЊ.

[target.'cfg(target_os = "android")'.dependencies]
android_logger = "0.14"
log = "0.4"
jni = "0.21"
ndk-context = "0.1"
libc = "0.2"

```
pulsar-ui\src-tauri\src\backend\mod.rs
```
//! РџР»Р°С‚С„РѕСЂРјРµРЅРЅР°СЏ Р°Р±СЃС‚СЂР°РєС†РёСЏ VPN-Р±СЌРєРµРЅРґР° РґР»СЏ Tauri-РїСЂРёР»РѕР¶РµРЅРёСЏ.
//!
//! РђСЂС…РёС‚РµРєС‚СѓСЂР°:
//! - РќР° **Windows** Tauri-UI РїРѕРґРЅРёРјР°РµС‚ `pulsar-daemon.exe` (РѕС‚РґРµР»СЊРЅС‹Р№
//!   РїСЂРѕС†РµСЃСЃ) Рё РѕР±С‰Р°РµС‚СЃСЏ СЃ РЅРёРј РїРѕ gRPC. Р­С‚Рѕ РґР°С‘С‚ РЅР°РґС‘Р¶РЅСѓСЋ РёР·РѕР»СЏС†РёСЋ Рё
//!   РІРѕР·РјРѕР¶РЅРѕСЃС‚СЊ РїРµСЂРµР·Р°РїСѓСЃС‚РёС‚СЊ РґРµРјРѕРЅ, РЅРµ СѓР±РёРІР°СЏ UI.
//! - РќР° **Android** Tauri-UI Р¶РёРІС‘С‚ РІ РѕРґРЅРѕРј РїСЂРѕС†РµСЃСЃРµ, РїРѕСЌС‚РѕРјСѓ gRPC
//!   РЅРµ РЅСѓР¶РµРЅ вЂ” РјС‹ РёСЃРїРѕР»СЊР·СѓРµРј `pulsar_daemon::VpnEngine` РЅР°РїСЂСЏРјСѓСЋ.
//!   РњРµС‚СЂРёРєРё Рё СЃРѕСЃС‚РѕСЏРЅРёРµ РїСЂРёС…РѕРґСЏС‚ С‡РµСЂРµР· С‚РѕС‚ Р¶Рµ [`MetricsEvent`] /
//!   `VpnState`, С‡С‚Рѕ Рё РІ gRPC-РІР°СЂРёР°РЅС‚Рµ, РЅРѕ Р±РµР· СЃРµС‚РµРІС‹С… С…РѕРїРѕРІ.
//!
//! [`Backend`] вЂ” СЌС‚Рѕ enum, РІР°СЂРёР°РЅС‚С‹ РєРѕС‚РѕСЂРѕРіРѕ СЃРѕРґРµСЂР¶Р°С‚ РїР»Р°С‚С„РѕСЂРјРµРЅРЅС‹Рµ
//! РґРµС‚Р°Р»Рё (`VpnServiceClient` РґР»СЏ Windows, `VpnEngine` РґР»СЏ Android).
//! РњС‹ РІС‹Р±СЂР°Р»Рё enum, Р° РЅРµ `dyn Trait`, С‡С‚РѕР±С‹:
//!   1. РЅРµ РїР»Р°С‚РёС‚СЊ Р·Р° РІРёСЂС‚СѓР°Р»СЊРЅС‹Рµ РІС‹Р·РѕРІС‹ РЅР° РіРѕСЂСЏС‡РµРј РїСѓС‚Рё;
//!   2. РЅРµ РІРѕР·РёС‚СЊСЃСЏ СЃ `async fn in trait` (РґРѕ СЃС‚Р°Р±РёР»РёР·Р°С†РёРё РІ Rust 1.75+
//!      СЌС‚Рѕ С‚СЂРµР±РѕРІР°Р»Рѕ `BoxFuture` РёР»Рё `async-trait`);
//!   3. С…СЂР°РЅРёС‚СЊ `Clone` Р±РµР· `Arc<Mutex<...>>` (Tauri-State С‚СЂРµР±СѓРµС‚ Clone).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pulsar_daemon::vpn_engine::{ConnectRequest, MetricsEvent, VpnState};
use pulsar_shared::vpn::RoutingConfig;

#[allow(unused_imports)]
use crate::logger as _logger_unused;

/// Р“Р»РѕР±Р°Р»СЊРЅР°СЏ СЂСѓС‡РєР° РґР»СЏ С„Р»Р°РіР° "СЃС‚СЂРёРјРёРј Р»Рё Р»РѕРіРё РІ webview".
///
/// Р”РѕСЃС‚СѓРї Рє РЅРµР№ РЅСѓР¶РµРЅ РёР· Tauri-РєРѕРјР°РЅРґС‹ `set_logs_streaming`, РєРѕС‚РѕСЂР°СЏ
/// РЅРµ РїРѕР»СѓС‡Р°РµС‚ `AppState`. РЎР°РјР° СЂСѓС‡РєР° С…СЂР°РЅРёС‚СЃСЏ РІ `OnceLock`, С‡С‚РѕР±С‹
/// РЅРµ РїР»РѕРґРёС‚СЊ РіР»РѕР±Р°Р»СЊРЅС‹С… `static mut`.
static LOGS_STREAMING_FLAG: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

/// РЎРѕС…СЂР°РЅСЏРµС‚ РіР»РѕР±Р°Р»СЊРЅС‹Р№ С„Р»Р°Рі. Р’С‹Р·С‹РІР°РµС‚СЃСЏ РѕРґРёРЅ СЂР°Р· РёР· `main()`.
pub fn set_logs_streaming_handle(flag: Arc<AtomicBool>) {
    let _ = LOGS_STREAMING_FLAG.set(flag);
}

/// РЈСЃС‚Р°РЅР°РІР»РёРІР°РµС‚ Р·РЅР°С‡РµРЅРёРµ РіР»РѕР±Р°Р»СЊРЅРѕРіРѕ С„Р»Р°РіР°. РќР° Android вЂ” noop.
pub fn set_logs_streaming_global(enable: bool) {
    if let Some(flag) = LOGS_STREAMING_FLAG.get() {
        flag.store(enable, Ordering::Relaxed);
    }
}

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "windows")]
mod windows;

/// Р РµР·СѓР»СЊС‚Р°С‚ `connect`/`disconnect` РІ РІРёРґРµ РїСЂРѕСЃС‚РѕРіРѕ `(state, message)`,
/// РєРѕС‚РѕСЂС‹Р№ Р»РµРіРєРѕ РїСЂРµРІСЂР°С‰Р°РµС‚СЃСЏ РІ gRPC `StatusResponse` РёР»Рё РІ HTTP/JSON
/// РѕС‚РІРµС‚ РґР»СЏ Android (РµСЃР»Рё РІ Р±СѓРґСѓС‰РµРј РїРѕРЅР°РґРѕР±РёС‚СЃСЏ).
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

/// Р§С‚Рѕ РјС‹ РґРµР»Р°РµРј РїРѕ `toggle_vpn(connect=true|false)` вЂ” РЅР° СѓСЂРѕРІРЅРµ API
/// Р±СЌРєРµРЅРґР° СЌС‚Рѕ РѕРґРёРЅ РјРµС‚РѕРґ, РїРѕС‚РѕРјСѓ С‡С‚Рѕ Рё connect, Рё disconnect вЂ”
/// РµСЃС‚РµСЃС‚РІРµРЅРЅС‹Рµ РѕРїРµСЂР°С†РёРё РЅР° VPN-РґРІРёР¶РєРµ.
#[derive(Debug, Clone)]
pub struct ToggleRequest {
    pub connect: bool,
    pub profile_link: String,
    pub routing: RoutingConfig,
}

/// РџР»Р°С‚С„РѕСЂРјРµРЅРЅР°СЏ РѕР±С‘СЂС‚РєР°. РЎРѕРґРµСЂР¶РёС‚ РІРЅСѓС‚СЂРё enum СЃ РґРµС‚Р°Р»СЏРјРё, РїРѕС‚РѕРјСѓ С‡С‚Рѕ
/// СЂР°Р·РЅС‹Рµ РїР»Р°С‚С„РѕСЂРјС‹ РёРјРµСЋС‚ РїСЂРёРЅС†РёРїРёР°Р»СЊРЅРѕ СЂР°Р·РЅС‹Рµ РїРѕР»СЏ (gRPC client vs
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
    /// РЎРѕР·РґР°С‘С‚ Р±СЌРєРµРЅРґ РґР»СЏ С‚РµРєСѓС‰РµР№ РїР»Р°С‚С„РѕСЂРјС‹.
    ///
    /// РџСЂРёРЅРёРјР°РµС‚ `&tauri::App`, РїРѕС‚РѕРјСѓ С‡С‚Рѕ:
    /// - РќР° Windows РЅСѓР¶РµРЅ `resource_dir()` (РіРґРµ Р»РµР¶РёС‚ `pulsar-daemon.exe`)
    ///   Рё `AppHandle` (РґР»СЏ СЌРјРёС‚Р° `vpn_log`).
    /// - РќР° Android вЂ” РїР°СЂР°РјРµС‚СЂ РёРіРЅРѕСЂРёСЂСѓРµС‚СЃСЏ.
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

    /// Р’РѕР·РІСЂР°С‰Р°РµС‚ `true`, РµСЃР»Рё РЅР° СЌС‚РѕР№ РїР»Р°С‚С„РѕСЂРјРµ Р±СЌРєРµРЅРґ вЂ” РѕС‚РґРµР»СЊРЅС‹Р№
    /// РїСЂРѕС†РµСЃСЃ. РќР° Android СЌС‚Рѕ РІСЃРµРіРґР° `false`. РСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ UI-СЃС‚РѕСЂРѕРЅРѕР№
    /// РґР»СЏ РѕС‚РѕР±СЂР°Р¶РµРЅРёСЏ РёРЅРґРёРєР°С‚РѕСЂР° "РґРµРјРѕРЅ" РІ РЅР°СЃС‚СЂРѕР№РєР°С….
    #[allow(dead_code)]
    pub fn is_external_process(&self) -> bool {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(_) => true,
            #[cfg(target_os = "android")]
            BackendKind::Android(_) => false,
        }
    }

    /// РџРѕРґРєР»СЋС‡РёС‚СЊСЃСЏ РёР»Рё РѕС‚РєР»СЋС‡РёС‚СЊСЃСЏ. Р’РѕР·РІСЂР°С‰Р°РµС‚ `(state, message)`,
    /// РєРѕС‚РѕСЂС‹Р№ UI РјРѕР¶РµС‚ РїРѕРєР°Р·Р°С‚СЊ РІ РЅРѕС‚РёС„РёРєР°С†РёРё.
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

    /// РўРµРєСѓС‰РµРµ СЃРѕСЃС‚РѕСЏРЅРёРµ вЂ” РґР»СЏ СЂРµРґРєРёС… РѕРїСЂРѕСЃРѕРІ РёР· UI.
    #[allow(dead_code)]
    pub async fn state(&self) -> VpnState {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => b.state().await,
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => b.state().await,
        }
    }

    /// РџРѕРґРїРёСЃРєР° РЅР° РїРѕС‚РѕРє РјРµС‚СЂРёРє. UI РІС‹Р·С‹РІР°РµС‚ РѕРґРёРЅ СЂР°Р· РїСЂРё СЃС‚Р°СЂС‚Рµ;
    /// РІ Receiver РїСЂРёС…РѕРґСЏС‚ `MetricsEvent` СЃРѕ СЃРєРѕСЂРѕСЃС‚СЏРјРё.
    /// `None` вЂ” РµСЃР»Рё РїРѕРґРїРёСЃРєР° СѓР¶Рµ Р±С‹Р»Р° РІС‹РґР°РЅР° (РІС‚РѕСЂРѕР№ РІС‹Р·РѕРІ).
    pub async fn subscribe_metrics(&self) -> Option<tokio::sync::mpsc::Receiver<MetricsEvent>> {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => b.subscribe_metrics().await,
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => b.subscribe_metrics().await,
        }
    }

    /// РљРѕСЂСЂРµРєС‚РЅРѕ РѕСЃС‚Р°РЅРѕРІРёС‚СЊ VPN Рё (РµСЃР»Рё РїСЂРёРјРµРЅРёРјРѕ) СѓР±РёС‚СЊ РґРµРјРѕРЅ.
    /// Р’С‹Р·С‹РІР°РµС‚СЃСЏ РЅР° РІС‹С…РѕРґРµ РёР· РїСЂРёР»РѕР¶РµРЅРёСЏ.
    pub async fn shutdown(&self) {
        match &*self.inner {
            #[cfg(target_os = "windows")]
            BackendKind::Windows(b) => b.shutdown().await,
            #[cfg(target_os = "android")]
            BackendKind::Android(b) => b.shutdown().await,
        }
    }
}

// Р’РЅСѓС‚СЂРµРЅРЅРёРµ РјРѕРґСѓР»Рё СЌРєСЃРїРѕСЂС‚РёСЂСѓСЋС‚СЃСЏ С‡РµСЂРµР· `mod` РІС‹С€Рµ. main.rs СЃ РЅРёРјРё
// РЅР°РїСЂСЏРјСѓСЋ РЅРµ СЂР°Р±РѕС‚Р°РµС‚ вЂ” С‚РѕР»СЊРєРѕ С‡РµСЂРµР· `Backend`.
```
pulsar-ui\src-tauri\src\backend\windows.rs
```
//! Windows-Р±СЌРєРµРЅРґ: Р·Р°РїСѓСЃРєР°РµС‚ `pulsar-daemon.exe` Рё РѕР±С‰Р°РµС‚СЃСЏ СЃ РЅРёРј РїРѕ gRPC.
//!
//! Р–РёР·РЅРµРЅРЅС‹Р№ С†РёРєР»:
//! 1. `WindowsBackend::new()`:
//!    - С‚Р°СЃРєРєРёР»Р»РёРј "ghost"-РїСЂРѕС†РµСЃСЃС‹ (РµСЃР»Рё Tauri СѓР±РёР»Рё СЂР°РЅСЊС€Рµ РґРµРјРѕРЅР°);
//!    - СЃРїР°РІРЅРёРј `pulsar-daemon.exe <token>` РєР°Рє РґРѕС‡РµСЂРЅРёР№ РїСЂРѕС†РµСЃСЃ;
//!    - РІ С„РѕРЅРµ РїРѕРґРєР»СЋС‡Р°РµРјСЃСЏ Рє gRPC `127.0.0.1:50051` СЃ СЂРµС‚СЂР°СЏРјРё;
//!    - СЃРѕС…СЂР°РЅСЏРµРј `Client` + `Child` РІ `Arc<Mutex<...>>`.
//! 2. `toggle()`: РІС‹Р·С‹РІР°РµС‚ gRPC-РјРµС‚РѕРґ `ToggleConnection`.
//! 3. `subscribe_metrics()`: gRPC-СЃС‚СЂРёРј `StreamMetrics` Р·Р°РІРѕСЂР°С‡РёРІР°РµС‚СЃСЏ
//!    РІ `mpsc::Receiver<MetricsEvent>`.
//! 4. `shutdown()`: gRPC `Shutdown` + kill РїСЂРѕС†РµСЃСЃР° РґРµРјРѕРЅР°.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{Emitter, Manager};

use tokio::sync::{Mutex, mpsc};
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;
use uuid::Uuid;

use pulsar_daemon::vpn_engine::{ConnectRequest, MetricsEvent, VpnState};
use pulsar_shared::vpn::vpn_service_client::VpnServiceClient;
use pulsar_shared::vpn::{ConnectionRequest, Empty, Metrics, RoutingConfig};

/// gRPC-РєР»РёРµРЅС‚, РѕР±С‘СЂРЅСѓС‚С‹Р№ РІ interceptor СЃ С‚РѕРєРµРЅРѕРј.
type GrpcClient = VpnServiceClient<InterceptedService<Channel, AuthInterceptor>>;

#[derive(Clone)]
struct AuthInterceptor {
    token: String,
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request
            .metadata_mut()
            .insert("x-pulsar-token", self.token.parse().unwrap());
        let request_id = Uuid::new_v4().to_string();
        request
            .metadata_mut()
            .insert("x-request-id", request_id.parse().unwrap());
        Ok(request)
    }
}

/// РЎРѕСЃС‚РѕСЏРЅРёРµ Windows-Р±СЌРєРµРЅРґР°. Р”РµР»Р°РµРј `pub(crate)` РїРѕР»СЏ РїСЂРёРІР°С‚РЅС‹РјРё
/// С‡РµСЂРµР· `Arc<Mutex<...>>` Рё РїСЂРѕРєСЃРёСЂСѓРµРј С‚РѕР»СЊРєРѕ РЅСѓР¶РЅС‹Рµ РјРµС‚РѕРґС‹.
pub struct WindowsBackend {
    /// РўРµРєСѓС‰РёР№ gRPC-РєР»РёРµРЅС‚. `None` вЂ” РїРѕРєР° РЅРµ РїРѕРґРєР»СЋС‡РёР»РёСЃСЊ (РґРµРјРѕРЅ РµС‰С‘
    /// РЅРµ РїСЂРёРЅСЏР» СЃРѕРµРґРёРЅРµРЅРёРµ).
    client: Arc<Mutex<Option<GrpcClient>>>,
    /// Р”РѕС‡РµСЂРЅРёР№ РїСЂРѕС†РµСЃСЃ РґРµРјРѕРЅР°. Р”РµСЂР¶РёРј, С‡С‚РѕР±С‹ СѓР±РёС‚СЊ РїСЂРё РІС‹С…РѕРґРµ.
    daemon_process: Arc<Mutex<Option<std::process::Child>>>,
    /// РџРѕРґРїРёСЃРєР° РЅР° РјРµС‚СЂРёРєРё РІС‹РґР°С‘С‚СЃСЏ РѕРґРЅРѕРєСЂР°С‚РЅРѕ.
    metrics_rx: Arc<Mutex<Option<mpsc::Receiver<MetricsEvent>>>>,
    /// Tauri AppHandle РЅСѓР¶РµРЅ, С‡С‚РѕР±С‹ СЌРјРёС‚РёС‚СЊ `vpn_log`.
    app_handle: tauri::AppHandle,
}

impl WindowsBackend {
    /// РџСѓР±Р»РёС‡РЅР°СЏ С‚РѕС‡РєР° РІС…РѕРґР°, РІС‹Р·С‹РІР°РµРјР°СЏ РёР· `setup()`. РЎРїР°РІРЅРёС‚ РґРµРјРѕРЅ,
    /// РїРѕРґРєР»СЋС‡Р°РµС‚СЃСЏ Рє gRPC, РІРѕР·РІСЂР°С‰Р°РµС‚ РіРѕС‚РѕРІС‹Р№ Р±СЌРєРµРЅРґ.
    pub fn attach_app(app: &tauri::App, logs_streaming: Arc<AtomicBool>) -> Result<Self, String> {
        let time_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let session_token = format!("pulsar-{}-{}", std::process::id(), time_nanos);

        // --- Р—Р°С‡РёСЃС‚РєР° "ghost"-РїСЂРѕС†РµСЃСЃРѕРІ ---
        kill_ghosts();

        // --- РЎС‚Р°СЂС‚ РґРµРјРѕРЅР° ---
        let daemon_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("resource_dir: {e}"))?
            .join("binaries")
            .join("pulsar-daemon.exe");
        if !daemon_path.exists() {
            return Err(format!(
                "pulsar-daemon.exe not found at {}",
                daemon_path.display()
            ));
        }

        let mut cmd = Command::new(&daemon_path);
        cmd.arg(&session_token)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let mut child = cmd.spawn().map_err(|e| format!("spawn daemon: {e}"))?;
        let pid = child.id();
        tracing::info!(pid, "pulsar-daemon spawned");

        let app_handle = app.handle().clone();

        // --- Р›РѕРі-С„Р»Р°С€РµСЂ (Р±Р°С‚С‡РёРЅРі РІС‹РІРѕРґР° РґРµРјРѕРЅР° РІ webview) ---
        spawn_log_flusher(
            child.stdout.take(),
            child.stderr.take(),
            app_handle.clone(),
            logs_streaming.clone(),
        );

        // --- РЎРѕС…СЂР°РЅСЏРµРј child РґР»СЏ РїРѕСЃР»РµРґСѓСЋС‰РµРіРѕ kill ---
        let daemon_process = Arc::new(Mutex::new(None));
        let process_store = daemon_process.clone();
        tauri::async_runtime::spawn(async move {
            *process_store.lock().await = Some(child);
        });

        // --- gRPC-РєРѕРЅРЅРµРєС‚ РІ С„РѕРЅРµ СЃ СЂРµС‚СЂР°СЏРјРё ---
        let client = Arc::new(Mutex::new(None));
        let client_store = client.clone();
        let token_for_grpc = session_token.clone();
        tauri::async_runtime::spawn(async move {
            let mut attempts = 0u32;
            loop {
                attempts += 1;
                match Channel::from_static("http://127.0.0.1:50051")
                    .connect()
                    .await
                {
                    Ok(channel) => {
                        tracing::info!(attempt = attempts, "gRPC connected to pulsar-daemon");
                        let c = VpnServiceClient::with_interceptor(
                            channel,
                            AuthInterceptor {
                                token: token_for_grpc.clone(),
                            },
                        );
                        *client_store.lock().await = Some(c);
                        return;
                    }
                    Err(e) => {
                        if attempts % 5 == 1 {
                            tracing::warn!(attempt = attempts, error = %e, "gRPC connect retrying");
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(Self {
            client,
            daemon_process,
            metrics_rx: Arc::new(Mutex::new(None)),
            app_handle,
        })
    }

    pub async fn connect(&self, req: ConnectRequest) -> Result<VpnState, String> {
        let mut g = self.client.lock().await;
        let client = g
            .as_mut()
            .ok_or_else(|| "Daemon is not connected to UI yet. Please wait.".to_string())?;
        let proto_req = ConnectionRequest {
            connect: true,
            profile_link: req.profile_link,
            routing_config: Some(req.routing),
        };
        client
            .toggle_connection(proto_req)
            .await
            .map(|r| {
                let r = r.into_inner();
                parse_proto_state(&r.state, &r.message)
            })
            .map_err(|e| {
                let msg = e.message().to_string();
                emit_error_notification(&self.app_handle, &msg);
                format!("gRPC: {msg}")
            })
    }

    pub async fn disconnect(&self) -> Result<VpnState, String> {
        let mut g = self.client.lock().await;
        let client = g
            .as_mut()
            .ok_or_else(|| "Daemon is not connected".to_string())?;
        let req = ConnectionRequest {
            connect: false,
            profile_link: String::new(),
            routing_config: Some(RoutingConfig::default()),
        };
        client
            .toggle_connection(req)
            .await
            .map(|r| {
                let r = r.into_inner();
                parse_proto_state(&r.state, &r.message)
            })
            .map_err(|e| e.message().to_string())
    }

    #[allow(dead_code)] // РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ С‡РµСЂРµР· Backend::state() (enum-dispatch)
    pub async fn state(&self) -> VpnState {
        let mut g = self.client.lock().await;
        let Some(client) = g.as_mut() else {
            return VpnState::Disconnected;
        };
        match client.get_status(Empty {}).await {
            Ok(r) => {
                let r = r.into_inner();
                parse_proto_state(&r.state, &r.message)
            }
            Err(_) => VpnState::Disconnected,
        }
    }

    pub async fn subscribe_metrics(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        // РћС‚РґР°С‘Рј РїРѕРґРїРёСЃРєСѓ РѕРґРЅРѕРєСЂР°С‚РЅРѕ. mpsc::Receiver вЂ” РЅРµ Clone, РїРѕСЌС‚РѕРјСѓ
        // РїСЂРѕСЃС‚Рѕ РѕС‚РґР°С‘Рј РµРіРѕ РІС‹Р·С‹РІР°СЋС‰РµРјСѓ; РїРѕРІС‚РѕСЂРЅС‹Р№ РІС‹Р·РѕРІ РІРµСЂРЅС‘С‚ None.
        let mut g = self.metrics_rx.lock().await;
        if g.take().is_some() {
            return None;
        }
        let (tx, rx) = mpsc::channel::<MetricsEvent>(8);
        let client_arc = self.client.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let stream_result = {
                    let mut g = client_arc.lock().await;
                    match g.as_mut() {
                        Some(c) => Some(c.stream_metrics(Empty {}).await),
                        None => None,
                    }
                };
                let Some(stream_result) = stream_result else {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                };
                let mut stream = match stream_result {
                    Ok(s) => s.into_inner(),
                    Err(e) => {
                        tracing::warn!(error = %e, "stream_metrics failed, retrying");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                };
                loop {
                    match stream.message().await {
                        Ok(Some(Metrics {
                            download_speed,
                            upload_speed,
                        })) => {
                            if tx
                                .send(MetricsEvent {
                                    download_speed,
                                    upload_speed,
                                })
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            tracing::warn!(error = %e, "metrics stream error, retrying");
                            break;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });
        // `g` (MutexGuard) РґСЂРѕРїР°РµС‚СЃСЏ Р·РґРµСЃСЊ, РїРѕС‚РѕРјСѓ С‡С‚Рѕ `take()` Р·Р°Р±СЂР°Р»
        // РїСЂРѕС€Р»РѕРµ Р·РЅР°С‡РµРЅРёРµ. Р’РѕР·РІСЂР°С‰Р°РµРј СЃР°Рј `rx`.
        drop(g);
        Some(rx)
    }

    pub async fn shutdown(&self) {
        // gRPC shutdown
        if let Some(c) = self.client.lock().await.as_mut() {
            let _ = c.shutdown(Empty {}).await;
        }
        // РЈР±РёРІР°РµРј РїСЂРѕС†РµСЃСЃ
        if let Some(mut child) = self.daemon_process.lock().await.take() {
            #[cfg(target_os = "windows")]
            {
                let pid = child.id();
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .status();
            }
            let _ = child.kill();
        }
    }
}

#[cfg(target_os = "windows")]
fn kill_ghosts() {
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "pulsar-daemon.exe", "/T"])
        .creation_flags(0x08000000)
        .status();
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "xray.exe", "/T"])
        .creation_flags(0x08000000)
        .status();
}

#[cfg(not(target_os = "windows"))]
fn kill_ghosts() {}

/// Р‘Р°С‚С‡РёРЅРі stdout/stderr РґРµРјРѕРЅР° РІ `vpn_log` СЃРѕР±С‹С‚РёСЏ webview.
/// РљРѕРґ РїСЂР°РєС‚РёС‡РµСЃРєРё РёРґРµРЅС‚РёС‡РµРЅ С‚РѕРјСѓ, С‡С‚Рѕ СЂР°РЅСЊС€Рµ Р¶РёР» РІ main.rs setup().
fn spawn_log_flusher(
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
    app: tauri::AppHandle,
    flag: Arc<AtomicBool>,
) {
    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<(u8, String)>();
    let flush_app = app.clone();
    let flush_flag = flag.clone();
    tauri::async_runtime::spawn(async move {
        const FLUSH_INTERVAL_MS: u64 = 200;
        const MAX_LINES_PER_BATCH: usize = 64;
        const MAX_BUFFER_LINES: usize = 500;
        let mut buf: Vec<(u8, String)> = Vec::with_capacity(MAX_LINES_PER_BATCH);
        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(FLUSH_INTERVAL_MS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                biased;
                maybe = log_rx.recv() => {
                    let mut closed = false;
                    if let Some(item) = maybe {
                        if buf.len() < MAX_BUFFER_LINES { buf.push(item); }
                    } else { closed = true; }
                    while buf.len() < MAX_BUFFER_LINES {
                        match log_rx.try_recv() {
                            Ok(more) => buf.push(more),
                            Err(_) => break,
                        }
                    }
                    let should_flush = buf.len() >= MAX_LINES_PER_BATCH || closed;
                    if should_flush && !buf.is_empty() {
                        if flush_flag.load(Ordering::Relaxed) {
                            let mut combined = String::with_capacity(buf.len() * 80);
                            for (_, line) in buf.drain(..) {
                                combined.push_str(&line);
                                combined.push('\n');
                            }
                            let _ = flush_app.emit("vpn_log", combined);
                        } else { buf.clear(); }
                    }
                    if closed { break; }
                }
                _ = ticker.tick() => {
                    if buf.is_empty() { continue; }
                    if flush_flag.load(Ordering::Relaxed) {
                        let mut combined = String::with_capacity(buf.len() * 80);
                        for (_, line) in buf.drain(..) {
                            combined.push_str(&line);
                            combined.push('\n');
                        }
                        let _ = flush_app.emit("vpn_log", combined);
                    } else { buf.clear(); }
                }
            }
        }
    });

    if let Some(stdout) = stdout {
        let tx = log_tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end_matches(['\r', '\n']);
                        if trimmed.is_empty() {
                            continue;
                        }
                        if tx.send((0, trimmed.to_string())).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
    if let Some(stderr) = stderr {
        let tx = log_tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim_end_matches(['\r', '\n']);
                        if trimmed.is_empty() {
                            continue;
                        }
                        if tx.send((1, format!("[CRITICAL] {}", trimmed))).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

fn parse_proto_state(state: &str, message: &str) -> VpnState {
    match state {
        "Connected" => VpnState::Connected,
        "Connecting" => VpnState::Connecting,
        "Disconnecting" => VpnState::Disconnecting,
        "Disconnected" => VpnState::Disconnected,
        "Error" => VpnState::Error(message.to_string()),
        _ => VpnState::Disconnected,
    }
}

fn emit_error_notification(app: &tauri::AppHandle, msg: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("PulsarVPN Error")
        .body(format!("Connection Failed: {}", msg))
        .show();
}
```
pulsar-ui\src-tauri\src\lib.rs
```
mod backend;
mod logger;

// Android VPN bridge module (included when targeting android)
// #[cfg(target_os = "android")]
// mod android;

use std::collections::HashMap;
use std::sync::Arc;

use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use backend::{Backend, ToggleRequest};
#[cfg(target_os = "android")]
use pulsar_daemon::vpn_engine::MetricsEvent;
use pulsar_daemon::vpn_engine::VpnState;
use pulsar_shared::vpn::routing_config::Mode;
use pulsar_shared::vpn::{DnsType, RoutingConfig};

#[derive(Clone)]
struct AppState {
    backend: Backend,
}

#[derive(serde::Deserialize)]
struct UIRoutingConfig {
    mode: String,
    domains: Vec<String>,
    apps: Vec<String>,
    enable_geoip: Option<bool>,
    geoip_rules: Option<HashMap<String, String>>,
    dns_type: Option<String>,
    allow_insecure: Option<bool>,
    use_ipv6: Option<bool>,
}

#[derive(serde::Deserialize)]
struct TogglePayload {
    connect: bool,
    link: String,
    routing_config: UIRoutingConfig,
}

#[derive(serde::Serialize)]
struct SubscriptionResponse {
    nodes: Vec<String>,
    userinfo: Option<String>,
}

#[derive(serde::Serialize)]
struct PlatformInfo {
    os: &'static str,
    apps_label: &'static str,
    apps_placeholder: &'static str,
    apps_append_suffix: bool,
    apps_supported: bool,
}

#[tauri::command]
fn get_platform_info() -> PlatformInfo {
    #[cfg(target_os = "windows")]
    {
        PlatformInfo {
            os: "windows",
            apps_label: "Applications",
            apps_placeholder: "e.g. chrome.exe, telegram.exe",
            apps_append_suffix: true,
            apps_supported: true,
        }
    }
    #[cfg(target_os = "linux")]
    {
        PlatformInfo {
            os: "linux",
            apps_label: "Applications",
            apps_placeholder: "e.g. chrome, telegram",
            apps_append_suffix: false,
            apps_supported: true,
        }
    }
    #[cfg(target_os = "macos")]
    {
        PlatformInfo {
            os: "macos",
            apps_label: "Applications",
            apps_placeholder: "e.g. Google Chrome, Telegram",
            apps_append_suffix: false,
            apps_supported: true,
        }
    }
    #[cfg(target_os = "android")]
    {
        PlatformInfo {
            os: "android",
            apps_label: "Apps (package name)",
            apps_placeholder: "e.g. com.android.chrome (coming soon)",
            apps_append_suffix: false,
            apps_supported: false,
        }
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "linux",
        target_os = "macos",
        target_os = "android"
    )))]
    {
        PlatformInfo {
            os: "unknown",
            apps_label: "Applications",
            apps_placeholder: "process name",
            apps_append_suffix: false,
            apps_supported: false,
        }
    }
}

#[tauri::command]
async fn toggle_vpn(
    payload: TogglePayload,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    tracing::info!(connect = payload.connect, mode = %payload.routing_config.mode, link_length = payload.link.len(), "toggle_vpn command called");

    let mode = match payload.routing_config.mode.as_str() {
        "global" => Mode::Global,
        "bypass" => Mode::Bypass,
        "proxy" => Mode::Proxy,
        _ => Mode::Bypass,
    };
    let dns_type_enum = match payload.routing_config.dns_type.as_deref() {
        Some("http") => DnsType::DnsHttp,
        Some("udp") => DnsType::DnsUdp,
        _ => DnsType::DnsHttps,
    };
    let allow_insecure = payload.routing_config.allow_insecure.unwrap_or(false);
    let use_ipv6 = payload.routing_config.use_ipv6.unwrap_or(false);

    let geoip_rules: HashMap<String, String> = payload
        .routing_config
        .geoip_rules
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_lowercase()))
        .filter(|(k, v)| {
            k.len() == 2
                && k.chars().all(|c| c.is_ascii_alphabetic())
                && matches!(v.as_str(), "proxy" | "direct" | "block")
        })
        .collect();

    let req = ToggleRequest {
        connect: payload.connect,
        profile_link: payload.link.clone(),
        routing: RoutingConfig {
            mode: mode.into(),
            domains: payload.routing_config.domains,
            apps: payload.routing_config.apps,
            enable_geoip: payload.routing_config.enable_geoip.unwrap_or(false),
            geoip_rules,
            dns_type: dns_type_enum.into(),
            allow_insecure,
            use_ipv6,
        },
    };

    let outcome = state.backend.toggle(req).await;

    match &outcome {
        Ok(o) if o.state == "Error" => {
            tracing::error!(error_message = %o.message, "Backend returned error state");
            let _ = app
                .notification()
                .builder()
                .title("PulsarVPN Error")
                .body(&o.message)
                .show();
            return Err(format!("Backend Error: {}", o.message));
        }
        Ok(o) if o.state == "Connected" => {
            let _ = app
                .notification()
                .builder()
                .title("PulsarVPN")
                .body("VPN Connected successfully")
                .show();
        }
        Ok(o) if o.state == "Disconnected" => {
            let _ = app
                .notification()
                .builder()
                .title("PulsarVPN")
                .body("VPN Disconnected")
                .show();
        }
        Err(e) => {
            tracing::error!(error = %e, "toggle_vpn failed");
            let _ = app
                .notification()
                .builder()
                .title("PulsarVPN Error")
                .body(format!("Connection Failed: {}", e))
                .show();
            return Err(e.clone());
        }
        _ => {}
    }
    Ok(outcome.unwrap().state)
}

#[tauri::command]
async fn get_vpn_state(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(match state.backend.state().await {
        VpnState::Connected => "Connected",
        VpnState::Connecting => "Connecting",
        VpnState::Disconnecting => "Disconnecting",
        VpnState::Disconnected => "Disconnected",
        VpnState::Error(_) => "Error",
    }
    .to_owned())
}

#[tauri::command]
async fn start_metrics_stream(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("start_metrics_stream command called");
    let mut rx = state
        .backend
        .subscribe_metrics()
        .await
        .ok_or_else(|| "metrics subscription already taken".to_string())?;
    tauri::async_runtime::spawn(async move {
        while let Some(ev) = rx.recv().await {
            let _ = app.emit(
                "vpn_metrics",
                serde_json::json!({"download": ev.download_speed, "upload": ev.upload_speed}),
            );
        }
    });
    Ok(())
}

#[tauri::command]
async fn check_ping(link: String) -> Result<u32, String> {
    let url = url::Url::parse(&link).map_err(|e| format!("Invalid URL: {}", e))?;
    let host = url.host_str().ok_or("No host".to_string())?;
    let port = url.port().unwrap_or(443);
    let start = std::time::Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(format!("{}:{}", host, port)),
    )
    .await
    {
        Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u32),
        Ok(Err(_)) => Err("Connection failed".into()),
        Err(_) => Err("Timeout".into()),
    }
}

#[tauri::command]
async fn fetch_subscription(url: String) -> Result<SubscriptionResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    let userinfo = resp
        .headers()
        .get("subscription-userinfo")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let decoded = STANDARD
        .decode(text.trim())
        .unwrap_or_else(|_| text.clone().into_bytes());
    let decoded_str = String::from_utf8_lossy(&decoded);
    let nodes: Vec<String> = decoded_str
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(SubscriptionResponse { nodes, userinfo })
}

#[tauri::command]
fn set_minimize_to_tray(enable: bool) {
    tracing::debug!(enable = enable, "set_minimize_to_tray");
}

#[tauri::command]
fn set_logs_streaming(enable: bool) {
    backend::set_logs_streaming_global(enable);
}

#[tauri::command]
fn log_message(level: String, module: String, message: String, request_id: Option<String>) {
    let _ = (request_id, module);
    match level.as_str() {
        "TRACE" => println!("[TRACE] {}", message),
        "DEBUG" => println!("[DEBUG] {}", message),
        "INFO" => println!("[INFO] {}", message),
        "WARN" => eprintln!("[WARN] {}", message),
        "ERROR" | "FATAL" => eprintln!("[{}] {}", level, message),
        _ => println!("[{}] {}", level, message),
    }
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn read_clipboard() -> Result<String, String> {
    let context = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }
        .map_err(|e| format!("Java VM unavailable: {e}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("cannot attach thread: {e}"))?;
    let app_context = unsafe { jni::objects::JObject::from_raw(context.context().cast()) };

    let class_loader = env
        .call_method(
            &app_context,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )
        .and_then(|value| value.l())
        .map_err(|e| format!("Android class loader unavailable: {e}"))?;
    let class_name = env
        .new_string("com.yann1n.pulsarvpn.MainActivity")
        .map_err(|e| format!("invalid class name: {e}"))?;
    let class_name_obj = jni::objects::JObject::from(class_name);
    let class = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[jni::objects::JValue::Object(&class_name_obj)],
        )
        .and_then(|value| value.l())
        .map(jni::objects::JClass::from)
        .map_err(|e| format!("Android class load failed: {e}"))?;

    let text_val = env
        .call_static_method(
            class,
            "getClipboardText",
            "(Landroid/content/Context;)Ljava/lang/String;",
            &[jni::objects::JValue::Object(&app_context)],
        )
        .map_err(|e| format!("Android getClipboardText call failed: {e}"))?;

    let text_obj = text_val
        .l()
        .map_err(|e| format!("Clipboard return value is not an object: {e}"))?;
    if text_obj.is_null() {
        return Ok(String::new());
    }

    let text: String = env
        .get_string(&jni::objects::JString::from(text_obj))
        .map_err(|e| format!("cannot get Rust string: {e}"))?
        .into();

    Ok(text)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
async fn read_clipboard() -> Result<String, String> {
    Err("Use browser clipboard API on desktop".into())
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Tauri setup started");

    #[cfg(desktop)]
    {
        use tauri::menu::{Menu, MenuItem};
        use tauri::tray::{MouseButton, TrayIconBuilder};

        let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
        let show_i = MenuItem::with_id(app, "show", "Open PulsarVPN", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
        let mut tray_builder = TrayIconBuilder::with_id("main_tray")
            .tooltip("Pulsar VPN")
            .menu(&menu)
            .on_menu_event(|app, event| match event.id().as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let tauri::tray::TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(w) = app.get_webview_window("main") {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }
            });
        if let Some(icon) = app.default_window_icon().cloned() {
            tray_builder = tray_builder.icon(icon);
        }
        if let Err(e) = tray_builder.build(app) {
            tracing::error!("Failed to build tray icon: {}", e);
        }
    }

    let logs_streaming_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    backend::set_logs_streaming_handle(logs_streaming_flag.clone());

    let backend = Backend::for_tauri(app, logs_streaming_flag.clone())
        .map_err(|e| -> Box<dyn std::error::Error> { format!("backend init: {e}").into() })?;
    app.manage(AppState { backend });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    if !ensure_windows_elevated() {
        return;
    }

    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }
    tracing::info!("Pulsar VPN UI starting...");

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ));
    }

    builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| setup_app(app))
        .invoke_handler(tauri::generate_handler![
            toggle_vpn,
            start_metrics_stream,
            check_ping,
            fetch_subscription,
            set_minimize_to_tray,
            set_logs_streaming,
            log_message,
            get_platform_info,
            get_vpn_state,
            read_clipboard,
        ])
        .on_window_event(|window, _event| {
            #[cfg(desktop)]
            {
                if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "windows")]
fn ensure_windows_elevated() -> bool {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let elevated = std::process::Command::new("whoami.exe")
        .args(["/groups", "/fo", "csv"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| {
            let groups = String::from_utf8_lossy(&output.stdout);
            groups.contains("S-1-16-12288") || groups.contains("S-1-16-16384")
        })
        .unwrap_or(false);
    if elevated {
        return true;
    }

    let Ok(executable) = std::env::current_exe() else {
        return false;
    };
    let path = executable.to_string_lossy().replace('\'', "''");
    let command = format!("Start-Process -FilePath '{path}' -Verb RunAs");
    let _ = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    false
}

#[cfg(target_os = "android")]
mod android_xray_runtime {
    use std::process::{Child, Command, Stdio};
    use std::sync::{Mutex, OnceLock};

    use jni::JNIEnv;
    use jni::objects::{JObject, JString};
    use jni::sys::{jint, jstring};

    static CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

    fn child() -> &'static Mutex<Option<Child>> {
        CHILD.get_or_init(|| Mutex::new(None))
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yann1n_pulsarvpn_PulsarVpnService_nativeStartXray(
        mut env: JNIEnv,
        _service: JObject,
        binary: JString,
        asset_directory: JString,
        config: JString,
        tun_fd: jint,
    ) -> jstring {
        let result = (|| -> Result<(), String> {
            stop_child();
            let binary: String = env.get_string(&binary).map_err(|e| e.to_string())?.into();
            let asset_directory: String = env
                .get_string(&asset_directory)
                .map_err(|e| e.to_string())?
                .into();
            let config: String = env.get_string(&config).map_err(|e| e.to_string())?.into();
            let config_path = std::path::Path::new(&asset_directory).join("config.json");
            std::fs::write(&config_path, config)
                .map_err(|e| format!("cannot write Xray config: {e}"))?;

            let flags = unsafe { libc::fcntl(tun_fd, libc::F_GETFD) };
            if flags < 0
                || unsafe { libc::fcntl(tun_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) } < 0
            {
                return Err(std::io::Error::last_os_error().to_string());
            }

            let spawned = Command::new(binary)
                .args(["run", "-config"])
                .arg(config_path)
                .current_dir(&asset_directory)
                .env("XRAY_TUN_FD", tun_fd.to_string())
                .env("XRAY_LOCATION_ASSET", &asset_directory)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("cannot start Xray: {e}"))?;
            *child()
                .lock()
                .map_err(|_| "Xray process lock poisoned".to_string())? = Some(spawned);
            Ok(())
        })();

        match result {
            Ok(()) => std::ptr::null_mut(),
            Err(error) => env
                .new_string(error)
                .map(JString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yann1n_pulsarvpn_PulsarVpnService_nativeStopXray(
        _env: JNIEnv,
        _service: JObject,
    ) {
        stop_child();
    }

    fn stop_child() {
        if let Ok(mut guard) = child().lock()
            && let Some(mut process) = guard.take()
        {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}
```
pulsar-ui\src-tauri\src\logger.rs
```
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

static LOG_GUARD: OnceLock<non_blocking::WorkerGuard> = OnceLock::new();

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Trace)
                .with_tag("PulsarVPN"),
        );
        return Ok(());
    }

    #[cfg(not(target_os = "android"))]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
        let log_dir = PathBuf::from(local_app_data).join("PulsarVPN").join("logs");
        std::fs::create_dir_all(&log_dir)?;

        let file_appender = rolling::daily(&log_dir, "pulsar-ui");
        let (non_blocking_appender, guard) = non_blocking(file_appender);
        let _ = LOG_GUARD.set(guard);

        let file_layer = fmt::Layer::default()
            .with_writer(non_blocking_appender)
            .with_ansi(false)
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .boxed();

        let console_layer = fmt::Layer::default()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .boxed();

        let subscriber = Registry::default()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(file_layer)
            .with(console_layer);

        if let Err(_) = subscriber.try_init() {
            eprintln!("Logger already initialized");
        }

        Ok(())
    }
}
```
pulsar-ui\src-tauri\src\main.rs
```
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pulsar_ui_lib::run();
}
```
pulsar-ui\src-tauri\tauri.conf.json
```
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Pulsar VPN",
  "version": "0.1.0",
  "identifier": "com.yann1n.pulsarvpn",
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Pulsar VPN",
        "width": 820,
        "height": 670,
        "minWidth": 480,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' ipc: http://ipc.localhost https://ipc.localhost; font-src 'self' data:"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [
      "binaries/pulsar-daemon.exe",
      "binaries/xray.exe",
      "binaries/wintun.dll",
      "binaries/geoip.dat",
      "binaries/geosite.dat"
    ],
    "android": {
      "minSdkVersion": 24,
      "autoIncrementVersionCode": true
    }
  }
}
```
pulsar-ui\vite.config.ts
```
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { templateCompilerOptions } from '@tresjs/core' // 1. РРјРїРѕСЂС‚РёСЂСѓРµРј РЅР°СЃС‚СЂРѕР№РєРё TresJS

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
    plugins: [
        vue({
            // 2. Р’РЅРµРґСЂСЏРµРј РЅР°СЃС‚СЂРѕР№РєРё РєРѕРјРїРёР»СЏС‚РѕСЂР°, С‡С‚РѕР±С‹ Vue РїРѕРЅРёРјР°Р» 3D-С‚РµРіРё (<TresMesh> Рё С‚.Рґ.)
            ...templateCompilerOptions
        })
    ],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                protocol: "ws",
                host,
                port: 1421,
            }
            : undefined,
        watch: {
            // 3. tell Vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
}));```
test_config.rs
```
// Quick test: generate sing-box config from a hy2 link
use pulsar_shared::vpn::{DnsType, RoutingConfig};
use pulsar_daemon::config_gen::generate_singbox_config;

fn main() {
    let link = "hy2://d110dc919800a1b4@2.27.52.157:443?mport&security=tls&sni=web.max.ru&allowInsecure=true&alpn&obfs=salamander&obfs-password=dd7c80b47f00eb4e#User1";

    let routing = RoutingConfig {
        mode: pulsar_shared::vpn::routing_config::Mode::Global,
        domains: vec![],
        apps: vec![],
        dns_type: DnsType::DnsHttps,
        use_ipv6: false,
        allow_insecure: true,
        enable_geoip: false,
        geoip_rules: Default::default(),
    };

    match generate_singbox_config(link, &routing) {
        Ok(config) => {
            println!("=== GENERATED CONFIG ===");
            println!("{}", config);
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
```
xtask\Cargo.toml
```
[package]
name = "xtask"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
publish = false
description = "Pulsar VPN build orchestrator. Provides `cargo xtask <subcommand>` to build, dev, clean, lint, and inspect the workspace."

[[bin]]
name = "xtask"
path = "src/main.rs"

[dependencies]
# intentionally zero external deps вЂ” xtask is a thin wrapper over std::process
```
xtask\src\main.rs
```
//! `xtask` вЂ” РµРґРёРЅР°СЏ С‚РѕС‡РєР° РІС…РѕРґР° РґР»СЏ СЃР±РѕСЂРєРё / СЂР°Р·СЂР°Р±РѕС‚РєРё / РїСЂРѕРІРµСЂРєРё Pulsar VPN.
//!
//! Р Р°РЅСЊС€Рµ РїРѕР»СЊР·РѕРІР°С‚РµР»СЋ РЅСѓР¶РЅРѕ Р±С‹Р»Рѕ РІСЂСѓС‡РЅСѓСЋ:
//!   1. `cargo build --release -p pulsar-daemon`
//!   2. `Copy-Item target\release\pulsar-daemon.exe pulsar-ui\src-tauri\binaries\`
//!   3. `cd pulsar-ui && bun install && bun run build`
//!   4. `cd pulsar-ui && bun run tauri build`
//!
//! РўРµРїРµСЂСЊ РІСЃС‘ СЌС‚Рѕ РѕРґРЅР° РєРѕРјР°РЅРґР°: `cargo xtask build`.
//!
//! Р”РѕСЃС‚СѓРїРЅС‹Рµ РїРѕРґРєРѕРјР°РЅРґС‹: `build`, `dev`, `clean`, `lint`, `info`, `help`.
//! РЎРј. `print_help()` РЅРёР¶Рµ.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Green,
    Yellow,
    Cyan,
    Gray,
    Reset,
}

impl Color {
    fn code(self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Cyan => "\x1b[36m",
            Color::Gray => "\x1b[90m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Stdout/stderr Сѓ Tauri cargo вЂ” РјРЅРѕРіРѕСЃР»РѕРІРЅС‹Р№. РџРµС‡Р°С‚Р°РµРј С€Р°РіРё РЅР° stderr,
/// С‡С‚РѕР±С‹ РїРѕР»СЊР·РѕРІР°С‚РµР»СЊ РІРёРґРµР» РїСЂРѕРіСЂРµСЃСЃ Рё РїСЂРё С‚СЂСѓР±Р°С…, Рё РїСЂРё Р»РѕРіРёСЂРѕРІР°РЅРёРё.
fn paint(color: Color, s: &str) -> String {
    // Windows 10+ РІРєР»СЋС‡Р°РµС‚ РІРёСЂС‚СѓР°Р»СЊРЅС‹Р№ С‚РµСЂРјРёРЅР°Р» РїРѕ СѓРјРѕР»С‡Р°РЅРёСЋ, РЅРѕ РІСЃС‘ Р¶Рµ
    // Р°РєРєСѓСЂР°С‚РЅРѕ РѕС‚РєР»СЋС‡Р°РµРј С†РІРµС‚Р°, РµСЃР»Рё NO_COLOR РІС‹СЃС‚Р°РІР»РµРЅ РёР»Рё stderr РЅРµ TTY.
    if env::var_os("NO_COLOR").is_some() || !atty_stderr() {
        return s.to_string();
    }
    format!("{}{}{}", color.code(), s, Color::Reset.code())
}

fn atty_stderr() -> bool {
    // РџСЂРѕСЃС‚РµР№С€Р°СЏ РїСЂРѕРІРµСЂРєР° вЂ” РЅР° Windows isatty С‡РµСЂРµР· std РЅРµРґРѕСЃС‚СѓРїРµРЅ,
    // РїРѕСЌС‚РѕРјСѓ СЌРІСЂРёСЃС‚РёРєР°: РµСЃР»Рё РІ stderr РЅРµ РїР°Р№Рї, СЃС‡РёС‚Р°РµРј С‡С‚Рѕ TTY.
    // (Tauri cargo РЅР° Windows 10+ РЅРѕСЂРјР°Р»СЊРЅРѕ РѕР±СЂР°Р±Р°С‚С‹РІР°РµС‚ ANSI Рё Р±РµР· СЌС‚РѕРіРѕ.)
    true
}

fn say(color: Color, prefix: &str, msg: &str) {
    eprintln!("{}{}", paint(color, prefix), msg);
}

fn info(msg: &str) {
    say(Color::Cyan, "==> ", msg);
}
fn ok(msg: &str) {
    say(Color::Green, "  вњ“ ", msg);
}
fn warn(msg: &str) {
    say(Color::Yellow, "  ! ", msg);
}
fn err(msg: &str) {
    say(Color::Red, "  вњ— ", msg);
}
fn dim(msg: &str) {
    say(Color::Gray, "    ", msg);
}

#[derive(Debug, Clone)]
struct Config {
    workspace_root: PathBuf,
    target_dir: PathBuf,
    release: bool,
    json: bool,
}

impl Config {
    fn detect() -> Self {
        // CARGO_MANIFEST_DIR СѓРєР°Р·С‹РІР°РµС‚ РЅР° xtask/ (Р·Р°РїСѓСЃРєР°РµРјС‹Р№ crate).
        // Р РѕРґРёС‚РµР»СЊ вЂ” РєРѕСЂРµРЅСЊ РІРѕСЂРєСЃРїРµР№СЃР°.
        let workspace_root = env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|p| PathBuf::from(p).parent().map(|p| p.to_path_buf()))
            .or_else(|| env::current_dir().ok())
            .expect("xtask: cannot determine workspace root");
        let target_dir = workspace_root.join("target");
        Self {
            workspace_root,
            target_dir,
            release: false,
            json: false,
        }
    }

    fn exe(name: &str) -> String {
        if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        }
    }

    fn daemon_binary() -> String {
        Self::exe("pulsar-daemon")
    }
    fn xray_binary() -> String {
        Self::exe("xray")
    }

    fn binaries_dir(&self) -> PathBuf {
        self.workspace_root
            .join("pulsar-ui")
            .join("src-tauri")
            .join("binaries")
    }

    fn daemon_built_path(&self) -> PathBuf {
        let profile = if self.release { "release" } else { "debug" };
        self.target_dir.join(profile).join(Self::daemon_binary())
    }

    fn daemon_dest_path(&self) -> PathBuf {
        self.binaries_dir().join(Self::daemon_binary())
    }

    fn xray_expected_path(&self) -> PathBuf {
        self.binaries_dir().join(Self::xray_binary())
    }

    fn ui_dir(&self) -> PathBuf {
        self.workspace_root.join("pulsar-ui")
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let task = args.get(1).map(String::as_str).unwrap_or("help");
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    let mut cfg = Config::detect();
    cfg.json = rest.iter().any(|a| a == "--json");

    match task {
        "build" => {
            cfg.release = rest.iter().any(|a| a == "--release" || a == "-r");
            run_build(cfg)
        }
        "build-android" => {
            cfg.release = rest.iter().any(|a| a == "--release" || a == "-r");
            run_build_android(cfg)
        }
        "build-all" => {
            cfg.release = rest.iter().any(|a| a == "--release" || a == "-r");
            run_build_all(cfg)
        }
        "dev" => run_dev(cfg),
        "clean" => run_clean(cfg, &rest),
        "lint" => run_lint(cfg),
        "info" => run_info(cfg),
        "check" => {
            // РђР»РёР°СЃ РґР»СЏ `lint`, СѓРґРѕР±РЅС‹Р№ РІ CI.
            cfg.json = rest.iter().any(|a| a == "--json");
            run_lint(cfg)
        }
        "help" | "-h" | "--help" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            err(&format!("РЅРµРёР·РІРµСЃС‚РЅР°СЏ РїРѕРґРєРѕРјР°РЅРґР°: `{}`", other));
            eprintln!();
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    let help = "\
xtask вЂ” РѕСЂРєРµСЃС‚СЂР°С‚РѕСЂ СЃР±РѕСЂРєРё Pulsar VPN

РРЎРџРћР›Р¬Р—РћР’РђРќРР•:
    cargo xtask <РљРћРњРђРќР”Рђ> [РћРџР¦РР]

РљРћРњРђРќР”Р«:
    build [--release]       РџРѕР»РЅР°СЏ prod-СЃР±РѕСЂРєР° РґР»СЏ С‚РµРєСѓС‰РµР№ РћРЎ (Windows installer / etc.)
    build-android [--release]  РЎР±РѕСЂРєР° Android APK/AAB (РЅСѓР¶РЅС‹ SDK+NDK+JDK+rustup targets)
    build-all [--release]   build + build-android РїРѕРґСЂСЏРґ (РµСЃР»Рё РѕРєСЂСѓР¶РµРЅРёРµ РїРѕРґРґРµСЂР¶РёРІР°РµС‚ РѕР±Р°)
    dev                     Р РµР¶РёРј СЂР°Р·СЂР°Р±РѕС‚РєРё: daemon (debug) в†’ copy в†’ tauri dev
    clean [--deep]          РћС‡РёСЃС‚РёС‚СЊ Р°СЂС‚РµС„Р°РєС‚С‹ СЃР±РѕСЂРєРё (--deep С‚Р°РєР¶Рµ СѓРґР°Р»СЏРµС‚ node_modules)
    lint                    cargo check + clippy + vue-tsc
    check                   РђР»РёР°СЃ РґР»СЏ lint (СѓРґРѕР±РЅРѕ РІ CI)
    info                    РџСѓС‚Рё, РІРµСЂСЃРёРё РёРЅСЃС‚СЂСѓРјРµРЅС‚РѕРІ, СЃС‚Р°С‚СѓСЃ Р±РёРЅР°СЂРЅРёРєРѕРІ
    help                    РџРѕРєР°Р·Р°С‚СЊ СЌС‚Рѕ СЃРѕРѕР±С‰РµРЅРёРµ

РџР РРњР•Р Р«:
    cargo xtask build
    cargo xtask build --release
    cargo xtask build-android
    cargo xtask build-all
    cargo xtask dev
    cargo xtask clean --deep
    cargo xtask lint
    cargo xtask info
";
    print!("{}", help);
}

// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//  BUILD
// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

fn run_build(cfg: Config) -> ExitCode {
    let started = Instant::now();
    info("Pulsar VPN вЂ” production build");
    dim(&format!(
        "  profile:    {}",
        if cfg.release { "release" } else { "debug" }
    ));
    dim(&format!("  workspace:  {}", cfg.workspace_root.display()));
    eprintln!();

    let steps: [(&str, &str, fn(&Config) -> bool); 5] = [
        ("1/5", "РЎР±РѕСЂРєР° РґРµРјРѕРЅР° (pulsar-daemon)", build_daemon),
        ("2/5", "РљРѕРїРёСЂРѕРІР°РЅРёРµ РґРµРјРѕРЅР° РІ UI СЂРµСЃСѓСЂСЃС‹", copy_daemon),
        ("3/5", "РЈСЃС‚Р°РЅРѕРІРєР° Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№ UI", install_ui_deps),
        ("4/5", "РЎР±РѕСЂРєР° С„СЂРѕРЅС‚РµРЅРґР° (vue-tsc + vite)", build_frontend),
        ("5/5", "РЎР±РѕСЂРєР° Tauri РїСЂРёР»РѕР¶РµРЅРёСЏ Рё РёРЅСЃС‚Р°Р»Р»СЏС‚РѕСЂР°", build_tauri),
    ];

    for (n, name, f) in steps {
        if !run_step(n, name, || f(&cfg)) {
            err(&format!("СЃР±РѕСЂРєР° РїСЂРµСЂРІР°РЅР° РЅР° С€Р°РіРµ {}", n));
            return ExitCode::FAILURE;
        }
    }

    // РќРµ РєСЂРёС‚РёС‡РЅРѕ РґР»СЏ СЃР±РѕСЂРєРё, РЅРѕ Р±РµР· Xray РїСЂРёР»РѕР¶РµРЅРёРµ РЅРµ СЃРјРѕР¶РµС‚ РїРѕРґРЅСЏС‚СЊ VPN.
    check_xray(&cfg);

    let secs = started.elapsed().as_secs();
    eprintln!();
    ok(&format!("Р“РѕС‚РѕРІРѕ Р·Р° {}СЃ", secs));
    print_artifacts(&cfg);
    ExitCode::SUCCESS
}

fn run_dev(cfg: Config) -> ExitCode {
    info("Pulsar VPN вЂ” dev mode");
    dim("  Р‘СѓРґРµС‚ СЃРѕР±СЂР°РЅ debug-РґРµРјРѕРЅ Рё Р·Р°РїСѓС‰РµРЅ tauri dev СЃ hot-reload.");
    eprintln!();

    let steps: [(&str, &str, fn(&Config) -> bool); 3] = [
        ("1/3", "РЎР±РѕСЂРєР° РґРµРјРѕРЅР° (debug)", build_daemon),
        ("2/3", "РљРѕРїРёСЂРѕРІР°РЅРёРµ РґРµРјРѕРЅР° РІ UI", copy_daemon),
        ("3/3", "Р—Р°РїСѓСЃРє tauri dev (Ctrl+C РґР»СЏ РІС‹С…РѕРґР°)", tauri_dev),
    ];

    for (n, name, f) in steps {
        if !run_step(n, name, || f(&cfg)) {
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//  ANDROID BUILD
// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

/// РџСЂРѕРІРµСЂСЏРµС‚, С‡С‚Рѕ РЅР° РјР°С€РёРЅРµ РµСЃС‚СЊ РІСЃС‘ РґР»СЏ СЃР±РѕСЂРєРё Android:
/// - cargo-tauri CLI СЃ РїРѕРґРґРµСЂР¶РєРѕР№ `android` (>= 2.0)
/// - ANDROID_HOME / SDK
/// - NDK
/// - JDK 17+
/// - Rust-С‚Р°СЂРіРµС‚С‹ `aarch64-linux-android`, `armv7-linux-androideabi`,
///   `x86_64-linux-android`, `i686-linux-android` С…РѕС‚СЏ Р±С‹ РѕРґРёРЅ РёР· РЅРёС….
fn check_android_prereqs(cfg: &Config) -> bool {
    let mut all_ok = true;

    // tauri CLI
    match run_capture("cargo", &["tauri", "--version"], &cfg.workspace_root) {
        Ok(v) => {
            let v_trim = v.trim().to_lowercase();
            if !v_trim.contains("tauri") {
                err(&format!("`cargo tauri` РЅРµРґРѕСЃС‚СѓРїРµРЅ: {}", v));
                all_ok = false;
            } else {
                ok(&format!("tauri CLI: {}", v.trim()));
            }
        }
        Err(_) => {
            err("`cargo tauri` РЅРµ РЅР°Р№РґРµРЅ");
            dim("    СѓСЃС‚Р°РЅРѕРІРёС‚Рµ: cargo install tauri-cli --version \"^2.0\"");
            all_ok = false;
        }
    }

    // ANDROID_HOME / SDK
    let sdk = env::var("ANDROID_HOME")
        .or_else(|_| env::var("ANDROID_SDK_ROOT"))
        .ok();
    match sdk.as_deref() {
        Some(s) if !s.is_empty() && Path::new(s).exists() => {
            ok(&format!("Android SDK: {}", s));
        }
        _ => {
            err("ANDROID_HOME РЅРµ Р·Р°РґР°РЅ РёР»Рё РїСѓС‚СЊ РЅРµ СЃСѓС‰РµСЃС‚РІСѓРµС‚");
            dim("    СѓСЃС‚Р°РЅРѕРІРёС‚Рµ Android Studio Рё РІС‹РїРѕР»РЅРёС‚Рµ: export ANDROID_HOME=$HOME/Android/Sdk");
            all_ok = false;
        }
    }

    // NDK вЂ” Tauri СЃР°Рј РёС‰РµС‚ РІ $ANDROID_HOME/ndk/<version>/
    if let Some(sdk) = &sdk {
        let ndk_dir = Path::new(sdk).join("ndk");
        if ndk_dir.exists() {
            if let Ok(read) = std::fs::read_dir(&ndk_dir) {
                let versions: Vec<String> = read
                    .flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        if p.is_dir() {
                            p.file_name().and_then(|n| n.to_str()).map(String::from)
                        } else {
                            None
                        }
                    })
                    .collect();
                if versions.is_empty() {
                    err("NDK РЅРµ СѓСЃС‚Р°РЅРѕРІР»РµРЅ (РїР°РїРєР° ndk/ РїСѓСЃС‚Р°)");
                    dim(
                        "    РѕС‚РєСЂРѕР№С‚Рµ Android Studio в†’ SDK Manager в†’ SDK Tools в†’ NDK (Side by side)",
                    );
                    all_ok = false;
                } else {
                    ok(&format!(
                        "NDK: {} (РґРѕСЃС‚СѓРїРЅРѕ: {})",
                        ndk_dir.display(),
                        versions.join(", ")
                    ));
                }
            }
        } else {
            err(&format!("NDK РґРёСЂРµРєС‚РѕСЂРёСЏ РЅРµ РЅР°Р№РґРµРЅР°: {}", ndk_dir.display()));
            all_ok = false;
        }
    }

    // JDK
    if let Ok(v) = run_capture("java", &["-version"], &cfg.workspace_root) {
        ok(&format!("JDK: {}", v.lines().next().unwrap_or("?").trim()));
    } else {
        err("JDK РЅРµ РЅР°Р№РґРµРЅ (РЅСѓР¶РµРЅ JDK 17+)");
        dim("    СѓСЃС‚Р°РЅРѕРІРёС‚Рµ JDK 17+ Рё СѓР±РµРґРёС‚РµСЃСЊ С‡С‚Рѕ `java -version` СЂР°Р±РѕС‚Р°РµС‚");
        all_ok = false;
    }

    // Rust-С‚Р°СЂРіРµС‚С‹ РґР»СЏ Android
    if let Ok(v) = run_capture(
        "rustup",
        &["target", "list", "--installed"],
        &cfg.workspace_root,
    ) {
        let have: Vec<&str> = v.lines().collect();
        let needed = [
            "aarch64-linux-android",
            "armv7-linux-androideabi",
            "x86_64-linux-android",
            "i686-linux-android",
        ];
        let missing: Vec<&&str> = needed
            .iter()
            .filter(|t| !have.iter().any(|h| h.trim() == **t))
            .collect();
        if missing.len() == needed.len() {
            err("РЅРё РѕРґРЅРѕРіРѕ Android Rust target РЅРµ СѓСЃС‚Р°РЅРѕРІР»РµРЅРѕ");
            dim(&format!(
                "    СѓСЃС‚Р°РЅРѕРІРёС‚Рµ С…РѕС‚СЏ Р±С‹: rustup target add {}",
                needed[0]
            ));
            all_ok = false;
        } else {
            let installed: Vec<&&str> = needed
                .iter()
                .filter(|t| have.iter().any(|h| h.trim() == **t))
                .collect();
            ok(&format!(
                "Android Rust targets: {}",
                installed.iter().map(|s| **s).collect::<Vec<_>>().join(", ")
            ));
        }
    } else {
        err("rustup РЅРµРґРѕСЃС‚СѓРїРµРЅ, РЅРµРІРѕР·РјРѕР¶РЅРѕ РїСЂРѕРІРµСЂРёС‚СЊ Android targets");
        all_ok = false;
    }

    all_ok
}

/// РџСЂРѕРІРµСЂСЏРµС‚, С‡С‚Рѕ `cargo tauri android init` СѓР¶Рµ Р·Р°РїСѓСЃРєР°Р»СЃСЏ (СЃРѕР·РґР°Р»
/// `gen/android/`). Р•СЃР»Рё РЅРµС‚ вЂ” РїРѕРґСЃРєР°Р·С‹РІР°РµС‚ Р·Р°РїСѓСЃС‚РёС‚СЊ.
fn check_tauri_android_init(cfg: &Config) -> bool {
    // Tauri 2 РєР»Р°РґС‘С‚ gen/ РІ src-tauri/ (СЂСЏРґРѕРј СЃ Cargo.toml, Р° РЅРµ РІ pulsar-ui/).
    let gen_dir = cfg.ui_dir().join("src-tauri").join("gen").join("android");
    if gen_dir.exists() && gen_dir.is_dir() {
        ok(&format!(
            "gen/android СѓР¶Рµ РёРЅРёС†РёР°Р»РёР·РёСЂРѕРІР°РЅ: {}",
            gen_dir.display()
        ));
        true
    } else {
        err("gen/android РЅРµ РЅР°Р№РґРµРЅ");
        dim("    РІС‹РїРѕР»РЅРёС‚Рµ РѕРґРёРЅ СЂР°Р·: cd pulsar-ui && cargo tauri android init");
        dim("    СЌС‚Рѕ СЃРѕР·РґР°СЃС‚ Gradle-РїСЂРѕРµРєС‚; РєРѕРјРјРёС‚РёС‚СЊ РµРіРѕ РЅРµ РЅСѓР¶РЅРѕ вЂ” РѕРЅ СЂРµРіРµРЅРµСЂРёСЂСѓРµС‚СЃСЏ.");
        false
    }
}

fn run_build_android(cfg: Config) -> ExitCode {
    let started = Instant::now();
    info("Pulsar VPN вЂ” Android build");
    dim(&format!(
        "  profile:   {}",
        if cfg.release { "release" } else { "debug" }
    ));
    dim(&format!("  workspace: {}", cfg.workspace_root.display()));
    eprintln!();

    info("[1/3] РџСЂРѕРІРµСЂРєР° pre-requisites (SDK / NDK / JDK / rust targets)");
    if !check_android_prereqs(&cfg) {
        err("РЅРµ С…РІР°С‚Р°РµС‚ РёРЅСЃС‚СЂСѓРјРµРЅС‚РѕРІ РґР»СЏ Android-СЃР±РѕСЂРєРё");
        return ExitCode::FAILURE;
    }
    eprintln!();

    info("[2/3] РџСЂРѕРІРµСЂРєР° `cargo tauri android init`");
    if !check_tauri_android_init(&cfg) {
        return ExitCode::FAILURE;
    }
    eprintln!();

    info("[3/3] РЎР±РѕСЂРєР° APK");
    // `cargo tauri android build` вЂ” release РїРѕ СѓРјРѕР»С‡Р°РЅРёСЋ, --debug РґР»СЏ РґРµР±Р°РіР°.
    // Р­С‚Рѕ РѕС‚Р»РёС‡Р°РµС‚СЃСЏ РѕС‚ `cargo tauri build` РґР»СЏ РґРµСЃРєС‚РѕРїР°, РіРґРµ РїСЂРѕС„РёР»СЊ Р·Р°РґР°С‘С‚СЃСЏ
    // С‡РµСЂРµР· `[profile.release]` РІ Cargo.toml.
    let mut args: Vec<&str> = vec!["tauri", "android", "build"];
    if !cfg.release {
        args.push("--debug");
    }
    if !run("cargo", &args, &cfg.ui_dir()) {
        err("tauri android build Р·Р°РІРµСЂС€РёР»СЃСЏ СЃ РѕС€РёР±РєРѕР№");
        return ExitCode::FAILURE;
    }

    let secs = started.elapsed().as_secs();
    eprintln!();
    ok(&format!("Р“РѕС‚РѕРІРѕ Р·Р° {}СЃ", secs));
    print_android_artifacts(&cfg);
    ExitCode::SUCCESS
}

fn print_android_artifacts(cfg: &Config) {
    eprintln!();
    info("РђСЂС‚РµС„Р°РєС‚С‹:");
    let mut found = false;
    // Tauri 2 Android РєР»Р°РґС‘С‚ Р°СЂС‚РµС„Р°РєС‚С‹ РІ outputs/{apk,bundle}/, Рё РІР»РѕР¶РµРЅРЅРѕСЃС‚СЊ
    // Р·Р°РІРёСЃРёС‚ РѕС‚ РІР°СЂРёР°РЅС‚Р°: universal (apk/universal/{debug,release}/),
    // per-ABI (apk/{aarch64,armv7,i686,x86_64}/{debug,release}/) Рё С‚. Рї.
    // Р РµРєСѓСЂСЃРёРІРЅРѕ РїСЂРѕС…РѕРґРёРј outputs/ РІ РїРѕРёСЃРєР°С… .apk/.aab.
    let outputs_root = cfg
        .ui_dir()
        .join("src-tauri")
        .join("gen")
        .join("android")
        .join("app")
        .join("build")
        .join("outputs");
    if outputs_root.exists() {
        if let Ok(read) = std::fs::read_dir(&outputs_root) {
            for top in read.flatten() {
                let top_path = top.path();
                if !top_path.is_dir() {
                    continue;
                }
                if let Ok(walk) = walkdir(&top_path, 3) {
                    for path in walk {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if matches!(ext, "apk" | "aab") {
                                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                dim(&format!(
                                    "  {}  ({:.1} MB)",
                                    path.display(),
                                    size as f64 / 1_048_576.0
                                ));
                                found = true;
                            }
                        }
                    }
                }
            }
        }
    }
    if !found {
        dim("  (APK/AAB РЅРµ РЅР°Р№РґРµРЅС‹ вЂ” РїСЂРѕРІРµСЂСЊС‚Рµ Р»РѕРі tauri android РІС‹С€Рµ)");
    }
}

/// Р РµРєСѓСЂСЃРёРІРЅС‹Р№ РѕР±С…РѕРґ РґРёСЂРµРєС‚РѕСЂРёРё РґРѕ Р·Р°РґР°РЅРЅРѕР№ РіР»СѓР±РёРЅС‹ (0 = С‚РѕР»СЊРєРѕ С‚РµРєСѓС‰Р°СЏ).
/// Р’РѕР·РІСЂР°С‰Р°РµС‚ Vec<PathBuf>. Р‘РµР· Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№.
fn walkdir(root: &Path, max_depth: usize) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    fn recurse(p: &Path, depth: usize, max: usize, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if depth > max {
            return Ok(());
        }
        for entry in std::fs::read_dir(p)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                recurse(&path, depth + 1, max, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    recurse(root, 0, max_depth, &mut out)?;
    Ok(out)
}

fn run_build_all(cfg: Config) -> ExitCode {
    info("Pulsar VPN вЂ” build-all (current platform + Android)");
    eprintln!();

    // РЁР°Рі 1: С‚РµРєСѓС‰Р°СЏ РїР»Р°С‚С„РѕСЂРјР° (Windows installer Рё С‚. Рї.)
    info("=== Р­С‚Р°Рї 1/2: РґРµСЃРєС‚РѕРї-СЃР±РѕСЂРєР° ===");
    if run_build(cfg.clone()) != ExitCode::SUCCESS {
        err("РґРµСЃРєС‚РѕРї-СЃР±РѕСЂРєР° РїСЂРѕРІР°Р»РёР»Р°СЃСЊ");
        return ExitCode::FAILURE;
    }
    eprintln!();

    // РЁР°Рі 2: Android (РµСЃР»Рё РѕРєСЂСѓР¶РµРЅРёРµ РїРѕР·РІРѕР»СЏРµС‚).
    info("=== Р­С‚Р°Рї 2/2: Android-СЃР±РѕСЂРєР° ===");
    if run_build_android(cfg) != ExitCode::SUCCESS {
        warn("Android-СЃР±РѕСЂРєР° РїСЂРѕРїСѓС‰РµРЅР° РёР»Рё РїСЂРѕРІР°Р»РёР»Р°СЃСЊ (СЃРј. РѕС€РёР±РєРё РІС‹С€Рµ).");
        warn("Р­С‚Рѕ РЅРѕСЂРјР°Р»СЊРЅРѕ, РµСЃР»Рё РЅР° РјР°С€РёРЅРµ РЅРµС‚ Android SDK/NDK.");
        warn("Р”РµСЃРєС‚РѕРї-Р°СЂС‚РµС„Р°РєС‚С‹ РїСЂРё СЌС‚РѕРј РіРѕС‚РѕРІС‹.");
        return ExitCode::SUCCESS; // РґРµСЃРєС‚РѕРї СѓР¶Рµ РіРѕС‚РѕРІ вЂ” РѕР±С‰РёР№ СѓСЃРїРµС…
    }

    eprintln!();
    ok("Р’СЃРµ РїР»Р°С‚С„РѕСЂРјС‹ СЃРѕР±СЂР°РЅС‹");
    ExitCode::SUCCESS
}

fn run_step(label: &str, name: &str, f: impl FnOnce() -> bool) -> bool {
    let started = Instant::now();
    info(&format!("[{}] {}", label, name));
    let ok = f();
    if !ok {
        return false;
    }
    let dt = started.elapsed();
    if dt.as_millis() > 100 {
        dim(&format!("    в””в”Ђ Р·Р°РЅСЏР»Рѕ {:.1?}", dt));
    }
    eprintln!();
    true
}

fn build_daemon(cfg: &Config) -> bool {
    let args: &[&str] = if cfg.release {
        &["build", "--release", "-p", "pulsar-daemon"]
    } else {
        &["build", "-p", "pulsar-daemon"]
    };
    run("cargo", args, &cfg.workspace_root)
}

fn copy_daemon(cfg: &Config) -> bool {
    let src = cfg.daemon_built_path();
    let dst = cfg.daemon_dest_path();
    if !src.exists() {
        err(&format!("РґРµРјРѕРЅ РЅРµ РЅР°Р№РґРµРЅ РїРѕСЃР»Рµ СЃР±РѕСЂРєРё: {}", src.display()));
        return false;
    }
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                err(&format!("РЅРµ СѓРґР°Р»РѕСЃСЊ СЃРѕР·РґР°С‚СЊ {}: {}", parent.display(), e));
                return false;
            }
        }
    }
    if let Err(e) = std::fs::copy(&src, &dst) {
        err(&format!(
            "РєРѕРїРёСЂРѕРІР°РЅРёРµ {} в†’ {}: {}",
            src.display(),
            dst.display(),
            e
        ));
        return false;
    }
    dim(&format!("    {}", dst.display()));
    ok("РґРµРјРѕРЅ СЃРєРѕРїРёСЂРѕРІР°РЅ");
    true
}

fn check_xray(cfg: &Config) {
    let p = cfg.xray_expected_path();
    if !p.exists() {
        warn(&format!("Xray not found: {}", p.display()));
        warn("  РїСЂРёР»РѕР¶РµРЅРёРµ Р·Р°РїСѓСЃС‚РёС‚СЃСЏ, РЅРѕ РЅРµ СЃРјРѕР¶РµС‚ РїРѕРґРЅРёРјР°С‚СЊ VPN-С‚СѓРЅРЅРµР»СЊ.");
        warn("  Put xray.exe next to pulsar-daemon.exe and rebuild.");
    } else {
        ok("Xray found");
    }
}

fn install_ui_deps(cfg: &Config) -> bool {
    run("bun", &["install", "--frozen-lockfile"], &cfg.ui_dir())
}

fn build_frontend(cfg: &Config) -> bool {
    run("bun", &["run", "build"], &cfg.ui_dir())
}

fn build_tauri(cfg: &Config) -> bool {
    run("bun", &["run", "tauri", "build"], &cfg.ui_dir())
}

fn tauri_dev(cfg: &Config) -> bool {
    // РќРµ РёСЃРїРѕР»СЊР·СѓРµРј piped stdout вЂ” РїРѕР»СЊР·РѕРІР°С‚РµР»СЊ РґРѕР»Р¶РµРЅ РІРёРґРµС‚СЊ Р»РѕРі tauri РЅР°РїСЂСЏРјСѓСЋ.
    run("bun", &["run", "tauri", "dev"], &cfg.ui_dir())
}

fn print_artifacts(cfg: &Config) {
    eprintln!();
    info("РђСЂС‚РµС„Р°РєС‚С‹:");
    // Tauri РєР»Р°РґС‘С‚ bundle РІ pulsar-ui/src-tauri/target/release/bundle (РѕС‚РґРµР»СЊРЅС‹Р№ target
    // РёР·-Р·Р° build.rs РІ src-tauri) Р»РёР±Рѕ РІ target/release/bundle (РµСЃР»Рё Tauri РїРµСЂРµРёСЃРїРѕР»СЊР·СѓРµС‚
    // workspace target). РџСЂРѕРІРµСЂРёРј РѕР±Р° РІР°СЂРёР°РЅС‚Р°.
    let ui_target_bundle = cfg
        .ui_dir()
        .join("src-tauri")
        .join("target")
        .join("release")
        .join("bundle");
    let ws_target_bundle = cfg.target_dir.join("release").join("bundle");
    let bundles = [ui_target_bundle, ws_target_bundle];

    let mut found = false;
    for root in &bundles {
        if !root.exists() {
            continue;
        }
        if let Ok(read) = std::fs::read_dir(root) {
            for sub in read.flatten() {
                let sub_path = sub.path();
                if sub_path.is_dir() {
                    if let Ok(files) = std::fs::read_dir(&sub_path) {
                        for f in files.flatten() {
                            let path = f.path();
                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                if matches!(ext, "msi" | "exe" | "AppImage" | "deb" | "dmg") {
                                    let size = f.metadata().map(|m| m.len()).unwrap_or(0);
                                    dim(&format!(
                                        "  {}  ({:.1} MB)",
                                        path.display(),
                                        size as f64 / 1_048_576.0
                                    ));
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !found {
        dim("  (РёРЅСЃС‚Р°Р»Р»СЏС‚РѕСЂС‹ РЅРµ РЅР°Р№РґРµРЅС‹ вЂ” РїСЂРѕРІРµСЂСЊС‚Рµ Р»РѕРі Tauri РІС‹С€Рµ)");
    }
}

// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//  CLEAN
// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

fn run_clean(cfg: Config, args: &[String]) -> ExitCode {
    let deep = args.iter().any(|a| a == "--deep");
    info("РћС‡РёСЃС‚РєР° Р°СЂС‚РµС„Р°РєС‚РѕРІ СЃР±РѕСЂРєРё");
    eprintln!();

    let mut ok_all = true;

    info("cargo clean (workspace target)");
    ok_all &= run_quiet("cargo", &["clean"], &cfg.workspace_root);

    // Tauri src-tauri РёРјРµРµС‚ СЃРІРѕР№ build.rs, РєРѕС‚РѕСЂС‹Р№ РґРµСЂР¶РёС‚ СЃРІРѕР№ target СЂСЏРґРѕРј.
    let tauri_target = cfg.ui_dir().join("src-tauri").join("target");
    if tauri_target.exists() {
        info(&format!("СѓРґР°Р»РµРЅРёРµ {}", tauri_target.display()));
        ok_all &= rm(&tauri_target);
    }

    let dist = cfg.ui_dir().join("dist");
    if dist.exists() {
        info(&format!("СѓРґР°Р»РµРЅРёРµ {}", dist.display()));
        ok_all &= rm(&dist);
    }

    if deep {
        let nm = cfg.ui_dir().join("node_modules");
        if nm.exists() {
            info(&format!("СѓРґР°Р»РµРЅРёРµ {}", nm.display()));
            ok_all &= rm(&nm);
        }
    } else {
        warn("node_modules РѕСЃС‚Р°РІР»РµРЅ (РёСЃРїРѕР»СЊР·СѓР№С‚Рµ --deep С‡С‚РѕР±С‹ СѓРґР°Р»РёС‚СЊ)");
    }

    eprintln!();
    if ok_all {
        ok("РѕС‡РёСЃС‚РєР° Р·Р°РІРµСЂС€РµРЅР°");
        ExitCode::SUCCESS
    } else {
        err("РѕС‡РёСЃС‚РєР° РїСЂРѕС€Р»Р° СЃ РѕС€РёР±РєР°РјРё");
        ExitCode::FAILURE
    }
}

// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//  LINT
// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

fn run_lint(cfg: Config) -> ExitCode {
    info("Р›РёРЅС‚ РІРѕСЂРєСЃРїРµР№СЃР°");
    dim("  cargo check в†’ cargo clippy в†’ vue-tsc");
    eprintln!();

    let mut ok_all = true;

    info("[1/3] cargo check --workspace --all-targets");
    ok_all &= run(
        "cargo",
        &["check", "--workspace", "--all-targets"],
        &cfg.workspace_root,
    );

    info("[2/3] cargo clippy --workspace --all-targets -- -D warnings");
    ok_all &= run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &cfg.workspace_root,
    );

    info("[3/3] bun run typecheck (vue-tsc --noEmit)");
    ok_all &= run("bun", &["run", "typecheck"], &cfg.ui_dir());

    eprintln!();
    if ok_all {
        ok("Р»РёРЅС‚ РїСЂРѕР№РґРµРЅ");
        ExitCode::SUCCESS
    } else {
        err("Р»РёРЅС‚ РїСЂРѕРІР°Р»РёР»СЃСЏ");
        ExitCode::FAILURE
    }
}

// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//  INFO
// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

fn run_info(cfg: Config) -> ExitCode {
    let line = |k: &str, v: &str| println!("  {:<16}{}", k, v);
    println!("Pulsar VPN workspace");
    line("root", &cfg.workspace_root.display().to_string());
    line("target", &cfg.target_dir.display().to_string());
    line("daemon bin", &cfg.daemon_built_path().display().to_string());
    line("daemon dest", &cfg.daemon_dest_path().display().to_string());
    line("Xray", &cfg.xray_expected_path().display().to_string());
    line("ui dir", &cfg.ui_dir().display().to_string());
    println!();
    println!("РРЅСЃС‚СЂСѓРјРµРЅС‚С‹:");
    if let Ok(v) = run_capture("rustc", &["--version"], &cfg.workspace_root) {
        line("rustc", v.trim());
    }
    if let Ok(v) = run_capture("cargo", &["--version"], &cfg.workspace_root) {
        line("cargo", v.trim());
    }
    if let Ok(v) = run_capture("bun", &["--version"], &cfg.workspace_root) {
        line("bun", v.trim());
    }
    if let Ok(v) = run_capture("node", &["--version"], &cfg.workspace_root) {
        line("node", v.trim());
    }
    println!();
    println!("Р‘РёРЅР°СЂРЅРёРєРё:");
    line("daemon built", &status(cfg.daemon_built_path().exists()));
    line("daemon bundled", &status(cfg.daemon_dest_path().exists()));
    line("Xray bundled", &status(cfg.xray_expected_path().exists()));
    ExitCode::SUCCESS
}

fn status(ok: bool) -> &'static str {
    if ok {
        "вњ“ РЅР°Р№РґРµРЅ"
    } else {
        "вњ— РѕС‚СЃСѓС‚СЃС‚РІСѓРµС‚"
    }
}

// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ
//  Process helpers
// в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђ

/// Р—Р°РїСѓСЃРєР°РµС‚ РєРѕРјР°РЅРґСѓ, РїСЂРѕРєРёРґС‹РІР°СЏ stdout/stderr РїРѕР»СЊР·РѕРІР°С‚РµР»СЋ. Р’РѕР·РІСЂР°С‰Р°РµС‚ СѓСЃРїРµС… РїРѕ РєРѕРґСѓ РІС‹С…РѕРґР°.
fn run(prog: &str, args: &[&str], cwd: &Path) -> bool {
    let mut cmd = Command::new(prog);
    cmd.args(args).current_dir(cwd);
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            err(&format!("РЅРµ СѓРґР°Р»РѕСЃСЊ Р·Р°РїСѓСЃС‚РёС‚СЊ `{}`: {}", prog, e));
            if prog == "bun" && cfg!(windows) {
                dim("    РїРѕРґСЃРєР°Р·РєР°: СѓСЃС‚Р°РЅРѕРІРёС‚Рµ bun (https://bun.sh) Рё РїРµСЂРµР·Р°РїСѓСЃС‚РёС‚Рµ С‚РµСЂРјРёРЅР°Р»");
            } else if prog == "cargo" {
                dim("    РїРѕРґСЃРєР°Р·РєР°: СѓСЃС‚Р°РЅРѕРІРёС‚Рµ Rust С‡РµСЂРµР· https://rustup.rs");
            }
            return false;
        }
    };
    status.success()
}

/// РўРѕ Р¶Рµ, С‡С‚Рѕ `run`, РЅРѕ РїРѕРґР°РІР»СЏРµС‚ РІС‹РІРѕРґ (РґР»СЏ cargo clean Рё РїРѕРґРѕР±РЅРѕРіРѕ).
fn run_quiet(prog: &str, args: &[&str], cwd: &Path) -> bool {
    let mut cmd = Command::new(prog);
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match cmd.status() {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// Р—Р°С…РІР°С‚С‹РІР°РµС‚ stdout (РґР»СЏ info).
fn run_capture(prog: &str, args: &[&str], cwd: &Path) -> Result<String, ()> {
    let out = Command::new(prog)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| ())?;
    if !out.status.success() {
        return Err(());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn rm(path: &Path) -> bool {
    let meta = match path.metadata() {
        Ok(m) => m,
        Err(_) => return true, // СѓР¶Рµ РЅРµС‚ вЂ” СЃС‡РёС‚Р°РµРј СѓСЃРїРµС…РѕРј
    };
    let res = if meta.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match res {
        Ok(_) => true,
        Err(e) => {
            err(&format!("РЅРµ СѓРґР°Р»РѕСЃСЊ СѓРґР°Р»РёС‚СЊ {}: {}", path.display(), e));
            false
        }
    }
}

