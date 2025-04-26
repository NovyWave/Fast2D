// ... (imports) ...

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

// Ensure no duplicate definitions remain below this line
