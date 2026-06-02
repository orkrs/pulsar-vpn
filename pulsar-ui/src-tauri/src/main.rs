//! Pulsar VPN Tauri-приложение.
//!
//! Логика разделена:
//! - [`backend`] — платформенная абстракция VPN (Windows gRPC / Android in-process).
//! - [`logger`] — инициализация tracing с записью в файл.
//! - Tauri-команды здесь: тонкие обёртки над Backend + UI-хелперы (пинг,
//!   подписка, уведомления).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod logger;

use std::collections::HashMap;
use std::sync::Arc;

use tauri::{Emitter, Manager, State};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder};
use tauri_plugin_notification::NotificationExt;

use backend::{Backend, ToggleRequest};
use pulsar_daemon::vpn_engine::MetricsEvent;
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
    /// "process_name" (Windows/Linux/macOS) или "package_name" (Android).
    apps_label: &'static str,
    apps_placeholder: &'static str,
    /// Приписывать ли .exe / нет.
    apps_append_suffix: bool,
    /// Работает ли в apps-list per-app routing вообще.
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
        // Per-app routing на Android возможен (через
        // VpnService.Builder.addAllowedApplication/Disallowed), но
        // требует отдельной реализации в android-stub процесса.
        // Пока backend возвращает NotImplemented, поэтому apps не
        // работают; см. pulsar-daemon/src/vpn_engine/process/android.rs.
        PlatformInfo {
            os: "android",
            apps_label: "Apps (package name)",
            apps_placeholder: "e.g. com.android.chrome (coming soon)",
            apps_append_suffix: false,
            apps_supported: false,
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos", target_os = "android")))]
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
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    tracing::info!(
        connect = payload.connect,
        mode = %payload.routing_config.mode,
        link_length = payload.link.len(),
        "toggle_vpn command called"
    );

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

    // Нормализуем GeoIP-коды: lowercase + валидация действия.
    // SagerNet/sing-geoip хранит файлы в lowercase (ru.srs, не RU.srs),
    // UI исторически отправлял "RU" — config_gen тоже это уже учитывает,
    // но делаем нормализацию ещё на границе Tauri, чтобы не зависеть от
    // корректности config_gen.
    let geoip_rules: HashMap<String, String> = payload
        .routing_config
        .geoip_rules
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_lowercase();
            (key, val)
        })
        .filter(|(k, v)| {
            // Только ISO 3166-1 alpha-2 (две буквы) и валидные действия.
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
            let _ = app.notification().builder()
                .title("PulsarVPN Error")
                .body(&o.message)
                .show();
            return Err(format!("Backend Error: {}", o.message));
        }
        Ok(o) if o.state == "Connected" => {
            let _ = app.notification().builder()
                .title("PulsarVPN")
                .body("VPN Connected successfully")
                .show();
        }
        Ok(o) if o.state == "Disconnected" => {
            let _ = app.notification().builder()
                .title("PulsarVPN")
                .body("VPN Disconnected")
                .show();
        }
        Err(e) => {
            tracing::error!(error = %e, "toggle_vpn failed");
            let _ = app.notification().builder()
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
async fn start_metrics_stream(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("start_metrics_stream command called");
    let mut rx = state
        .backend
        .subscribe_metrics()
        .await
        .ok_or_else(|| "metrics subscription already taken".to_string())?;

    tauri::async_runtime::spawn(async move {
        const MIN_INTERVAL_MS: u64 = 500;
        const MIN_DELTA_BYTES: u64 = 1024;
        const MIN_REL_DELTA: f64 = 0.01;
        let mut last_emit = std::time::Instant::now() - std::time::Duration::from_millis(MIN_INTERVAL_MS);
        let mut last_down: u64 = 0;
        let mut last_up: u64 = 0;
        let mut count: u64 = 0;

        while let Some(ev) = rx.recv().await {
            let MetricsEvent { download_speed, upload_speed } = ev;
            count += 1;
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_emit).as_millis() as u64;

            let dd = download_speed.abs_diff(last_down);
            let du = upload_speed.abs_diff(last_up);
            let max_prev = last_down.max(last_up).max(1);
            let rel = (dd.max(du) as f64) / (max_prev as f64);
            let changed_enough = dd >= MIN_DELTA_BYTES || du >= MIN_DELTA_BYTES || rel >= MIN_REL_DELTA;

            if elapsed >= MIN_INTERVAL_MS && changed_enough {
                if count % 120 == 0 {
                    tracing::trace!(metric_count = count, "Metrics stream active");
                }
                let _ = app.emit(
                    "vpn_metrics",
                    serde_json::json!({ "download": download_speed, "upload": upload_speed }),
                );
                last_emit = now;
                last_down = download_speed;
                last_up = upload_speed;
            }
        }
        tracing::warn!("Metrics stream ended");
    });
    Ok(())
}

#[tauri::command]
async fn check_ping(link: String) -> Result<u32, String> {
    tracing::debug!(link_length = link.len(), "check_ping command called");
    let url = url::Url::parse(&link).map_err(|e| format!("Invalid URL: {}", e))?;
    let host = url.host_str().ok_or_else(|| "No host".to_string())?;
    let port = url.port().unwrap_or(443);

    tracing::debug!(host = host, port = port, "Starting ping check");
    let start = std::time::Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(format!("{}:{}", host, port)),
    ).await {
        Ok(Ok(_)) => {
            let elapsed = start.elapsed().as_millis() as u32;
            tracing::debug!(host = host, port = port, ping_ms = elapsed, "Ping successful");
            Ok(elapsed)
        }
        Ok(Err(e)) => {
            tracing::warn!(host = host, port = port, error = %e, "Ping connection failed");
            Err("Connection failed".into())
        }
        Err(_) => {
            tracing::warn!(host = host, port = port, "Ping timeout");
            Err("Timeout".into())
        }
    }
}

#[tauri::command]
async fn fetch_subscription(url: String) -> Result<SubscriptionResponse, String> {
    tracing::info!(subscription_url = %url, "fetch_subscription command called");
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    let userinfo = resp.headers()
        .get("subscription-userinfo")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let decoded = STANDARD.decode(text.trim()).unwrap_or_else(|_| text.clone().into_bytes());
    let decoded_str = String::from_utf8_lossy(&decoded);
    let nodes: Vec<String> = decoded_str.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    tracing::info!(profile_count = nodes.len(), has_userinfo = userinfo.is_some(), "Subscription parsed successfully");
    Ok(SubscriptionResponse { nodes, userinfo })
}

#[tauri::command]
fn set_minimize_to_tray(enable: bool) {
    tracing::debug!(enable = enable, "set_minimize_to_tray command called");
}

#[tauri::command]
fn set_logs_streaming(enable: bool) {
    // На Windows флаг хранится в WindowsBackend и доступен через
    // logs_streaming::FLAG; на Android — noop.
    backend::set_logs_streaming_global(enable);
    tracing::debug!(enable = enable, "set_logs_streaming command called");
}

#[tauri::command]
fn log_message(level: String, module: String, message: String, request_id: Option<String>) {
    match level.as_str() {
        "TRACE" => tracing::trace!(request_id = ?request_id, module = %module, "{}", message),
        "DEBUG" => tracing::debug!(request_id = ?request_id, module = %module, "{}", message),
        "INFO" => tracing::info!(request_id = ?request_id, module = %module, "{}", message),
        "WARN" => tracing::warn!(request_id = ?request_id, module = %module, "{}", message),
        "ERROR" => tracing::error!(request_id = ?request_id, module = %module, "{}", message),
        "FATAL" => tracing::error!(request_id = ?request_id, module = %module, "FATAL: {}", message),
        _ => tracing::info!(request_id = ?request_id, module = %module, "{}", message),
    }
}

/// Создаёт tray с пунктами "Open" и "Exit". На Android tray-иконки
/// нет — игнорируется.
fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Open PulsarVPN", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
    let mut tray_builder = TrayIconBuilder::with_id("main_tray")
        .tooltip("Pulsar VPN")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => { app.exit(0); }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }
    if let Err(e) = tray_builder.build(app) {
        tracing::error!("Failed to build tray icon (continuing anyway): {}", e);
    }
    Ok(())
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("CRITICAL RUST PANIC:\n{:#?}", info);
        eprintln!("{}", msg);
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let log_dir = std::path::PathBuf::from(local_app_data).join("PulsarVPN").join("logs");
            let _ = std::fs::create_dir_all(&log_dir);
            let crash_file = log_dir.join("ui-crash.log");
            let _ = std::fs::write(crash_file, msg);
        }
    }));

    if let Err(e) = logger::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }
    tracing::info!("Pulsar VPN UI starting...");

    let logs_streaming_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    backend::set_logs_streaming_handle(logs_streaming_flag.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(move |app| {
            tracing::info!("Tauri setup started");
            build_tray(app)?;
            // Бэкенд создаётся после tray, чтобы лог-флашер мог
            // эмитить `vpn_log` в webview.
            let backend = Backend::for_tauri(app, logs_streaming_flag.clone())
                .map_err(|e| -> Box<dyn std::error::Error> { format!("backend init: {e}").into() })?;
            app.manage(AppState { backend });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_vpn,
            start_metrics_stream,
            check_ping,
            fetch_subscription,
            set_minimize_to_tray,
            set_logs_streaming,
            log_message,
            get_platform_info,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("Error initializing Tauri")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let state: State<AppState> = app_handle.state();
                let backend = state.backend.clone();
                tauri::async_runtime::spawn(async move {
                    backend.shutdown().await;
                });
            }
        });
}
