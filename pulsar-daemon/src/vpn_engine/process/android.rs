use crate::vpn_engine::MetricsSnapshot;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Not implemented")]
    NotImplemented,
}

pub struct PlatformProcessImpl {}

impl PlatformProcessImpl {
    pub fn new() -> Result<Self, String> {
        Ok(Self {})
    }

    pub fn noop() -> Self {
        Self {}
    }

    pub fn binary_path(&self) -> Option<PathBuf> {
        None
    }

    pub async fn is_running(&self) -> bool {
        // Заглушка для Android
        true
    }

    pub async fn start(&self, _config_json: &str) -> Result<(), PlatformError> {
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), PlatformError> {
        Ok(())
    }

    pub async fn sample_metrics(&self) -> MetricsSnapshot {
        MetricsSnapshot::default()
    }
}
