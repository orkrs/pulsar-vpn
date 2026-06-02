//! Windows-реализация [`PlatformProcess`]: дочерний процесс sing-box + sysinfo.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::io::AsyncBufReadExt;
use sysinfo::{System, SystemExt, NetworkExt};

use crate::vpn_engine::MetricsSnapshot;

/// Ошибка платформенной операции.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("sing-box не найден: {0}")]
    SingBoxMissing(PathBuf),
    #[error("не удалось запустить sing-box: {0}")]
    Spawn(String),
    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),
    #[error("sing-box завершился с кодом {0}")]
    Exited(i32),
}

/// Реальная реализация для Windows. Держит дочерний `Child` под `Mutex`,
/// чтобы команды конкурентно не плодили процессы.
pub struct PlatformProcessImpl {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    child: Option<Child>,
    singbox_path: PathBuf,
}

impl PlatformProcessImpl {
    pub fn new() -> Result<Self, String> {
        let singbox_path = find_singbox()
            .ok_or_else(|| "sing-box не найден рядом с бинарём и в PATH".to_string())?;
        Ok(Self {
            inner: Arc::new(Mutex::new(Inner { child: None, singbox_path })),
        })
    }

    pub fn noop() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner { child: None, singbox_path: PathBuf::new() })),
        }
    }

    pub fn binary_path(&self) -> Option<PathBuf> {
        let g = self.inner.try_lock().ok()?;
        if g.singbox_path.as_os_str().is_empty() { None } else { Some(g.singbox_path.clone()) }
    }

    pub async fn start(&self, config_json: &str) -> Result<(), PlatformError> {
        let mut g = self.inner.lock().await;
        if g.child.is_some() { return Ok(()); } // уже запущен

        // Записываем конфиг во временный файл.
        let cfg_dir = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("PulsarVPN");
        let _ = std::fs::create_dir_all(&cfg_dir);
        let cfg_path = cfg_dir.join("config.json");
        tokio::fs::write(&cfg_path, config_json).await?;

        // Запускаем sing-box.
        let mut cmd = Command::new(&g.singbox_path);
        cmd.arg("run").arg("-c").arg(&cfg_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            // CREATE_NO_WINDOW — чтобы не маячило консольное окно.
            // tokio::process::Command на Windows нативно предоставляет
            // метод creation_flags (0x08000000), отдельный use не нужен.
            cmd.creation_flags(0x08000000);
        }

        let mut child = cmd.spawn().map_err(|e| PlatformError::Spawn(e.to_string()))?;

        // Перехватываем логи в tracing.
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut lines = tokio::io::BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!(target: "singbox", "{line}");
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut lines = tokio::io::BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(target: "singbox", "{line}");
                }
            });
        }

        // Небольшая задержка — если процесс сразу упал, заметим.
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
            // Сначала taskkill (убивает и дочерние процессы), затем kill().
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
        // Делаем раз в секунду, refresh_networks() дешёвый (только данные
        // по уже-известным интерфейсам), refresh_networks_list() — дорогой,
        // зовём только при смене состояния подключения.
        // Здесь мы внутри метрик-цикла, и состояние нас не интересует —
        // если TUN нет, вернём нули (UI их и так рисует при Disconnected).
        use std::sync::atomic::{AtomicBool, Ordering};
        static INIT: AtomicBool = AtomicBool::new(false);
        if !INIT.swap(true, Ordering::Relaxed) {
            self.refresh_list();
        }
        self.refresh_data()
    }

    fn refresh_list(&self) {
        // sysinfo — sync API, делаем в spawn_blocking, чтобы не блокировать
        // tokio executor (на Windows refresh_networks_list дёргает WMI).
        let _ = tokio::task::spawn_blocking(|| {
            let mut s = System::new_all();
            s.refresh_networks_list();
        });
    }

    fn refresh_data(&self) -> MetricsSnapshot {
        // Тоже через spawn_blocking, потому что sysinfo — sync.
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
            MetricsSnapshot { download_speed: td, upload_speed: tu }
        })
    }
}

/// Ищет sing-box рядом с текущим бинарём, потом в PATH.
fn find_singbox() -> Option<PathBuf> {
    let candidates = [
        // рядом с pulsar-daemon.exe — стандартная раскладка
        std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("sing-box.exe"))),
        Some(PathBuf::from("sing-box.exe")),
    ];
    for c in candidates.into_iter().flatten() {
        if c.exists() { return Some(c); }
    }
    which_singbox()
}

#[cfg(target_os = "windows")]
fn which_singbox() -> Option<PathBuf> {
    // Минимальный which — без лишней зависимости.
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let p = dir.join("sing-box.exe");
            if p.exists() { return Some(p); }
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn which_singbox() -> Option<PathBuf> { None }
