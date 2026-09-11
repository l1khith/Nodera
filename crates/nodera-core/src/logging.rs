use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize tracing subscriber for testing or development.
/// Safely callable multiple times (subsequent calls will be no-ops).
pub fn init_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_test_writer()
            .try_init();
    });
}
