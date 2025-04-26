// ... (imports) ...

#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub width: f32, // Change to f32
    pub height: f32, // Change to f32
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub x: f32, // Change to f32
    pub y: f32, // Change to f32
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BorderRadii {
    pub top_left: f32,     // Change to f32
    pub top_right: f32,    // Change to f32
    pub bottom_left: f32,  // Change to f32
    pub bottom_right: f32, // Change to f32
}

// Color struct likely remains f64 or f32 for components 0.0-1.0 internally
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f64, // Keep as is, or consider f32 if preferred
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } // Default to opaque black
    }
}

// ... (FamilyOwned remains the same) ...
