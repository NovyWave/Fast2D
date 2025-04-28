// ... (imports) ...
use std::borrow::Cow;

/// Shared, backend-agnostic types for 2D objects.

// Simple Color struct using f32 for components (0.0 to 1.0 range)
// Consolidate derives here
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    #[cfg(feature = "canvas")]
    pub(crate) fn to_canvas_rgba(&self) -> String {
        format!("rgba({}, {}, {}, {})",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            self.a
        )
    }

    // --- sRGB / Linear Conversion ---
    // Convert sRGB color component to linear
    #[cfg(not(feature = "canvas"))]
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    // Convert this Color (assumed sRGB) to linear RGBA values for vertex buffers
    #[cfg(not(feature = "canvas"))]
    pub fn to_linear(&self) -> [f32; 4] {
        [
            Self::srgb_to_linear(self.r),
            Self::srgb_to_linear(self.g),
            Self::srgb_to_linear(self.b),
            self.a, // Alpha is linear
        ]
    }

     // Convert this Color (assumed sRGB) to Glyphon's Color type (u8)
     // Always pass direct sRGB f32 values scaled to u8 range.
     #[cfg(not(feature = "canvas"))]
     pub fn to_glyphon_color(&self) -> GlyphonColor {
         GlyphonColor::rgba(
             (self.r * 255.0).clamp(0.0, 255.0) as u8,
             (self.g * 255.0).clamp(0.0, 255.0) as u8,
             (self.b * 255.0).clamp(0.0, 255.0) as u8,
             (self.a * 255.0).clamp(0.0, 255.0) as u8,
         )
     }

    // Convert this Color (assumed sRGB) to LINEAR values scaled to u8 for Glyphon.
    // Used when target is sRGB and ColorMode::Accurate is set.
    #[cfg(not(feature = "canvas"))]
    pub fn to_glyphon_color_linear(&self) -> GlyphonColor {
        let linear = self.to_linear();
        GlyphonColor::rgba(
            (linear[0] * 255.0).clamp(0.0, 255.0) as u8,
            (linear[1] * 255.0).clamp(0.0, 255.0) as u8,
            (linear[2] * 255.0).clamp(0.0, 255.0) as u8,
            (linear[3] * 255.0).clamp(0.0, 255.0) as u8, // Alpha
        )
    }
}

// Simple Point struct
// Consolidate derives here
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
     pub fn new(x: f32, y: f32) -> Self {
         Self { x, y }
     }
}

// Simple Size struct
// Consolidate derives here
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
     pub fn new(width: f32, height: f32) -> Self {
         Self { width, height }
     }
}

// Simple BorderRadii struct
// Consolidate derives here
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl BorderRadii {
    pub fn are_all_zero(&self) -> bool {
        self.top_left == 0.0 && self.top_right == 0.0 && self.bottom_left == 0.0 && self.bottom_right == 0.0
    }
}

// Conditionally export FamilyOwned from glyphon
#[cfg(not(feature = "canvas"))]
pub use glyphon::FamilyOwned;

#[cfg(not(feature = "canvas"))]
use glyphon::Color as GlyphonColor; // Conditionally import GlyphonColor

/// Unified font family enum for all backends.
#[derive(Debug, Clone, PartialEq)]
pub enum Family {
    Name(String),
    SansSerif,
    Serif,
    Monospace,
    Cursive,
    Fantasy,
}

impl From<&Family> for String {
    fn from(family: &Family) -> Self {
        match family {
            Family::Name(name) => name.clone(),
            Family::SansSerif => "sans-serif".to_owned(),
            Family::Serif => "serif".to_owned(),
            Family::Monospace => "monospace".to_owned(),
            Family::Cursive => "cursive".to_owned(),
            Family::Fantasy => "fantasy".to_owned(),
        }
    }
}

impl From<Family> for String {
    fn from(family: Family) -> Self {
        String::from(&family)
    }
}

impl Family {
    /// Helper to create a Family::Name from a string literal or String.
    pub fn name<S: Into<String>>(name: S) -> Self {
        Family::Name(name.into())
    }
}

#[cfg(not(feature = "canvas"))]
impl From<&crate::Family> for FamilyOwned {
    fn from(family: &crate::Family) -> Self {
        match family {
            crate::Family::Name(name) => FamilyOwned::Name(name.clone().into()),
            crate::Family::SansSerif => FamilyOwned::SansSerif,
            crate::Family::Serif => FamilyOwned::Serif,
            crate::Family::Monospace => FamilyOwned::Monospace,
            crate::Family::Cursive => FamilyOwned::Cursive,
            crate::Family::Fantasy => FamilyOwned::Fantasy,
        }
    }
}

#[cfg(not(feature = "canvas"))]
impl From<crate::Family> for FamilyOwned {
    fn from(family: crate::Family) -> Self {
        Self::from(&family)
    }
}

// Ensure no duplicate definitions remain below this line
