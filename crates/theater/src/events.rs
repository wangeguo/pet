//! Messages for the Replay system

use bevy::prelude::*;
use common::script::Action;

/// Message to request switching to a different behavior script
#[derive(Message)]
pub struct SwitchScriptEvent {
    /// The ID of the script to switch to
    pub script_id: String,
    /// If true, ignore the interruptible flag of the current script
    pub force: bool,
}

/// Message to execute a specific action from a keyframe
#[derive(Message)]
pub struct ExecuteActionEvent {
    /// The entity to execute the action on
    pub entity: Entity,
    /// The action to execute
    pub action: Action,
}

/// Message to request playing a specific animation
#[derive(Message)]
pub struct PlayAnimationEvent {
    /// The entity to play the animation on
    pub entity: Entity,
    /// The name of the animation to play
    pub animation_name: String,
}

/// Message triggered when the pet is clicked (not dragged)
#[derive(Message)]
pub struct PetClickedEvent;

/// Message triggered when a script has completed
#[derive(Message)]
pub struct ScriptCompletedEvent {
    /// The entity whose script completed
    pub entity: Entity,
    /// The ID of the completed script
    pub script_id: String,
    /// The next script to transition to (if specified)
    pub next_script: Option<String>,
}
