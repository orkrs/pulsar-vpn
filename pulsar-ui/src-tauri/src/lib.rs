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
    use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jstring};

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

            let stdout = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(&asset_directory).join("xray.log"))
                .map_err(|e| format!("cannot open Xray log: {e}"))?;
            let stderr = stdout
                .try_clone()
                .map_err(|e| format!("cannot clone Xray log: {e}"))?;
            let spawned = Command::new(binary)
                .args(["run", "-config"])
                .arg(config_path)
                .current_dir(&asset_directory)
                .env("XRAY_TUN_FD", tun_fd.to_string())
                .env("XRAY_LOCATION_ASSET", &asset_directory)
                .stdout(Stdio::from(stdout))
                .stderr(Stdio::from(stderr))
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

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yann1n_pulsarvpn_PulsarVpnService_nativeIsXrayRunning(
        _env: JNIEnv,
        _service: JObject,
    ) -> jboolean {
        let Ok(mut guard) = child().lock() else {
            return JNI_FALSE;
        };
        let Some(process) = guard.as_mut() else {
            return JNI_FALSE;
        };
        match process.try_wait() {
            Ok(None) => JNI_TRUE,
            _ => {
                guard.take();
                JNI_FALSE
            }
        }
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
