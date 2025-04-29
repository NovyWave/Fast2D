mod register_fonts;
pub use register_fonts::register_fonts;

mod canvas_wrapper;
pub use canvas_wrapper::CanvasWrapper;

mod color;
pub use color::Color;

mod draw;
pub use draw::draw;

mod graphics;
pub use graphics::{resize_graphics, create_graphics};

use {
    lyon::path::{Path, Winding},
    lyon::path::builder::BorderRadii as LyonBorderRadii,
    lyon::math::Box2D,
    lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder, StrokeTessellator, StrokeOptions, StrokeVertex, LineCap, LineJoin},
    wgpu::{Device, Queue, Surface, SurfaceConfiguration, Texture, BindGroupLayout, BindGroup, Buffer as WgpuBuffer, TextureViewDescriptor},
    wgpu::util::DeviceExt,
    glyphon::{Shaping, Buffer as GlyphonBuffer, SwashCache, TextArea, Attrs, TextBounds, Metrics, Family as GlyphonFamily},
    bytemuck,
    web_sys::console,
    web_sys::wasm_bindgen::{JsValue, UnwrapThrowExt},
    lyon::math::point,
};

pub static FONT_SYSTEM: std::sync::OnceLock<std::sync::Mutex<glyphon::FontSystem>> = std::sync::OnceLock::new();
pub const MSAA_SAMPLE_COUNT: u32 = 4;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CanvasUniforms {
    pub width: f32,
    pub height: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl ColoredVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[allow(dead_code)]
pub struct Graphics {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub is_srgb: bool,
    pub msaa_texture: Texture,
    pub swash_cache: SwashCache,
    pub viewport: glyphon::Viewport,
    pub atlas: glyphon::TextAtlas,
    pub text_renderer: glyphon::TextRenderer,
    pub uniform_buffer: WgpuBuffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
    pub rect_pipeline: wgpu::RenderPipeline,
}
