cfg_if::cfg_if! {
    if #[cfg(any(feature = "webgl", feature = "webgpu"))] {
        mod backend_wgpu;
        pub use backend_wgpu::*;
    } else if #[cfg(feature = "canvas")] {
        mod backend_canvas;
        pub use backend_canvas::*;
    }
}

// Simple Point struct
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// Simple Size struct
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

// Simple BorderRadii struct
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}
