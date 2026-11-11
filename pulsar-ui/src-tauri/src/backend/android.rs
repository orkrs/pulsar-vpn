use std::sync::Arc;

use jni::objects::{JClass, JObject, JValue};
use pulsar_daemon::{config_gen, vpn_engine::{ConnectRequest, MetricsEvent, VpnState}};
use tokio::sync::{Mutex, mpsc};

pub struct AndroidBackend {
    state: Arc<Mutex<VpnState>>,
}

impl AndroidBackend {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            state: Arc::new(Mutex::new(VpnState::Disconnected)),
        })
    }

    pub async fn connect(&self, req: ConnectRequest) -> Result<VpnState, String> {
        *self.state.lock().await = VpnState::Connecting;
        let result = config_gen::generate_xray_config(&req.profile_link, &req.routing)
            .map_err(|error| format!("Config generation failed: {error:#}"))
            .and_then(|config| android_request_vpn(&config));
        if let Err(error) = result {
            *self.state.lock().await = VpnState::Disconnected;
            return Err(error);
        }
        // Permission may still be on screen, but Android owns the continuation.
        *self.state.lock().await = VpnState::Connected;
        Ok(VpnState::Connected)
    }

    pub async fn disconnect(&self) -> Result<VpnState, String> {
        android_stop_vpn()?;
        *self.state.lock().await = VpnState::Disconnected;
        Ok(VpnState::Disconnected)
    }

    pub async fn state(&self) -> VpnState {
        let state = if android_vpn_running().unwrap_or(false) {
            VpnState::Connected
        } else {
            VpnState::Disconnected
        };
        *self.state.lock().await = state.clone();
        state
    }

    pub async fn subscribe_metrics(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        None
    }

    pub async fn shutdown(&self) {
        // Closing the UI must not tear down an Android foreground VPN service.
    }
}

enum AndroidCall<'a> {
    Start(&'a str),
    Stop,
    IsRunning,
}

fn call_main_activity(call: AndroidCall<'_>) -> Result<i32, String> {
    let context = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }
        .map_err(|error| format!("Java VM unavailable: {error}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|error| format!("cannot attach Android thread: {error}"))?;
    let app_context = unsafe { JObject::from_raw(context.context().cast()) };
    let loader = env
        .call_method(&app_context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .and_then(|value| value.l())
        .map_err(|error| format!("Android class loader unavailable: {error}"))?;
    let name = env
        .new_string("com.yann1n.pulsarvpn.MainActivity")
        .map_err(|error| error.to_string())?;
    let name = JObject::from(name);
    let class = env
        .call_method(
            loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )
        .and_then(|value| value.l())
        .map(JClass::from)
        .map_err(|error| format!("MainActivity class load failed: {error}"))?;

    match call {
        AndroidCall::Start(config) => {
            let config = env.new_string(config).map_err(|error| error.to_string())?;
            let config = JObject::from(config);
            env.call_static_method(
                class,
                "requestVpn",
                "(Landroid/content/Context;Ljava/lang/String;)I",
                &[JValue::Object(&app_context), JValue::Object(&config)],
            )
            .and_then(|value| value.i())
            .map_err(|error| format!("cannot start Android VPN: {error}"))
        }
        AndroidCall::Stop => {
            env.call_static_method(
                class,
                "stopVpn",
                "(Landroid/content/Context;)V",
                &[JValue::Object(&app_context)],
            )
            .map_err(|error| format!("cannot stop Android VPN: {error}"))?;
            Ok(0)
        }
        AndroidCall::IsRunning => env
            .call_static_method(
                class,
                "isVpnRunning",
                "(Landroid/content/Context;)Z",
                &[JValue::Object(&app_context)],
            )
            .and_then(|value| value.z())
            .map(i32::from)
            .map_err(|error| format!("cannot read Android VPN state: {error}")),
    }
}

fn android_request_vpn(config: &str) -> Result<(), String> {
    call_main_activity(AndroidCall::Start(config)).map(|_| ())
}

fn android_stop_vpn() -> Result<(), String> {
    call_main_activity(AndroidCall::Stop).map(|_| ())
}

fn android_vpn_running() -> Result<bool, String> {
    call_main_activity(AndroidCall::IsRunning).map(|value| value != 0)
}
