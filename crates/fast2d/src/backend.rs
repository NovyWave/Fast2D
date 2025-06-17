use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "native")] {
        mod backend_wgpu_native;
        pub use backend_wgpu_native::*;
    } else if #[cfg(feature = "webgpu-blade")] {
        mod backend_blade;
        pub use backend_blade::*;
    } else if #[cfg(any(feature = "webgl", feature = "webgpu"))] {
        mod backend_wgpu;
        pub use backend_wgpu::*;
    } else if #[cfg(feature = "canvas")] {
        mod backend_canvas;
        pub use backend_canvas::*;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RoundedCorners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

/// Errors that can happen when registering fonts with [`register_fonts`] function.
#[derive(Debug)]
pub enum RegisterFontsError {
    NoFontsProvided,
    NoWindow,
    NoDocument,
    FontParseFailed,
    FontFaceError(String),
    AddFontError(String),
    NoValidFontLoaded,
}

impl std::fmt::Display for RegisterFontsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFontsProvided => write!(f, "No fonts provided"),
            Self::NoWindow => write!(f, "No window available"),
            Self::NoDocument => write!(f, "No document available"),
            Self::FontParseFailed => write!(f, "Failed to parse font data"),
            Self::FontFaceError(error) => write!(f, "FontFace error: {error}"),
            Self::AddFontError(error) => write!(f, "Add font error: {error}"),
            Self::NoValidFontLoaded => write!(f, "No valid font loaded"),
        }
    }
}

impl std::error::Error for RegisterFontsError {}
