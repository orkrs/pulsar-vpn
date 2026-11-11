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

    pub fn noop() -> Self {
        Self(Arc::new(PlatformProcessImpl::noop()))
    }

    pub fn binary_path(&self) -> Option<PathBuf> {
        self.0.binary_path()
    }

    pub async fn is_running(&self) -> bool {
        self.0.is_running().await
    }
}
