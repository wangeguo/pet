//! Bevy application configuration and setup

use crate::plugins::{InteractionPlugin, PetPlugin, ReplayPlugin};
use crate::resources::TheaterConfig;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::window::{PresentMode, WindowLevel, WindowResolution};
use bevy::winit::{UpdateMode, WinitSettings};
use common::{AppConfig, AppPaths};

/// Default window size for the pet theater
const DEFAULT_WINDOW_SIZE: u32 = 400;

/// Run the theater application
pub fn run_theater() -> common::Result<()> {
    let paths = AppPaths::new()?;
    let config = AppConfig::load(&paths)?;

    let theater_config = TheaterConfig {
        model_path: config.get_active_pet().map(|pet| pet.model_path.clone()),
        pet_scale: config.pet_scale,
        window_position: (config.pet_position.x, config.pet_position.y),
        paths,
    };

    let assets_dir = AppPaths::find_assets_dir()?;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: assets_dir.to_string_lossy().to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pet".into(),
                        resolution: WindowResolution::new(DEFAULT_WINDOW_SIZE, DEFAULT_WINDOW_SIZE),
                        transparent: true,
                        decorations: false,
                        window_level: WindowLevel::AlwaysOnTop,
                        present_mode: PresentMode::AutoVsync,
                        resizable: false,
                        #[cfg(target_os = "macos")]
                        composite_alpha_mode: bevy::window::CompositeAlphaMode::PostMultiplied,
                        position: WindowPosition::At(IVec2::new(
                            theater_config.window_position.0,
                            theater_config.window_position.1,
                        )),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        WgpuSettings {
                            backends: Some(bevy::render::settings::Backends::all()),
                            ..default()
                        },
                    ),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::NONE))
        // Limit frame rate to reduce CPU usage - desktop pet doesn't need 60fps
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(100)),
            unfocused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(100)),
        })
        .insert_resource(theater_config)
        .add_plugins(PetPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(ReplayPlugin)
        .run();

    Ok(())
}
