//! Interaction plugin - handles window dragging

use crate::resources::DragState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Plugin for handling user interactions
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Update, handle_drag);
    }
}

/// Handle window dragging
fn handle_drag(
    mut drag_state: ResMut<DragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    if mouse_button.just_pressed(MouseButton::Left)
        && let Some(cursor_pos) = window.cursor_position()
    {
        drag_state.is_dragging = true;
        drag_state.drag_start_cursor = Some(cursor_pos);
        drag_state.drag_start_window = match window.position {
            WindowPosition::At(pos) => Some(pos),
            _ => None,
        };
    }

    if drag_state.is_dragging
        && mouse_button.pressed(MouseButton::Left)
        && let (Some(start_cursor), Some(start_window)) =
            (drag_state.drag_start_cursor, drag_state.drag_start_window)
        && let Some(current_cursor) = window.cursor_position()
    {
        let delta = current_cursor - start_cursor;
        window.position =
            WindowPosition::At(start_window + IVec2::new(delta.x as i32, delta.y as i32));
    }

    if mouse_button.just_released(MouseButton::Left) {
        drag_state.is_dragging = false;
        drag_state.drag_start_cursor = None;
        drag_state.drag_start_window = None;
    }
}
