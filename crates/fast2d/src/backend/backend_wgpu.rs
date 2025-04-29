mod register_fonts;
pub use register_fonts::register_fonts;

mod canvas_wrapper;
pub use canvas_wrapper::CanvasWrapper;

mod color;
pub use color::Color;

mod draw;
pub use draw::draw;

use {
    lyon::path::{Path, Winding},
    lyon::path::builder::BorderRadii as LyonBorderRadii,
    lyon::math::Box2D,
    lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder, StrokeTessellator, StrokeOptions, StrokeVertex, LineCap, LineJoin},
    wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture, BindGroupLayout, BindGroup, Buffer as WgpuBuffer, TextureViewDescriptor},
    wgpu::util::DeviceExt,
    glyphon::{
        Cache, Shaping, Buffer as GlyphonBuffer,
        SwashCache, TextAtlas, TextRenderer, Viewport, TextArea,
        Attrs, TextBounds, Resolution, Metrics, Family as GlyphonFamily, ColorMode, FamilyOwned
    },
    bytemuck,
    web_sys::HtmlCanvasElement,
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

pub async fn create_graphics(canvas: HtmlCanvasElement, width: u32, height: u32) -> Graphics {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(SurfaceTarget::Canvas(canvas))
        .unwrap_throw();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap_throw();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WGPU Device"),
                memory_hints: wgpu::MemoryHints::default(),
                required_features: wgpu::Features::empty(),
                #[cfg(feature = "webgpu")]
                required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                #[cfg(feature = "webgl")]
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                trace: wgpu::Trace::Off,
            },
        )
        .await
        .unwrap_throw();

    let surface_caps = surface.get_capabilities(&adapter);
    let preferred_linear_formats = [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Bgra8Unorm,
    ];
    let surface_format = preferred_linear_formats.iter()
        .copied()
        .find(|format| surface_caps.formats.contains(format))
        .unwrap_or(surface_caps.formats[0]);
    let is_srgb = false;
    let target_format = surface_format;

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: surface_caps.present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    let uniforms = CanvasUniforms {
        width: width as f32,
        height: height as f32,
        _padding1: 0.0,
        _padding2: 0.0,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Canvas Uniform Buffer"),
        contents: bytemuck::cast_slice(&[uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Canvas Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Canvas Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("MSAA Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: surface_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let mut viewport = Viewport::new(&device, &cache);
    viewport.update(&queue, Resolution { width, height });

    let color_mode = if is_srgb {
        ColorMode::Accurate
    } else {
        ColorMode::Web
    };

    let mut atlas = TextAtlas::with_color_mode(
        &device,
        &queue,
        &cache,
        target_format,
        color_mode,
    );

    let text_renderer = TextRenderer::new(
        &mut atlas,
        &device,
        MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        None,
    );

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shape Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("backend_wgpu/shaders.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shape Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shape Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[ColoredVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    Graphics {
        device,
        queue,
        surface,
        surface_config,
        is_srgb,
        msaa_texture,
        swash_cache,
        viewport,
        atlas,
        text_renderer,
        uniform_buffer,
        bind_group_layout,
        bind_group,
        rect_pipeline,
    }
}

/// Resize the graphics resources to match the new canvas size.
pub fn resize_graphics(graphics: &mut Graphics, width: u32, height: u32) {
    let new_width = width.max(1);
    let new_height = height.max(1);
    graphics.surface_config.width = new_width;
    graphics.surface_config.height = new_height;
    graphics.surface.configure(&graphics.device, &graphics.surface_config);
    graphics.msaa_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("MSAA Texture"),
        size: wgpu::Extent3d { width: new_width, height: new_height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: crate::backend::MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: graphics.surface_config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    graphics.viewport.update(&graphics.queue, glyphon::Resolution { width: new_width, height: new_height });
    let uniforms = crate::backend::CanvasUniforms { width: new_width as f32, height: new_height as f32, _padding1: 0.0, _padding2: 0.0 };
    graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
}

pub fn font_weight_to_glyphon(weight: crate::object2d::FontWeight) -> glyphon::fontdb::Weight {
    use crate::object2d::FontWeight::*;
    match weight {
        Thin => glyphon::fontdb::Weight::THIN,
        ExtraLight => glyphon::fontdb::Weight::EXTRA_LIGHT,
        Light => glyphon::fontdb::Weight::LIGHT,
        Regular => glyphon::fontdb::Weight::NORMAL,
        Medium => glyphon::fontdb::Weight::MEDIUM,
        SemiBold => glyphon::fontdb::Weight::SEMIBOLD,
        Bold => glyphon::fontdb::Weight::BOLD,
        ExtraBold => glyphon::fontdb::Weight::EXTRA_BOLD,
        Black => glyphon::fontdb::Weight::BLACK,
    }
}

// Backend-specific conversion: Family -> glyphon::FamilyOwned
impl From<&crate::object2d::Family> for glyphon::FamilyOwned {
    fn from(family: &crate::object2d::Family) -> Self {
        use crate::object2d::Family;
        match family {
            Family::Name(name) => glyphon::FamilyOwned::Name(name.clone().into_owned().into()),
            Family::SansSerif => glyphon::FamilyOwned::SansSerif,
            Family::Serif => glyphon::FamilyOwned::Serif,
            Family::Monospace => glyphon::FamilyOwned::Monospace,
            Family::Cursive => glyphon::FamilyOwned::Cursive,
            Family::Fantasy => glyphon::FamilyOwned::Fantasy,
        }
    }
}
