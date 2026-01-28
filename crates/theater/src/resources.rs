//! ECS Resources for the theater process

use bevy::animation::graph::AnimationNodeIndex;
use bevy::prelude::*;
use common::AppPaths;
use common::script::BehaviorScript;
use std::collections::HashMap;
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

/// Script library resource - stores all loaded behavior scripts
#[derive(Resource, Default)]
pub struct ScriptLibrary {
    scripts: HashMap<String, BehaviorScript>,
}

impl ScriptLibrary {
    /// Add a script to the library
    pub fn add(&mut self, script: BehaviorScript) {
        self.scripts.insert(script.id.clone(), script);
    }

    /// Get a script by its ID
    pub fn get(&self, id: &str) -> Option<&BehaviorScript> {
        self.scripts.get(id)
    }

    /// Load all builtin scripts into the library
    pub fn load_builtin(&mut self) {
        for script in BehaviorScript::builtin_scripts() {
            self.add(script);
        }
    }

    /// Get a list of all script IDs
    pub fn script_ids(&self) -> Vec<&String> {
        self.scripts.keys().collect()
    }
}

/// Animation mapping resource - maps animation names to graph node indices
#[derive(Resource, Default)]
pub struct AnimationMap {
    /// Map from animation name to AnimationNodeIndex
    pub name_to_index: HashMap<String, AnimationNodeIndex>,
    /// Handle to the AnimationGraph
    pub graph: Option<Handle<AnimationGraph>>,
}
