//! ECS Components for the theater process

use bevy::prelude::*;

/// Marker component for the pet entity
#[derive(Component)]
pub struct PetMarker;

/// Component to store the pet's animation state
#[derive(Component, Default)]
pub struct PetAnimationState {
    /// The name of the currently playing animation
    pub current_animation: Option<String>,
    /// The entity containing the AnimationPlayer component
    pub player_entity: Option<Entity>,
}

/// Component to track replay state for a pet entity
#[derive(Component)]
pub struct ReplayState {
    /// The ID of the currently executing script
    pub script_id: String,
    /// Time elapsed since the script started (in seconds)
    pub elapsed_time: f32,
    /// Index of the next keyframe to execute
    pub next_keyframe_index: usize,
    /// Whether the script has completed
    pub completed: bool,
}

impl ReplayState {
    /// Create a new ReplayState for the given script
    pub fn new(script_id: String) -> Self {
        Self {
            script_id,
            elapsed_time: 0.0,
            next_keyframe_index: 0,
            completed: false,
        }
    }

    /// Switch to a new script, resetting all state
    pub fn switch_to(&mut self, script_id: String) {
        self.script_id = script_id;
        self.elapsed_time = 0.0;
        self.next_keyframe_index = 0;
        self.completed = false;
    }
}

/// Component for smooth position tweening (MoveTo action)
#[derive(Component)]
pub struct MovementTween {
    /// Starting position
    pub start_position: Vec3,
    /// Target position
    pub target_position: Vec3,
    /// Duration of the tween in seconds
    pub duration: f32,
    /// Time elapsed since tween started
    pub elapsed: f32,
}

/// Component for smooth scale tweening (Scale action)
#[derive(Component)]
pub struct ScaleTween {
    /// Starting scale
    pub start_scale: Vec3,
    /// Target scale
    pub target_scale: Vec3,
    /// Duration of the tween in seconds
    pub duration: f32,
    /// Time elapsed since tween started
    pub elapsed: f32,
}

/// Component for bounce tweening (Bounce action)
#[derive(Component)]
pub struct BounceTween {
    /// Base Y position to bounce from
    pub base_y: f32,
    /// Peak height of the bounce
    pub height: f32,
    /// Duration of the bounce in seconds
    pub duration: f32,
    /// Time elapsed since bounce started
    pub elapsed: f32,
}

/// Component for smooth rotation tweening (Spin action)
#[derive(Component)]
pub struct RotationTween {
    /// Starting rotation
    pub start_rotation: Quat,
    /// Total rotation angle in radians
    pub total_angle: f32,
    /// Duration of the tween in seconds
    pub duration: f32,
    /// Time elapsed since tween started
    pub elapsed: f32,
}
