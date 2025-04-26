use std::borrow::Cow;
use glyphon::{FamilyOwned, Metrics, TextBounds};
// use crate::object_2d::Color; // Incorrect import
use wgpu::Color as WgpuColor; // Correct import

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) text: Cow<'static, str>,
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) font_size: f32,
    pub(crate) line_height_multiplier: f32, // Replace line_height with a multiplier
    pub(crate) color: WgpuColor, // Use wgpu::Color
    pub(crate) family: FamilyOwned,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: Cow::Borrowed(""),
            left: 0.0,
            top: 0.0,
            font_size: 16.0,
            line_height_multiplier: 1.0, // Set default multiplier
            // Use wgpu::Color for default
            color: WgpuColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // Default to white (using f64 for wgpu::Color)
            family: FamilyOwned::SansSerif,
            width: f32::MAX,
            height: f32::MAX,
        }
    }
}

impl Text {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.text = text.into();
        self
    }

    pub fn position(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    // Update line_height method to set the multiplier
    pub fn line_height(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier.max(0.0); // Ensure non-negative
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self { // Keep a as f64 for wgpu::Color
        // Create wgpu::Color
        self.color = WgpuColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a,
        };
        self
    }

    pub fn family(mut self, family: FamilyOwned) -> Self {
        self.family = family;
        self
    }

    // Set the size (width and height) of the text area
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    // Removed unused pub(crate) methods:
    // - get_text
    // - get_left
    // - get_top
    // - get_metrics
    // - get_text_bounds
}

impl From<Text> for crate::Object2d {
    fn from(text: Text) -> Self {
        crate::Object2d::Text(text)
    }
}
