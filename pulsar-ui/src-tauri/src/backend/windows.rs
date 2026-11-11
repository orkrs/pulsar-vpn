//! Windows-бэкенд: запускает `pulsar-daemon.exe` и общается с ним по gRPC.
//!
//! Жизненный цикл:
//! 1. `WindowsBackend::new()`:
//!    - тасккиллим "ghost"-процессы (если Tauri убили раньше демона);
//!    - спавним `pulsar-daemon.exe <token>` как дочерний процесс;
//!    - в фоне подключаемся к gRPC `127.0.0.1:50051` с ретраями;
//!    - сохраняем `Client` + `Child` в `Arc<Mutex<...>>`.
//! 2. `toggle()`: вызывает gRPC-метод `ToggleConnection`.
//! 3. `subscribe_metrics()`: gRPC-стрим `StreamMetrics` заворачивается
//!    в `mpsc::Receiver<MetricsEvent>`.
//! 4. `shutdown()`: gRPC `Shutdown` + kill процесса демона.

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

/// gRPC-клиент, обёрнутый в interceptor с токеном.
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

/// Состояние Windows-бэкенда. Делаем `pub(crate)` поля приватными
/// через `Arc<Mutex<...>>` и проксируем только нужные методы.
pub struct WindowsBackend {
    /// Текущий gRPC-клиент. `None` — пока не подключились (демон ещё
    /// не принял соединение).
    client: Arc<Mutex<Option<GrpcClient>>>,
    /// Дочерний процесс демона. Держим, чтобы убить при выходе.
    daemon_process: Arc<Mutex<Option<std::process::Child>>>,
    /// Подписка на метрики выдаётся однократно.
    metrics_rx: Arc<Mutex<Option<mpsc::Receiver<MetricsEvent>>>>,
    /// Tauri AppHandle нужен, чтобы эмитить `vpn_log`.
    app_handle: tauri::AppHandle,
}

impl WindowsBackend {
    /// Публичная точка входа, вызываемая из `setup()`. Спавнит демон,
    /// подключается к gRPC, возвращает готовый бэкенд.
    pub fn attach_app(app: &tauri::App, logs_streaming: Arc<AtomicBool>) -> Result<Self, String> {
        let time_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let session_token = format!("pulsar-{}-{}", std::process::id(), time_nanos);

        // --- Зачистка "ghost"-процессов ---
        kill_ghosts();

        // --- Старт демона ---
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

        // --- Лог-флашер (батчинг вывода демона в webview) ---
        spawn_log_flusher(
            child.stdout.take(),
            child.stderr.take(),
            app_handle.clone(),
            logs_streaming.clone(),
        );

        // --- Сохраняем child для последующего kill ---
        let daemon_process = Arc::new(Mutex::new(None));
        let process_store = daemon_process.clone();
        tauri::async_runtime::spawn(async move {
            *process_store.lock().await = Some(child);
        });

        // --- gRPC-коннект в фоне с ретраями ---
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

    #[allow(dead_code)] // используется через Backend::state() (enum-dispatch)
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
        // Отдаём подписку однократно. mpsc::Receiver — не Clone, поэтому
        // просто отдаём его вызывающему; повторный вызов вернёт None.
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
        // `g` (MutexGuard) дропается здесь, потому что `take()` забрал
        // прошлое значение. Возвращаем сам `rx`.
        drop(g);
        Some(rx)
    }

    pub async fn shutdown(&self) {
        // gRPC shutdown
        if let Some(c) = self.client.lock().await.as_mut() {
            let _ = c.shutdown(Empty {}).await;
        }
        // Убиваем процесс
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

/// Батчинг stdout/stderr демона в `vpn_log` события webview.
/// Код практически идентичен тому, что раньше жил в main.rs setup().
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
