use core_bin::telemetry::init_tracing;

fn main() {
    init_tracing();

    tracing::info!("Core is starting");
}
