use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,core_bin=debug,core_lib=debug".into()),
        )
        .with(fmt::layer().pretty())
        .init()
}
