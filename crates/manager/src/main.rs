use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet_manager=debug".parse().unwrap()),
        )
        .init();

    info!("Manager process started (placeholder for Phase 4)");

    // Phase 4 will implement the full Iced application
    // For now, just keep the process running briefly
    std::thread::sleep(std::time::Duration::from_secs(5));
    info!("Manager process exiting");
}
