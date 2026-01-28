//! ECS Components for the theater process

use bevy::prelude::*;

/// Marker component for the pet entity
#[derive(Component)]
pub struct PetMarker;

/// Component to store the pet's animation state (Phase 3)
#[derive(Component, Default)]
#[allow(dead_code)]
pub struct PetAnimationState {
    pub current_animation: Option<String>,
}
