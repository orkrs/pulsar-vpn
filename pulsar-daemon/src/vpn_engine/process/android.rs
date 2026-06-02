//! Android-реализация [`PlatformProcess`] — ЗАГЛУШКА.
//!
//! На Android нельзя просто запустить sing-box как дочерний процесс:
//! 1. VPN-трафик нужно перехватывать через `android.net.VpnService`.
//!    Это Java/Kotlin API, которое Tauri-приложение должно вызвать
//!    из MainActivity (Activity, onActivityResult и т. д.).
//! 2. Сам sing-box нужно либо (а) собрать как нативный `.so` через
//!    NDK и подключить через JNI, либо (б) использовать
//!    `sing-box-for-android` (community-порт, поддерживается нерегулярно).
//! 3. Метрики скорости берутся из `TrafficStats` Android, а не из
//!    sysinfo.
//!
//! Поскольку Tauri 2 на Android живёт в одном процессе с Tauri-UI, всё
//! это делается in-process, без gRPC. Структура кода остаётся прежней —
//! меняется только реализация `PlatformProcessImpl`.
//!
//! ## Следующие шаги (НЕ в рамках текущей задачи, требуют NDK + Kotlin):
//! 1. Добавить `tauri-plugin-android` либо собственный JNI-мост.
//! 2. Сгенерировать Kotlin-обёртку `VpnService` в `gen/android/app/src/main/kotlin/`.
//! 3. Скомпилировать sing-box как `libsingbox.so` под `aarch64-linux-android`.
//! 4. Реализовать методы `PlatformProcessImpl` через JNI:
//!    - `start`: `VpnService.Builder` → `establish()` → `FileDescriptor`
//!      → передать в нативный sing-box через FFI.
//!    - `stop`: `vpnService.stopService()`.
//!    - `sample_metrics`: `TrafficStats.getUidTxBytes(packageUid)` минус
//!      предыдущее значение.
//! 5. Подключить `libsingbox.so` через `#[link(name = "singbox")]`.

use crate::vpn_engine::MetricsSnapshot;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Android VPN-бэкенд ещё не реализован")]
    NotImplemented,
}

pub struct PlatformProcessImpl {
    started: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl PlatformProcessImpl {
    pub fn new() -> Result<Self, String> {
        Ok(Self { started: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)) })
    }

    pub fn noop() -> Self { Self::new().unwrap() }

    pub fn binary_path(&self) -> Option<std::path::PathBuf> { None }

    pub async fn start(&self, _config_json: &str) -> Result<(), PlatformError> {
        Err(PlatformError::NotImplemented)
    }

    pub async fn stop(&self) -> Result<(), PlatformError> {
        self.started.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    pub async fn sample_metrics(&self) -> MetricsSnapshot {
        MetricsSnapshot::default()
    }
}
