use lyon::math::Point;
use wgpu::Color as WgpuColor;
use crate::Object2d;

#[derive(Debug, Clone)]
pub struct Circle {
    pub(crate) center: Point,
    pub(crate) radius: f32,
    pub(crate) color: WgpuColor,
    pub(crate) border_width: Option<f32>,
    pub(crate) border_color: Option<WgpuColor>,
}

impl Circle {
    pub fn new() -> Self {
        Self {
            center: Point::new(0.0, 0.0),
            radius: 50.0, // Default radius
            color: WgpuColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, // Default transparent
            border_width: None,
            border_color: None,
        }
    }

    pub fn center(mut self, x: i32, y: i32) -> Self {
        self.center = Point::new(x as f32, y as f32);
        self
    }

    pub fn radius(mut self, radius: u32) -> Self {
        self.radius = radius as f32;
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self {
        self.color = WgpuColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a,
        };
        self
    }

    pub fn inner_border(mut self, width: u32, r: u8, g: u8, b: u8, a: f64) -> Self {
        self.border_width = Some(width as f32);
        self.border_color = Some(WgpuColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a,
        });
        self
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Object2d> for Circle {
    fn into(self) -> Object2d {
        Object2d::Circle(self)
    }
}
