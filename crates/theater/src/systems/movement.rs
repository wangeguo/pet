//! Movement tween system

use bevy::prelude::*;

use crate::components::MovementTween;

/// Ease-out cubic function for smooth deceleration
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Update movement tweens
pub fn update_movement_tween(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MovementTween)>,
) {
    for (entity, mut transform, mut tween) in query.iter_mut() {
        tween.elapsed += time.delta_secs();

        let progress = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased_progress = ease_out_cubic(progress);

        transform.translation = tween
            .start_position
            .lerp(tween.target_position, eased_progress);

        // Remove component when tween is complete
        if progress >= 1.0 {
            commands.entity(entity).remove::<MovementTween>();
        }
    }
}
