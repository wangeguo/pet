//! Animation playback systems

use bevy::animation::AnimationPlayer;
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use tracing::{debug, info, warn};

use crate::components::{PetAnimationState, PetMarker, ReplayState};
use crate::events::PlayAnimationEvent;
use crate::resources::{AnimationMap, PetModelState};

/// Setup animation graph when GLTF scene is ready
pub fn setup_animation_graph(
    mut commands: Commands,
    pet_model_state: Res<PetModelState>,
    gltf_assets: Res<Assets<Gltf>>,
    mut animation_map: ResMut<AnimationMap>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_players: Query<Entity, (With<AnimationPlayer>, Without<AnimationGraphHandle>)>,
) {
    // Skip if already set up
    if animation_map.graph.is_some() {
        return;
    }

    let Some(ref gltf_handle) = pet_model_state.gltf_handle else {
        return;
    };

    let Some(gltf) = gltf_assets.get(gltf_handle) else {
        return;
    };

    // Find an AnimationPlayer that needs a graph
    let Some(player_entity) = animation_players.iter().next() else {
        return;
    };

    // Create animation graph
    let mut graph = AnimationGraph::new();

    // Add named animations from GLTF
    for (name, clip_handle) in gltf.named_animations.iter() {
        let node_index = graph.add_clip(clip_handle.clone(), 1.0, graph.root);
        animation_map
            .name_to_index
            .insert(name.to_string(), node_index);
        info!("Added animation '{}' to graph", name);
    }

    // If no named animations, use indexed animations
    if animation_map.name_to_index.is_empty() {
        for (index, clip_handle) in gltf.animations.iter().enumerate() {
            let name = format!("animation_{}", index);
            let node_index = graph.add_clip(clip_handle.clone(), 1.0, graph.root);
            animation_map.name_to_index.insert(name.clone(), node_index);
            info!("Added indexed animation '{}' to graph", name);
        }
    }

    if animation_map.name_to_index.is_empty() {
        warn!("No animations found in GLTF model");
        return;
    }

    let graph_handle = graphs.add(graph);
    animation_map.graph = Some(graph_handle.clone());

    // Attach graph to the player
    commands
        .entity(player_entity)
        .insert(AnimationGraphHandle(graph_handle));

    info!(
        "Animation graph setup complete with {} animations",
        animation_map.name_to_index.len()
    );
}

/// Link AnimationPlayer entity to PetAnimationState
#[allow(clippy::type_complexity)]
pub fn link_animation_player(
    mut commands: Commands,
    pet_query: Query<
        Entity,
        (
            With<PetMarker>,
            With<ReplayState>,
            Without<PetAnimationState>,
        ),
    >,
    children_query: Query<&Children>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
) {
    for pet_entity in pet_query.iter() {
        if let Some(player_entity) =
            find_animation_player_recursive(pet_entity, &children_query, &animation_players)
        {
            commands.entity(pet_entity).insert(PetAnimationState {
                current_animation: None,
                player_entity: Some(player_entity),
            });
            info!(
                "Linked AnimationPlayer {:?} to pet {:?}",
                player_entity, pet_entity
            );
        }
    }
}

/// Recursively find AnimationPlayer in entity hierarchy
fn find_animation_player_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    animation_players: &Query<Entity, With<AnimationPlayer>>,
) -> Option<Entity> {
    // Check if this entity has an AnimationPlayer
    if animation_players.contains(entity) {
        return Some(entity);
    }

    // Check children
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if let Some(found) =
                find_animation_player_recursive(child, children_query, animation_players)
            {
                return Some(found);
            }
        }
    }

    None
}

/// Handle animation playback requests
pub fn play_animation(
    mut events: MessageReader<PlayAnimationEvent>,
    animation_map: Res<AnimationMap>,
    mut pet_query: Query<&mut PetAnimationState>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for event in events.read() {
        let Some(node_index) = animation_map.name_to_index.get(&event.animation_name) else {
            if animation_map.name_to_index.is_empty() {
                debug!(
                    "Animation '{}' skipped: model has no animations",
                    event.animation_name
                );
            } else {
                warn!(
                    "Animation '{}' not found in animation map",
                    event.animation_name
                );
            }
            continue;
        };

        // Get the pet's animation state
        let Ok(mut pet_state) = pet_query.get_mut(event.entity) else {
            continue;
        };

        // Update current animation name
        pet_state.current_animation = Some(event.animation_name.clone());

        // Get the animation player
        let Some(player_entity) = pet_state.player_entity else {
            warn!("No AnimationPlayer linked to pet entity {:?}", event.entity);
            continue;
        };

        let Ok(mut player) = animation_players.get_mut(player_entity) else {
            warn!("AnimationPlayer not found on entity {:?}", player_entity);
            continue;
        };

        // Play the animation
        player.play(*node_index).repeat();

        info!("Playing animation '{}'", event.animation_name);
    }
}
