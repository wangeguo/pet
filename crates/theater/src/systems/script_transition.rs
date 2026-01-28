//! Script transition and completion systems

use bevy::prelude::*;
use tracing::{info, warn};

use crate::components::ReplayState;
use crate::events::{PetClickedEvent, ScriptCompletedEvent, SwitchScriptEvent};
use crate::resources::ScriptLibrary;

/// Check if scripts have completed and emit completion events
pub fn check_script_completion(
    mut query: Query<(Entity, &mut ReplayState)>,
    script_library: Res<ScriptLibrary>,
    mut completed_events: MessageWriter<ScriptCompletedEvent>,
) {
    for (entity, mut state) in query.iter_mut() {
        if state.completed {
            continue;
        }

        let Some(script) = script_library.get(&state.script_id) else {
            continue;
        };

        // Check if script is completed
        let is_completed = match script.duration {
            Some(duration) => state.elapsed_time >= duration,
            None => {
                // For looping scripts, check if all keyframes have been executed
                state.next_keyframe_index >= script.keyframes.len()
            }
        };

        if is_completed {
            state.completed = true;
            completed_events.write(ScriptCompletedEvent {
                entity,
                script_id: state.script_id.clone(),
                next_script: script.next.clone(),
            });
        }
    }
}

/// Handle script transitions from completion events and switch requests
pub fn handle_script_transition(
    mut completed_events: MessageReader<ScriptCompletedEvent>,
    mut switch_events: MessageReader<SwitchScriptEvent>,
    mut query: Query<(Entity, &mut ReplayState)>,
    script_library: Res<ScriptLibrary>,
) {
    // Handle script completions
    for event in completed_events.read() {
        if let Some(ref next_script_id) = event.next_script {
            // Switch to next script if it exists
            if script_library.get(next_script_id).is_some() {
                for (entity, mut state) in query.iter_mut() {
                    if entity == event.entity {
                        state.switch_to(next_script_id.clone());
                        info!(
                            "Script '{}' completed, switching to '{}'",
                            event.script_id, next_script_id
                        );
                        break;
                    }
                }
            }
        } else {
            // No next script specified - for looping scripts, restart
            for (entity, mut state) in query.iter_mut() {
                if entity == event.entity {
                    if let Some(script) = script_library.get(&state.script_id)
                        && script.duration.is_none()
                    {
                        // Looping script - restart
                        let current_id = state.script_id.clone();
                        state.switch_to(current_id.clone());
                        info!("Looping script '{}' restarted", current_id);
                    }
                    break;
                }
            }
        }
    }

    // Handle manual switch requests
    for event in switch_events.read() {
        if script_library.get(&event.script_id).is_none() {
            warn!("Script '{}' not found in library", event.script_id);
            continue;
        }

        for (_entity, mut state) in query.iter_mut() {
            // Check if current script can be interrupted
            if !event.force
                && let Some(current_script) = script_library.get(&state.script_id)
                && !current_script.interruptible
                && !state.completed
            {
                info!(
                    "Cannot interrupt script '{}' (not interruptible)",
                    state.script_id
                );
                continue;
            }

            state.switch_to(event.script_id.clone());
            info!("Switched to script '{}'", event.script_id);
        }
    }
}

/// Handle pet click events by switching to the happy script
pub fn handle_pet_click(
    mut click_events: MessageReader<PetClickedEvent>,
    mut switch_events: MessageWriter<SwitchScriptEvent>,
) {
    for _event in click_events.read() {
        switch_events.write(SwitchScriptEvent {
            script_id: "happy".to_string(),
            force: false,
        });
    }
}
