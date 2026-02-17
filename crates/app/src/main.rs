mod process;

use common::autostart;
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

    if let Err(e) = autostart::sync_autostart(config.general.auto_start) {
        error!("Failed to sync auto-start setting: {e}");
    }

    let mut manager = ProcessManager::new(paths);
    manager.run(&config).await?;

    info!("Pet desktop companion stopped.");
    Ok(())
}
