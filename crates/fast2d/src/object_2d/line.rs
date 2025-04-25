use lyon::math::Point;
use wgpu::Color as WgpuColor;
use crate::Object2d;

#[derive(Debug, Clone)]
pub struct Line {
    pub(crate) points: Vec<Point>,
    pub(crate) color: WgpuColor,
    pub(crate) width: f32, // Renamed from thickness
}

impl Line {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            color: WgpuColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // Default white
            width: 1.0, // Renamed from thickness, default 1 pixel wide
        }
    }

    /// Sets the points defining the line segments.
    /// Expects points in the format [x1, y1, x2, y2, x3, y3, ...].
    pub fn points(mut self, points: &[f32]) -> Self {
        self.points = points
            .chunks_exact(2)
            .map(|chunk| Point::new(chunk[0], chunk[1]))
            .collect();
        self
    }

    /// Sets the color of the line (RGBA).
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self {
        self.color = WgpuColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a,
        };
        self
    }

    /// Sets the width of the line.
    pub fn width(mut self, width: f32) -> Self { // Renamed from thickness
        self.width = width.max(0.1); // Use width field, ensure minimum width
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
