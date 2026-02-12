//! Replay plugin - manages behavior script playback

use bevy::prelude::*;
use tracing::info;

use crate::components::{PetMarker, ReplayState};
use crate::events::{
    ExecuteActionEvent, PetClickedEvent, PlayAnimationEvent, ScriptCompletedEvent,
    SwitchScriptEvent,
};
use crate::resources::{AnimationMap, PetModelState, ScriptLibrary};
use crate::systems::{
    advance_replay_time, check_script_completion, dispatch_actions, execute_keyframes,
    handle_pet_click, handle_script_transition, link_animation_player, play_animation,
    setup_animation_graph, update_bounce_tween, update_movement_tween, update_scale_tween,
};

/// Plugin for behavior script replay system
pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ScriptLibrary>()
            .init_resource::<AnimationMap>()
            // Messages
            .add_message::<SwitchScriptEvent>()
            .add_message::<ExecuteActionEvent>()
            .add_message::<PlayAnimationEvent>()
            .add_message::<PetClickedEvent>()
            .add_message::<ScriptCompletedEvent>()
            // Startup systems
            .add_systems(Startup, load_script_library)
            // Update systems
            .add_systems(
                Update,
                (
                    // Animation setup (runs once when model is loaded)
                    setup_animation_graph,
                    link_animation_player,
                    // Initialize ReplayState for new pets
                    initialize_replay_state,
                )
                    .chain()
                    .run_if(resource_exists::<PetModelState>),
            )
            .add_systems(
                Update,
                (
                    // Handle click events
                    handle_pet_click,
                    // Replay time management
                    advance_replay_time,
                    // Keyframe execution
                    execute_keyframes,
                    dispatch_actions,
                    // Action handlers
                    play_animation,
                    update_movement_tween,
                    update_scale_tween,
                    update_bounce_tween,
                    // Script completion check
                    check_script_completion,
                )
                    .chain()
                    .run_if(any_with_component::<ReplayState>),
            )
            .add_systems(
                PostUpdate,
                handle_script_transition.run_if(any_with_component::<ReplayState>),
            );
    }
}

/// Load builtin scripts into the script library
fn load_script_library(mut script_library: ResMut<ScriptLibrary>) {
    script_library.load_builtin();

    info!(
        "Loaded {} builtin scripts: {:?}",
        script_library.script_ids().len(),
        script_library.script_ids()
    );
}

/// Initialize ReplayState for pets that don't have one yet
fn initialize_replay_state(
    mut commands: Commands,
    pet_query: Query<Entity, (With<PetMarker>, Without<ReplayState>)>,
    pet_model_state: Res<PetModelState>,
) {
    // Only initialize after the model has been spawned
    if !pet_model_state.spawned {
        return;
    }

    for entity in pet_query.iter() {
        commands
            .entity(entity)
            .insert(ReplayState::new("idle".to_string()));
        info!("Initialized ReplayState for pet entity {:?}", entity);
    }
}
