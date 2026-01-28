//! Scale tween system

use bevy::prelude::*;

use crate::components::ScaleTween;

/// Ease-out back function for bouncy scale effect
fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Update scale tweens
pub fn update_scale_tween(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ScaleTween)>,
) {
    for (entity, mut transform, mut tween) in query.iter_mut() {
        tween.elapsed += time.delta_secs();

        let progress = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased_progress = ease_out_back(progress);

        transform.scale = tween.start_scale.lerp(tween.target_scale, eased_progress);

        // Remove component when tween is complete
        if progress >= 1.0 {
            commands.entity(entity).remove::<ScaleTween>();
        }
    }
}
