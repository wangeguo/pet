//! Keyframe execution systems

use bevy::prelude::*;
use common::script::Action;
use rand::prelude::IndexedRandom;

use crate::components::{BounceTween, MovementTween, ReplayState, ScaleTween};
use crate::events::{ExecuteActionEvent, PlayAnimationEvent, SwitchScriptEvent};
use crate::resources::ScriptLibrary;

/// Advance the replay time for all entities with ReplayState
pub fn advance_replay_time(time: Res<Time>, mut query: Query<&mut ReplayState>) {
    let delta = time.delta_secs();

    for mut state in query.iter_mut() {
        if !state.completed {
            state.elapsed_time += delta;
        }
    }
}

/// Check and execute keyframes that are due
pub fn execute_keyframes(
    mut query: Query<(Entity, &mut ReplayState)>,
    script_library: Res<ScriptLibrary>,
    mut action_events: MessageWriter<ExecuteActionEvent>,
) {
    for (entity, mut state) in query.iter_mut() {
        let Some(script) = script_library.get(&state.script_id) else {
            continue;
        };

        // Execute all keyframes that are due
        while state.next_keyframe_index < script.keyframes.len() {
            let keyframe = &script.keyframes[state.next_keyframe_index];

            if keyframe.time <= state.elapsed_time {
                action_events.write(ExecuteActionEvent {
                    entity,
                    action: keyframe.action.clone(),
                });
                state.next_keyframe_index += 1;
            } else {
                break;
            }
        }
    }
}

/// Dispatch actions to their respective handlers
pub fn dispatch_actions(
    mut action_events: MessageReader<ExecuteActionEvent>,
    mut play_anim_events: MessageWriter<PlayAnimationEvent>,
    mut switch_script_events: MessageWriter<SwitchScriptEvent>,
    mut commands: Commands,
    query: Query<&Transform>,
) {
    for event in action_events.read() {
        match &event.action {
            Action::PlayAnimation { name } => {
                play_anim_events.write(PlayAnimationEvent {
                    entity: event.entity,
                    animation_name: name.clone(),
                });
            }

            Action::MoveTo { x, y } => {
                if let Ok(transform) = query.get(event.entity) {
                    // Map script coordinates to 3D space:
                    // x -> x (left/right)
                    // y -> z (forward/backward in Bevy's coordinate system)
                    commands.entity(event.entity).insert(MovementTween {
                        start_position: transform.translation,
                        target_position: Vec3::new(*x, transform.translation.y, *y),
                        duration: 1.0, // Default 1 second for movement
                        elapsed: 0.0,
                    });
                }
            }

            Action::Scale { factor } => {
                if let Ok(transform) = query.get(event.entity) {
                    commands.entity(event.entity).insert(ScaleTween {
                        start_scale: transform.scale,
                        target_scale: Vec3::splat(*factor),
                        duration: 0.3, // Quick scale transition
                        elapsed: 0.0,
                    });
                }
            }

            Action::Wait { duration: _ } => {
                // Wait is handled implicitly by the elapsed time check
            }

            Action::Random { scripts } => {
                if let Some(selected) = scripts.choose(&mut rand::rng()) {
                    switch_script_events.write(SwitchScriptEvent {
                        script_id: selected.clone(),
                        force: false,
                    });
                }
            }

            Action::Bounce { height } => {
                if let Ok(transform) = query.get(event.entity) {
                    commands.entity(event.entity).insert(BounceTween {
                        base_y: transform.translation.y,
                        height: *height,
                        duration: 0.4,
                        elapsed: 0.0,
                    });
                }
            }

            Action::SetExpression { expression: _ } => {
                // Not implemented in Phase 3
            }
        }
    }
}
