use anyhow::Result;
use tao::window::Window;

/// Window management utilities
pub struct WindowManager {
    window: Window,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new(window: Window) -> Self {
        Self { window }
    }
    
    /// Get window inner size
    pub fn inner_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }
    
    /// Request redraw
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
    
    /// Set window title
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }
}