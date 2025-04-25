#![allow(unused_imports)] // Allow unused imports in this module definition file

pub mod text;
pub mod rectangle;
pub mod circle;
pub mod line; // Add line module

pub use circle::Circle;
pub use rectangle::Rectangle;
pub use text::Text;
pub use line::Line; // Re-export Line
