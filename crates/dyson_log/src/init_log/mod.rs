use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::daily;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Initialize logging with optional Tokio console support.
///
/// Keep the returned guard alive for the lifetime of your application to ensure
/// all logs are flushed to disk.
///
/// Example:
///
/// ```ignore
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let _guard = dyson_log::init_log(); // Keep the guard alive
///     // ... run your app
///     Ok(())
/// }
/// ```
pub fn init_log() -> Option<WorkerGuard> {
    if std::env::var("TOKIO_CONSOLE_ENABLE").unwrap_or_default() == "1" {
        console_subscriber::init();
    }
    if std::env::var("ENABLE_LOG").unwrap_or("1".to_string()) == "1" {
        let log_path = std::env::var("LOG_PATH").unwrap_or("logs".to_string());
        let log_file = std::env::var("LOG_FILE").unwrap_or("app.log".to_string());
        let file_appender = daily(log_path, log_file);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_target(true)
            .with_line_number(true)
            .json()
            .with_span_list(true)
            .with_filter(EnvFilter::from_default_env());

        let console_layer = tracing_subscriber::fmt::layer()
            .with_line_number(true)
            .with_target(true)
            .with_filter(EnvFilter::from_default_env());

        tracing_subscriber::registry()
            .with(file_layer)
            .with(console_layer)
            .init();

        return Some(_guard);
    };

    None
}
