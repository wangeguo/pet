//! Bounce tween system

use bevy::prelude::*;

use crate::components::BounceTween;

/// Parabolic arc: goes up then comes back down (0 -> 1 -> 0)
fn bounce_arc(t: f32) -> f32 {
    4.0 * t * (1.0 - t)
}

/// Update bounce tweens
pub fn update_bounce_tween(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut BounceTween)>,
) {
    for (entity, mut transform, mut tween) in query.iter_mut() {
        tween.elapsed += time.delta_secs();

        let progress = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let arc = bounce_arc(progress);

        transform.translation.y = tween.base_y + tween.height * arc;

        // Remove component when tween is complete
        if progress >= 1.0 {
            transform.translation.y = tween.base_y;
            commands.entity(entity).remove::<BounceTween>();
        }
    }
}
