// Shared types and utilities for Blade Browser Example

use serde::{Deserialize, Serialize};

// Example message types for frontend/backend communication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub webgpu_enabled: bool,
    pub canvas_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            webgpu_enabled: true,
            canvas_id: "blade-canvas".to_string(),
        }
    }
}