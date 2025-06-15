use crate::layout::{LayoutManager, Viewport};

/// Input event router
pub struct InputRouter {
    // Future: Handle mouse and keyboard events
    // Route events to the correct canvas region based on layout
}

impl InputRouter {
    /// Create a new input router
    pub fn new() -> Self {
        Self {}
    }
    
    /// Route mouse event to the appropriate canvas region
    pub fn route_mouse_event(&self, x: f32, y: f32, layout_manager: &LayoutManager) -> Option<usize> {
        let panel_layouts = layout_manager.calculate_panel_positions();
        
        for (index, layout) in panel_layouts.iter().enumerate() {
            if layout.visible && layout.viewport.contains_point(x, y) {
                return Some(index);
            }
        }
        
        None
    }
    
    /// Handle scroll event
    pub fn handle_scroll_event(&self, delta_y: f32, layout_manager: &mut LayoutManager) {
        layout_manager.handle_scroll(delta_y);
    }
}

/// Mouse event data
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub x: f32,
    pub y: f32,
    pub button: MouseButton,
    pub event_type: MouseEventType,
}

/// Mouse button types
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse event types
#[derive(Debug, Clone, Copy)]
pub enum MouseEventType {
    Press,
    Release,
    Move,
}