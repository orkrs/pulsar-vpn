//! Платформенная абстракция над VPN-процессом.
//!
//! На Windows: порождает дочерний процесс `sing-box(.exe)`, читает его
//! stdout/stderr, опрашивает сетевые интерфейсы через `sysinfo`.
//!
//! На Android: нативный sing-box запускается в адресном пространстве
//! самого приложения (через JNI), а трафик перехватывается через
//! Android `VpnService` (см. `android.rs`). Этот модуль на Android
//! сейчас — заглушка, задокументированная в [`android`].

use std::path::PathBuf;
use std::sync::Arc;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "windows")]
pub use windows::PlatformProcessImpl;
#[cfg(target_os = "android")]
pub use android::PlatformProcessImpl;

/// Тип-обёртка: на каждой платформе своя реализация, подключаемая через `mod`.
/// Используем новый-тип поверх `Arc<PlatformProcessImpl>`, чтобы реализации
/// могли иметь разные наборы полей и оборачиваться в Arc единообразно.
pub struct PlatformProcess(Arc<PlatformProcessImpl>);

impl std::ops::Deref for PlatformProcess {
    type Target = PlatformProcessImpl;
    fn deref(&self) -> &PlatformProcessImpl { &*self.0 }
}

impl Clone for PlatformProcess {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

impl PlatformProcess {
    pub fn new() -> Result<Self, String> {
        Ok(Self(Arc::new(PlatformProcessImpl::new()?)))
    }

    /// Заглушка для случая, когда `VpnEngine::new()` не смог найти
    /// sing-box / VpnService. Все вызовы возвращают ошибки.
    pub fn noop() -> Self {
        Self(Arc::new(PlatformProcessImpl::noop()))
    }

    /// Где лежит sing-box на текущей платформе (для UI-диагностики).
    pub fn binary_path(&self) -> Option<PathBuf> {
        self.0.binary_path()
    }
}
