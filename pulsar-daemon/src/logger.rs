use std::path::PathBuf;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Инициализация логирования для демона
///
/// Настраивает:
/// - Ротацию логов по дням в %LOCALAPPDATA%\PulsarVPN\logs\
/// - Формат: timestamp (мс), уровень, модуль, файл:строка, thread, сообщение
/// - Уровень из PULSAR_LOG_LEVEL или по умолчанию INFO
/// - Очистку логов старше 30 дней
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_log_dir()?;
    std::fs::create_dir_all(&log_dir)?;

    // Ротация по дням: файлы вида pulsar-daemon-2025-02-22.log
    let file_appender = rolling::daily(&log_dir, "pulsar-daemon");
    let (non_blocking_appender, guard) = non_blocking(file_appender);
    // The guard owns the logging worker; dropping it here silently stops file logs.
    Box::leak(Box::new(guard));

    // Формат: timestamp (мс), уровень, модуль, файл:строка, thread, сообщение
    let file_layer = fmt::Layer::default()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE)
        .boxed();

    // Консольный вывод (для отладки)
    let console_layer = fmt::Layer::default()
        .with_writer(std::io::stdout)
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .boxed();

    // Уровень логирования из переменной окружения или по умолчанию INFO
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    Registry::default()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    // Запускаем очистку старых логов в фоне
    let log_dir_clone = log_dir.clone();
    tokio::spawn(async move {
        cleanup_old_logs(&log_dir_clone).await;
    });

    tracing::info!("Logger initialized. Logs directory: {:?}", log_dir);
    Ok(())
}

fn get_log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    Ok(PathBuf::from(local_app_data).join("PulsarVPN").join("logs"))
}

/// Очистка логов старше 30 дней
async fn cleanup_old_logs(log_dir: &PathBuf) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let thirty_days_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (30 * 24 * 60 * 60);

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(modified_secs) = modified.duration_since(UNIX_EPOCH) {
                        if modified_secs.as_secs() < thirty_days_ago {
                            if let Err(e) = std::fs::remove_file(entry.path()) {
                                tracing::warn!(
                                    "Failed to delete old log file {:?}: {}",
                                    entry.path(),
                                    e
                                );
                            } else {
                                tracing::debug!("Deleted old log file: {:?}", entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
}
