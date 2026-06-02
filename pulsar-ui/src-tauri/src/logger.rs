use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer, EnvFilter, Registry,
};
use tracing_appender::{non_blocking, rolling};

// Глобальное хранилище для гарда, чтобы он не удалялся из памяти
static LOG_GUARD: OnceLock<non_blocking::WorkerGuard> = OnceLock::new();

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Определяем путь
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    let log_dir = PathBuf::from(local_app_data).join("PulsarVPN").join("logs");
    
    // Создаем папку, если нет
    std::fs::create_dir_all(&log_dir)?;

    // 2. Настраиваем ротацию файлов
    let file_appender = rolling::daily(&log_dir, "pulsar-ui");
    let (non_blocking_appender, guard) = non_blocking(file_appender);

    // 3. ВАЖНО: Сохраняем guard в глобальную переменную. 
    // Если этого не сделать, логи перестанут писаться сразу после выхода из этой функции.
    // .set вернет ошибку, если уже установлено, но нам это не важно.
    let _ = LOG_GUARD.set(guard);

    // 4. Настраиваем слои (файл + консоль)
    let file_layer = fmt::Layer::default()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        // with_file/with_line_number/with_thread_ids на Windows = syscall'ы
        // на каждую строку. Для файлового лога оставляем target+сообщение.
        .boxed();

    // Для отладки в консоли (если запущено через терминал).
    // ВАЖНО: здесь НЕ используем with_file/with_line_number/with_thread_ids —
    // на Windows это syscall'ы, и этот stderr всё равно перехватывается
    // Tauri'ом и эмитится в webview (см. main.rs setup), так что любой
    // "жир" формата напрямую бьёт по dispatcher'у WebView2.
    let console_layer = fmt::Layer::default()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .boxed();

    // 5. Инициализация
    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(file_layer)
        .with(console_layer);

    // 6. КРИТИЧЕСКОЕ ИСПРАВЛЕНИЕ: try_init() вместо init()
    // init() вызывает панику, если вызывается второй раз. try_init() просто вернет ошибку, которую мы игнорируем.
    if let Err(_) = subscriber.try_init() {
        eprintln!("Logger was already initialized. Skipping re-initialization.");
    } else {
        tracing::info!("Logger initialized successfully at {:?}", log_dir);
    }

    Ok(())
}