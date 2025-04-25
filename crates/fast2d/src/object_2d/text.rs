use std::borrow::Cow;
use glyphon::{FamilyOwned, Metrics, TextBounds};
// use crate::object_2d::Color; // Incorrect import
use wgpu::Color as WgpuColor; // Correct import

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) text: Cow<'static, str>,
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) font_size: u32,
    pub(crate) line_height: u32,
    pub(crate) color: WgpuColor, // Use wgpu::Color
    pub(crate) family: FamilyOwned,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: Cow::Borrowed(""),
            left: 0.0,
            top: 0.0,
            font_size: 16,
            line_height: 20,
            // Use wgpu::Color for default
            color: WgpuColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // Default to white (using f64 for wgpu::Color)
            family: FamilyOwned::SansSerif,
            width: 100,
            height: 50,
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

    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = left as f32;
        self.top = top as f32;
        self
    }

    pub fn font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        // Update default line height if it hasn't been explicitly set or is smaller
        if self.line_height < size {
            self.line_height = (size as f32 * 1.2) as u32; // Common default ratio
        }
        self
    }

    pub fn line_height(mut self, height: u32) -> Self {
        self.line_height = height;
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
    pub fn size(mut self, width: u32, height: u32) -> Self {
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
