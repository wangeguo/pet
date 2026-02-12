//! Rotation tween system

use bevy::prelude::*;

use crate::components::RotationTween;

/// Ease-in-out cubic function for smooth spin
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Update rotation tweens
pub fn update_rotation_tween(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut RotationTween)>,
) {
    for (entity, mut transform, mut tween) in query.iter_mut() {
        tween.elapsed += time.delta_secs();

        let progress = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased_progress = ease_in_out_cubic(progress);

        let current_angle = tween.total_angle * eased_progress;
        transform.rotation = tween.start_rotation * Quat::from_rotation_y(current_angle);

        // Remove component when tween is complete
        if progress >= 1.0 {
            commands.entity(entity).remove::<RotationTween>();
        }
    }
}
