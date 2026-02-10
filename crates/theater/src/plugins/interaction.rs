//! Interaction plugin - handles window dragging and click detection

use crate::events::PetClickedEvent;
use crate::resources::DragState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{UpdateMode, WinitSettings};

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

/// Low-power update interval for idle state
const LOW_POWER_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
/// High-frequency update interval for dragging
const DRAG_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(8);

/// Convert window-local cursor position to screen coordinates
fn cursor_to_screen(window: &Window) -> Option<IVec2> {
    let cursor_pos = window.cursor_position()?;
    let window_pos = match window.position {
        WindowPosition::At(pos) => pos,
        _ => return None,
    };
    Some(window_pos + IVec2::new(cursor_pos.x as i32, cursor_pos.y as i32))
}

/// Handle window dragging and click detection
fn handle_drag(
    mut drag_state: ResMut<DragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut click_events: MessageWriter<PetClickedEvent>,
    mut winit_settings: ResMut<WinitSettings>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    // Mouse button pressed - record start position in screen coordinates
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(screen_cursor) = cursor_to_screen(&window) {
            drag_state.drag_start_screen_cursor = Some(screen_cursor);
            drag_state.drag_start_window = match window.position {
                WindowPosition::At(pos) => Some(pos),
                _ => None,
            };
            drag_state.is_dragging = false;
        }
    }

    // Mouse held - check for drag threshold and handle dragging
    if mouse_button.pressed(MouseButton::Left)
        && let (Some(start_screen_cursor), Some(start_window)) = (
            drag_state.drag_start_screen_cursor,
            drag_state.drag_start_window,
        )
        && let Some(current_screen_cursor) = cursor_to_screen(&window)
    {
        let delta = current_screen_cursor - start_screen_cursor;

        // Check if movement exceeds drag threshold
        if !drag_state.is_dragging && (delta.x.abs() + delta.y.abs()) > DRAG_THRESHOLD as i32 {
            drag_state.is_dragging = true;
            // Switch to high-frequency updates for smooth dragging
            winit_settings.focused_mode = UpdateMode::reactive_low_power(DRAG_UPDATE_INTERVAL);
            winit_settings.unfocused_mode = UpdateMode::reactive_low_power(DRAG_UPDATE_INTERVAL);
        }

        // If dragging, update window position directly
        if drag_state.is_dragging {
            window.position = WindowPosition::At(start_window + delta);
        }
    }

    // Mouse released - check if it was a click or end of drag
    if mouse_button.just_released(MouseButton::Left) {
        if !drag_state.is_dragging && drag_state.drag_start_screen_cursor.is_some() {
            click_events.write(PetClickedEvent);
        }

        if drag_state.is_dragging {
            winit_settings.focused_mode = UpdateMode::reactive_low_power(LOW_POWER_INTERVAL);
            winit_settings.unfocused_mode = UpdateMode::reactive_low_power(LOW_POWER_INTERVAL);
        }

        drag_state.is_dragging = false;
        drag_state.drag_start_screen_cursor = None;
        drag_state.drag_start_window = None;
    }
}
