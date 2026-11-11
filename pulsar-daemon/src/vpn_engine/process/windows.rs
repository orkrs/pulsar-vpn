use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::{NetworkExt, System, SystemExt};
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::vpn_engine::MetricsSnapshot;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Xray not found: {0}")]
    XrayMissing(PathBuf),
    #[error("failed to start Xray: {0}")]
    Spawn(String),
    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),
    #[error("Xray exited with code {0}")]
    Exited(i32),
}

pub struct PlatformProcessImpl {
    inner: Arc<Mutex<Inner>>,
    /// Reused across metrics ticks — System::new_all() every sample burned ~10–25% CPU.
    sys: Arc<std::sync::Mutex<System>>,
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
            sys: Arc::new(std::sync::Mutex::new(System::new())),
        })
    }

    pub fn noop() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                child: None,
                xray_path: PathBuf::new(),
            })),
            sys: Arc::new(std::sync::Mutex::new(System::new())),
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

    pub async fn is_running(&self) -> bool {
        let mut g = self.inner.lock().await;
        if let Some(child) = g.child.as_mut() {
            // Если try_wait возвращает Ok(None), процесс всё ещё жив
            matches!(child.try_wait(), Ok(None))
        } else {
            false
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
        }

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

        // КРИТИЧЕСКИ ВАЖНО ДЛЯ ИЗБЕЖАНИЯ ОШИБКИ 23:
        if let Some(directory) = g.xray_path.parent() {
            cmd.current_dir(directory);
            cmd.env("XRAY_LOCATION_ASSET", directory);
        }

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| PlatformError::Spawn(e.to_string()))?;

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
        use std::sync::atomic::{AtomicBool, Ordering};
        static INIT: AtomicBool = AtomicBool::new(false);
        let sys = self.sys.clone();
        if !INIT.swap(true, Ordering::Relaxed) {
            if let Ok(mut s) = sys.lock() {
                s.refresh_networks_list();
            }
        }
        tokio::task::spawn_blocking(move || {
            let Ok(mut s) = sys.lock() else {
                return MetricsSnapshot {
                    download_speed: 0,
                    upload_speed: 0,
                };
            };
            s.refresh_networks();
            let mut td = 0u64;
            let mut tu = 0u64;
            for (name, data) in s.networks() {
                let n = name.to_lowercase();
                if n.contains("pulsar") || n.contains("wintun") {
                    td += data.total_received();
                    tu += data.total_transmitted();
                }
            }
            MetricsSnapshot {
                download_speed: td,
                upload_speed: tu,
            }
        })
        .await
        .unwrap_or(MetricsSnapshot {
            download_speed: 0,
            upload_speed: 0,
        })
    }
}

fn find_xray() -> Option<PathBuf> {
    let candidates = [
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
