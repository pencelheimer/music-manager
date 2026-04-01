use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!(
                "info,{}=debug,axum::rejection=trace",
                env!("CARGO_PKG_NAME")
            )
            .into()
        }))
        .with(fmt::layer().pretty())
        .init()
}
