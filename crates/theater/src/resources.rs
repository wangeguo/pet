//! ECS Resources for the theater process

use bevy::prelude::*;
use common::AppPaths;
use std::path::PathBuf;

/// Configuration resource for the theater
#[derive(Resource)]
pub struct TheaterConfig {
    /// Path to the GLB model file
    pub model_path: Option<PathBuf>,
    /// Scale factor for the pet
    #[allow(dead_code)]
    pub pet_scale: f32,
    /// Initial window position (x, y)
    pub window_position: (i32, i32),
    /// Application paths (for config reload in Phase 3)
    #[allow(dead_code)]
    pub paths: AppPaths,
}

/// Resource to track pet model loading state
#[derive(Resource, Default)]
pub struct PetModelState {
    pub spawned: bool,
    pub gltf_handle: Option<Handle<Gltf>>,
}

/// Resource to track window dragging state
#[derive(Resource, Default)]
pub struct DragState {
    pub is_dragging: bool,
    pub drag_start_cursor: Option<Vec2>,
    pub drag_start_window: Option<IVec2>,
}
