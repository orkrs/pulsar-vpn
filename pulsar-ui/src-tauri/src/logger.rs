use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

static LOG_GUARD: OnceLock<non_blocking::WorkerGuard> = OnceLock::new();

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Trace)
                .with_tag("PulsarVPN"),
        );
        return Ok(());
    }

    #[cfg(not(target_os = "android"))]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
        let log_dir = PathBuf::from(local_app_data).join("PulsarVPN").join("logs");
        std::fs::create_dir_all(&log_dir)?;

        let file_appender = rolling::daily(&log_dir, "pulsar-ui");
        let (non_blocking_appender, guard) = non_blocking(file_appender);
        let _ = LOG_GUARD.set(guard);

        let file_layer = fmt::Layer::default()
            .with_writer(non_blocking_appender)
            .with_ansi(false)
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .boxed();

        let console_layer = fmt::Layer::default()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_timer(UtcTime::rfc_3339())
            .with_target(true)
            .boxed();

        let subscriber = Registry::default()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(file_layer)
            .with(console_layer);

        if let Err(_) = subscriber.try_init() {
            eprintln!("Logger already initialized");
        }

        Ok(())
    }
}
