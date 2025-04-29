//! 2D object primitives for Fast2D.
//!
//! This module provides types for representing 2D graphical objects such as text, rectangles, circles, and lines.

mod text;
pub use text::{Text, FontWeight, Family};

mod rectangle;
pub use rectangle::Rectangle;

mod circle;
pub use circle::Circle;

mod line; 
pub use line::Line;

/// Represents a 2D object that can be rendered.
#[derive(Debug, Clone)]
pub enum Object2d {
    /// A text object.
    Text(Text),
    /// A rectangle object.
    Rectangle(Rectangle),
    /// A circle object.
    Circle(Circle),
    /// A line object.
    Line(Line),
}
