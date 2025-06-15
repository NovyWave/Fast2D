/// Layout manager handling scrollable panel positioning
pub struct LayoutManager {
    scroll_offset: f32,
    panel_height: f32,
    panel_spacing: f32,
    panel_max_width: f32,
    padding: f32,
    window_width: f32,
    window_height: f32,
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new(window_width: f32, window_height: f32) -> Self {
        Self {
            scroll_offset: 0.0,
            panel_height: 350.0,  // Same as tauri_example
            panel_spacing: 10.0,  // Same as tauri_example
            panel_max_width: 650.0, // Same as tauri_example
            padding: 10.0,        // Same as tauri_example
            window_width,
            window_height,
        }
    }
    
    /// Update window size
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }
    
    /// Calculate positions for all panels
    pub fn calculate_panel_positions(&self) -> Vec<PanelLayout> {
        let mut layouts = Vec::new();
        
        // Calculate panel width (centered, max 650px)
        let panel_width = self.panel_max_width.min(self.window_width - 2.0 * self.padding);
        let panel_x = (self.window_width - panel_width) / 2.0;
        
        for i in 0..3 {
            let panel_y = self.padding + i as f32 * (self.panel_height + self.panel_spacing) - self.scroll_offset;
            
            // Check if panel is visible
            let visible = panel_y + self.panel_height > 0.0 && panel_y < self.window_height;
            
            layouts.push(PanelLayout {
                viewport: Viewport {
                    x: panel_x,
                    y: panel_y,
                    width: panel_width,
                    height: self.panel_height,
                },
                visible,
            });
        }
        
        layouts
    }
    
    /// Handle scroll input
    pub fn handle_scroll(&mut self, delta_y: f32) {
        // Calculate total content height
        let total_content_height = 3.0 * self.panel_height + 2.0 * self.panel_spacing + 2.0 * self.padding;
        
        // Update scroll offset
        self.scroll_offset += delta_y;
        
        // Clamp scroll offset
        self.scroll_offset = self.scroll_offset.max(0.0);
        
        if total_content_height > self.window_height {
            let max_scroll = total_content_height - self.window_height;
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        } else {
            self.scroll_offset = 0.0;
        }
    }
    
    /// Get current scroll offset
    pub fn scroll_offset(&self) -> f32 {
        self.scroll_offset
    }
}

/// Layout information for a single panel
#[derive(Debug, Clone)]
pub struct PanelLayout {
    pub viewport: Viewport,
    pub visible: bool,
}

/// Viewport representing a rectangular region
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    /// Check if a point is inside this viewport
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
    
    /// Convert global coordinates to local coordinates
    pub fn to_local(&self, x: f32, y: f32) -> (f32, f32) {
        (x - self.x, y - self.y)
    }
}