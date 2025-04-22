use std::borrow::Cow;
use glyphon::{
    AttrsOwned, Color as GlyphonColor, Metrics, FamilyOwned,
    TextBounds, // Removed CacheKeyFlags, Style, Weight, Stretch from here
};
use glyphon::cosmic_text::CacheKeyFlags; // Correct import path for CacheKeyFlags
use wgpu::Color as WgpuColor; // Alias wgpu::Color
use crate::Object2d; // Import Object2d from the crate root

#[derive(Debug, Clone)] // Add derive for Debug and Clone if needed later
pub struct Text {
    text: Cow<'static, str>,
    left: f32,
    top: f32,
    font_size: f32,
    line_height: f32,
    color: WgpuColor,
    family: FamilyOwned,
    bounds_left: i32,
    bounds_top: i32,
    bounds_right: i32,
    bounds_bottom: i32,
}

impl Text {
    pub fn new() -> Self {
        Self {
            text: "".into(),
            left: 0.0,
            top: 0.0,
            font_size: 16.0,
            line_height: 19.2, // Example: 1.2 * font_size
            color: WgpuColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White
            family: FamilyOwned::SansSerif,
            bounds_left: 0,
            bounds_top: 0,
            bounds_right: i32::MAX,
            bounds_bottom: i32::MAX,
        }
    }

    pub fn position(mut self, left: i32, top: i32) -> Self {
        self.left = left as f32;
        self.top = top as f32;
        self
    }

    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.text = text.into();
        self
    }

    pub fn font_size(mut self, size: u32) -> Self {
        self.font_size = size as f32;
        // Optionally update line_height based on font_size if it's relative
        // self.line_height = self.font_size * 1.2;
        self
    }

    pub fn line_height(mut self, height: u32) -> Self {
        self.line_height = height as f32;
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self { // Changed a to f64
        self.color = WgpuColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a, // Assign f64 directly
        };
        self
    }

    pub fn family(mut self, family_name: impl Into<String>) -> Self {
        let name = family_name.into();
        // Basic mapping example
        self.family = match name.to_lowercase().as_str() {
            "monospace" | "firacode" => FamilyOwned::Monospace, // Treat FiraCode as Monospace here
            "sans-serif" => FamilyOwned::SansSerif,
            "serif" => FamilyOwned::Serif,
            "fantasy" => FamilyOwned::Fantasy,
            "cursive" => FamilyOwned::Cursive,
            _ => FamilyOwned::Name(name.into()), // Use .into() to convert String to SmolStr
        };
        self
    }

    pub fn bounds(mut self, left: i32, top: i32, right: i32, bottom: i32) -> Self {
        self.bounds_left = left;
        self.bounds_top = top;
        self.bounds_right = right;
        self.bounds_bottom = bottom;
        self
    }

    // --- Internal helpers accessible within the crate ---
    pub(crate) fn get_text(&self) -> &Cow<'static, str> {
        &self.text
    }

    pub(crate) fn get_left(&self) -> f32 {
        self.left
    }

     pub(crate) fn get_top(&self) -> f32 {
        self.top
    }

    pub fn get_attrs(&self) -> AttrsOwned {
        AttrsOwned::new(&glyphon::Attrs { // Borrow the struct literal
            color_opt: Some(self.get_glyphon_color()),
            family: self.family.as_family(), // Use as_family() to get Family<'_>
            stretch: Default::default(), // Use Default::default()
            style: Default::default(), // Use Default::default() instead of self.style
            weight: Default::default(), // Use Default::default() instead of self.weight
            metadata: 0,
            // Add missing fields with default values
            cache_key_flags: CacheKeyFlags::empty(), // Use empty() instead of default()
            font_features: Default::default(),
            letter_spacing_opt: Default::default(),
            metrics_opt: None, // Correct field name and initialize with None
        })
    }

    pub(crate) fn get_metrics(&self) -> Metrics {
        Metrics::new(self.font_size, self.line_height)
    }

    pub(crate) fn get_text_bounds(&self) -> TextBounds {
        TextBounds {
            left: self.bounds_left,
            top: self.bounds_top,
            right: self.bounds_right,
            bottom: self.bounds_bottom,
        }
    }

     pub(crate) fn get_glyphon_color(&self) -> GlyphonColor {
         GlyphonColor::rgba(
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8,
            (self.color.a * 255.0) as u8,
        )
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Object2d> for Text {
    fn into(self) -> Object2d {
        Object2d::Text(self)
    }
}
