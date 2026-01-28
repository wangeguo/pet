//! Pet Theater - Bevy-based 3D pet rendering process
//!
//! This is the main theater process that renders the 3D pet model
//! in a transparent, always-on-top window.

mod app;
mod components;
mod plugins;
mod resources;
mod systems;

use app::run_theater;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet_theater=info".parse().unwrap())
                .add_directive("bevy_render=warn".parse().unwrap())
                .add_directive("bevy_winit=warn".parse().unwrap())
                .add_directive("wgpu=error".parse().unwrap())
                .add_directive("naga=error".parse().unwrap()),
        )
        .init();

    info!("Theater process starting...");

    if let Err(e) = run_theater() {
        tracing::error!("Theater error: {e}");
        std::process::exit(1);
    }
}
