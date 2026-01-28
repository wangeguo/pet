mod process;

use common::paths::AppPaths;
use common::storage::StorageService;
use common::{AppConfig, Result};
use process::ProcessManager;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet=debug".parse().unwrap()),
        )
        .init();

    info!("Starting Pet desktop companion...");

    let paths = AppPaths::new()?;
    paths.ensure_dirs()?;

    let storage = StorageService::new(paths.clone());
    storage.init_builtin_scripts()?;

    let config = AppConfig::load(&paths)?;
    info!("Loaded configuration with {} pets", config.pets.len());

    let mut manager = ProcessManager::new(paths.clone());

    if let Err(e) = manager.start_tray() {
        error!("Failed to start tray process: {e}");
        return Err(e);
    }

    // Always start theater - it will show the test model if no pet is configured
    if let Err(e) = manager.start_theater() {
        error!("Failed to start theater process: {e}");
    }

    let is_first_run = config.pets.is_empty();
    if is_first_run {
        info!("First run detected, opening manager...");
        if let Err(e) = manager.start_manager() {
            error!("Failed to start manager process: {e}");
        }
    }

    manager.run().await?;

    info!("Pet desktop companion stopped.");
    Ok(())
}
