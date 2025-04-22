use wgpu::Color as WgpuColor;
use crate::Object2d; // Import Object2d from the crate root
use lyon::math::{Point, Size};
use lyon::path::builder::BorderRadii;

#[derive(Debug, Clone)] // Add derive for Debug and Clone
pub struct Rectangle {
    pub position: Point, // Made public for access in draw loop
    pub size: Size,      // Made public
    pub color: WgpuColor,// Made public
    pub border_radii: BorderRadii, // Made public
    // Add other fields as needed, e.g., border width, border color
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
            position: Point::new(0.0, 0.0),
            size: Size::new(10.0, 10.0), // Default size
            color: WgpuColor { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }, // Default gray
            border_radii: BorderRadii::default(), // No rounding by default
        }
    }

    pub fn position(mut self, x: u32, y: u32) -> Self {
        self.position = Point::new(x as f32, y as f32);
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Size::new(width as f32, height as f32);
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

    pub fn rounded_corners(mut self, top_left: u32, top_right: u32, bottom_right: u32, bottom_left: u32) -> Self {
        self.border_radii = BorderRadii {
            top_left: top_left as f32,
            top_right: top_right as f32,
            bottom_left: bottom_left as f32,
            bottom_right: bottom_right as f32,
        };
        self
    }

    // Add other builder methods as needed
}

impl Default for Rectangle {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Object2d> for Rectangle {
    fn into(self) -> Object2d {
        Object2d::Rectangle(self)
    }
}
