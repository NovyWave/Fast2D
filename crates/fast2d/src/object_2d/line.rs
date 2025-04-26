use lyon::math::Point;
use wgpu::Color as WgpuColor;
use crate::Object2d;
use super::types::{Position, Color};

#[derive(Debug, Clone)]
pub struct Line {
    pub(crate) points: Vec<Position>,
    pub(crate) color: Color,
    pub(crate) width: f32,
}

impl Line {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            color: Color::default(),
            width: 1.0,
        }
    }

    pub fn points(mut self, points_data: &[(f32, f32)]) -> Self {
        self.points = points_data.iter().map(|(x, y)| Position { x: *x, y: *y }).collect();
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self {
        self.color = Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a,
        };
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Line> for Object2d {
    fn from(line: Line) -> Self {
        Object2d::Line(line)
    }
}
