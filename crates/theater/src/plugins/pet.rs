//! Pet plugin - handles model loading, camera, and lighting

use crate::components::PetMarker;
use crate::resources::{PetModelState, TheaterConfig};
use bevy::prelude::*;
use std::path::PathBuf;

/// Default model filename in assets/pets directory
const DEFAULT_MODEL_NAME: &str = "duck.glb";

/// Plugin for pet-related functionality
pub struct PetPlugin;

impl Plugin for PetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PetModelState>()
            .add_systems(Startup, setup_scene)
            .add_systems(Update, spawn_gltf_scene)
            // Run camera removal in PostUpdate to catch newly spawned cameras from scenes
            .add_systems(PostUpdate, remove_gltf_cameras);
    }
}

/// Marker for the main camera (to distinguish from GLTF-embedded cameras)
#[derive(Component)]
struct MainCamera;

/// Marker for placeholder mesh shown while model loads
#[derive(Component)]
struct PlaceholderPet;

/// Setup the scene with camera and lighting
fn setup_scene(
    mut commands: Commands,
    config: Res<TheaterConfig>,
    asset_server: Res<AssetServer>,
    mut pet_state: ResMut<PetModelState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera - side view, pet facing left
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, -5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        MainCamera,
    ));

    // Ambient light base
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..default()
    });

    // Main light from front-top
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Fill light from below to reduce bottom shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, -3.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Placeholder cube while model loads
    // Note: PlaceholderPet does NOT have PetMarker to avoid ReplayState initialization
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.5, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.25, 0.0),
        PlaceholderPet,
    ));

    // Determine model path
    let model_path = config
        .model_path
        .as_ref()
        .filter(|p| p.exists())
        .cloned()
        .or_else(default_model_path);

    // Load GLTF model
    if let Some(path) = model_path {
        let gltf_handle: Handle<Gltf> = asset_server.load(path.to_string_lossy().to_string());
        pet_state.gltf_handle = Some(gltf_handle);
    }
}

/// Spawn the GLTF scene once loaded
fn spawn_gltf_scene(
    mut commands: Commands,
    mut pet_state: ResMut<PetModelState>,
    gltf_assets: Res<Assets<Gltf>>,
    placeholder_query: Query<Entity, With<PlaceholderPet>>,
) {
    if pet_state.spawned {
        return;
    }

    let Some(ref gltf_handle) = pet_state.gltf_handle else {
        return;
    };

    let Some(gltf) = gltf_assets.get(gltf_handle) else {
        return;
    };

    // Remove placeholder
    for entity in placeholder_query.iter() {
        commands.entity(entity).despawn();
        tracing::info!("Despawned placeholder entity {:?}", entity);
    }

    // Spawn the scene
    if let Some(scene_handle) = gltf
        .default_scene
        .clone()
        .or_else(|| gltf.scenes.first().cloned())
    {
        let pet_entity = commands
            .spawn((
                SceneRoot(scene_handle),
                Transform::from_scale(Vec3::splat(1.0)),
                PetMarker,
            ))
            .id();
        pet_state.spawned = true;
        tracing::info!("Spawned GLTF scene as pet entity {:?}", pet_entity);
    }
}

/// Remove cameras embedded in GLTF scenes to avoid conflicts
fn remove_gltf_cameras(
    mut commands: Commands,
    camera_query: Query<Entity, (With<Camera3d>, Without<MainCamera>)>,
) {
    for entity in camera_query.iter() {
        commands.entity(entity).despawn();
        tracing::info!("Removed GLTF camera entity {:?}", entity);
    }
}

/// Get the default model path
fn default_model_path() -> Option<PathBuf> {
    Some(PathBuf::from(format!("pets/{}", DEFAULT_MODEL_NAME)))
}
