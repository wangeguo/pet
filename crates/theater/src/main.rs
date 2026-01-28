use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet_theater=debug".parse().unwrap()),
        )
        .init();

    info!("Theater process started (placeholder for Phase 2)");

    // Phase 2 will implement the full Bevy application
    // For now, just keep the process running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
