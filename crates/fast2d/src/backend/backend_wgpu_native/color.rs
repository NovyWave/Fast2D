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

    pub(crate) fn to_linear(&self) -> [f32; 4] {
        fn srgb_to_linear(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        [
            srgb_to_linear(self.r as f32 / 255.0),
            srgb_to_linear(self.g as f32 / 255.0),
            srgb_to_linear(self.b as f32 / 255.0),
            self.a,
        ]
    }

    pub(crate) fn to_glyphon_color(&self) -> glyphon::Color {
        glyphon::Color::rgba(
            self.r,
            self.g,
            self.b,
            (self.a * 255.0).clamp(0.0, 255.0) as u8,
        )
    }
}
