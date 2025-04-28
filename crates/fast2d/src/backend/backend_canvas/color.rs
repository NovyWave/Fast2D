#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color { r: 255, g: 255, b: 255, a: 1.0 }
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a: a.clamp(0.0, 1.0) }
    }

    pub(crate) fn to_canvas_rgba(&self) -> String {
        let Self { r, g, b, a } = self;
        format!("rgba({r}, {g}, {b}, {a})")
    }
}
