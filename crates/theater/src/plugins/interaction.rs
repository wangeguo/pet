//! Interaction plugin - handles window dragging and click detection

use crate::events::PetClickedEvent;
use crate::resources::DragState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Minimum distance in pixels to consider a mouse movement as a drag
const DRAG_THRESHOLD: f32 = 5.0;

/// Plugin for handling user interactions
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Update, handle_drag);
    }
}

/// Handle window dragging and click detection
fn handle_drag(
    mut drag_state: ResMut<DragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut click_events: MessageWriter<PetClickedEvent>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    // Mouse button pressed - record start position
    if mouse_button.just_pressed(MouseButton::Left)
        && let Some(cursor_pos) = window.cursor_position()
    {
        drag_state.drag_start_cursor = Some(cursor_pos);
        drag_state.drag_start_window = match window.position {
            WindowPosition::At(pos) => Some(pos),
            _ => None,
        };
        drag_state.is_dragging = false; // Not dragging yet, need to exceed threshold
    }

    // Mouse held - check for drag threshold and handle dragging
    if mouse_button.pressed(MouseButton::Left)
        && let (Some(start_cursor), Some(start_window)) =
            (drag_state.drag_start_cursor, drag_state.drag_start_window)
        && let Some(current_cursor) = window.cursor_position()
    {
        let delta = current_cursor - start_cursor;

        // Check if movement exceeds drag threshold
        if !drag_state.is_dragging && delta.length() > DRAG_THRESHOLD {
            drag_state.is_dragging = true;
        }

        // If dragging, update window position
        if drag_state.is_dragging {
            window.position =
                WindowPosition::At(start_window + IVec2::new(delta.x as i32, delta.y as i32));
        }
    }

    // Mouse released - check if it was a click or end of drag
    if mouse_button.just_released(MouseButton::Left) {
        // If we never exceeded the drag threshold, it's a click
        if !drag_state.is_dragging && drag_state.drag_start_cursor.is_some() {
            click_events.write(PetClickedEvent);
        }

        // Reset drag state
        drag_state.is_dragging = false;
        drag_state.drag_start_cursor = None;
        drag_state.drag_start_window = None;
    }
}
