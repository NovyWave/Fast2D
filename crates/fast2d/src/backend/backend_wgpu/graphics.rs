//! Graphics module for Fast2D using WGPU (WebGPU) backend.
//!
//! This module manages all GPU resources, pipelines, and rendering state needed to draw 2D graphics efficiently.
//! It is designed to be beginner-friendly and well-documented for those new to graphics programming.

use wgpu::{Device, Queue, Surface, SurfaceConfiguration, Texture, BindGroup, Buffer as WgpuBuffer};
use super::MSAA_SAMPLE_COUNT;
use glyphon::Viewport;
use bytemuck;
use wgpu::SurfaceTarget;
use web_sys::HtmlCanvasElement;
use glyphon::{Cache, SwashCache, TextAtlas, TextRenderer, Resolution, ColorMode};
use wgpu::util::DeviceExt;
use web_sys::wasm_bindgen::UnwrapThrowExt;

/// Uniforms for the canvas, passed to shaders.
///
/// Uniforms are small pieces of data sent from the CPU to the GPU, often used to pass global parameters like screen size.
/// Padding fields are for alignment (required by WGSL, the shading language for WGPU).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    /// Width of the canvas in pixels
    width: f32,
    /// Height of the canvas in pixels
    height: f32,
    /// Padding for alignment (unused, but required for GPU memory layout)
    _padding1: f32,
    /// Padding for alignment (unused, but required for GPU memory layout)
    _padding2: f32,
}

/// Holds all GPU resources and state needed for rendering.
///
/// This struct is the main entry point for all graphics operations. It owns the device, queue, surface, and all other GPU objects.
pub struct Graphics {
    /// The GPU device (main handle to the graphics card)
    pub device: Device,
    /// The GPU queue (used to submit commands to the device)
    pub queue: Queue,
    /// The surface to render to (usually a window or canvas)
    pub surface: Surface<'static>,
    /// Configuration for the surface (size, format, etc)
    pub surface_config: SurfaceConfiguration,
    /// Texture used for multisample anti-aliasing (MSAA)
    pub msaa_texture: Texture,
    /// Glyphon font cache for fast text rendering
    pub swash_cache: glyphon::SwashCache,
    /// Glyphon viewport for text layout
    pub viewport: glyphon::Viewport,
    /// Glyphon text atlas (texture storing rendered glyphs)
    pub atlas: glyphon::TextAtlas,
    /// Glyphon text renderer
    pub text_renderer: glyphon::TextRenderer,
    /// Buffer holding canvas uniforms (size, etc)
    pub uniform_buffer: WgpuBuffer,
    /// Bind group for uniforms (used by shaders)
    pub bind_group: BindGroup,
    /// Pipeline for drawing rectangles (shapes)
    pub rect_pipeline: wgpu::RenderPipeline,
}

/// Resize the graphics surface and update all dependent resources.
///
/// This should be called whenever the window or canvas size changes.
/// It updates the surface, MSAA texture, text viewport, and uniform buffer.
///
/// # Arguments
/// * `graphics` - The graphics state to update
/// * `width` - New width in pixels
/// * `height` - New height in pixels
pub fn resize_graphics(graphics: &mut Graphics, width: u32, height: u32) {
    // Clamp to at least 1 pixel to avoid zero-size surfaces
    let new_width = width.max(1);
    let new_height = height.max(1);
    // Update surface config
    graphics.surface_config.width = new_width;
    graphics.surface_config.height = new_height;
    // Reconfigure the surface (resize the swapchain)
    graphics.surface.configure(&graphics.device, &graphics.surface_config);
    // Recreate the MSAA texture for the new size
    graphics.msaa_texture = create_msaa_texture(&graphics.device, new_width, new_height, graphics.surface_config.format);
    // Update the text viewport for the new size
    graphics.viewport.update(&graphics.queue, glyphon::Resolution { width: new_width, height: new_height });
    // Update the uniform buffer with the new size
    let uniforms = CanvasUniforms { width: new_width as f32, height: new_height as f32, _padding1: 0.0, _padding2: 0.0 };
    graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
}

/// Create a new MSAA (multisample anti-aliasing) texture for smoother edges.
///
/// # Arguments
/// * `device` - The GPU device
/// * `width` - Texture width in pixels
/// * `height` - Texture height in pixels
/// * `format` - Texture format (must match the surface format)
///
/// # Returns
/// A new MSAA texture.
fn create_msaa_texture(device: &Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("MSAA Texture"),
        size: wgpu::Extent3d { width: width.max(1), height: height.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Create and initialize the main Graphics struct for rendering.
///
/// This function sets up the GPU device, surface, pipelines, and all resources needed for drawing.
///
/// # Arguments
/// * `canvas` - The HTML canvas element to render to
/// * `width` - Initial width in pixels
/// * `height` - Initial height in pixels
///
/// # Returns
/// A fully initialized Graphics struct ready for rendering.
pub async fn create_graphics(canvas: HtmlCanvasElement, width: u32, height: u32) -> Graphics {
    // Create a new WGPU instance (entry point to the GPU API)
    let instance = wgpu::Instance::default();
    // Create a surface from the HTML canvas
    let surface = instance
        .create_surface(SurfaceTarget::Canvas(canvas))
        .unwrap_throw();

    // Request a suitable GPU adapter (graphics card)
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap_throw();

    // Request a logical device and command queue from the adapter
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

    // Get the supported surface formats and pick one
    let surface_caps = surface.get_capabilities(&adapter);
    let preferred_linear_formats = [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Bgra8Unorm,
    ];
    let surface_format = preferred_linear_formats.iter()
        .copied()
        .find(|format| surface_caps.formats.contains(format))
        .unwrap_or(surface_caps.formats[0]);
    let target_format = surface_format;

    // Configure the surface (swapchain) for rendering
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

    // Create the uniform buffer for passing canvas size to shaders
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

    // Create a bind group layout for the uniform buffer
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

    // Create a bind group to actually bind the uniform buffer to the pipeline
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Canvas Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // Create a dummy 1x1 MSAA texture; will be replaced by resize_graphics
    let msaa_texture = create_msaa_texture(&device, 1, 1, surface_format);

    // Set up Glyphon for fast, high-quality text rendering
    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let mut viewport = Viewport::new(&device, &cache);
    viewport.update(&queue, Resolution { width, height });

    let color_mode = ColorMode::Web;

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
        wgpu::MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        None,
    );

    // Load the WGSL shader for drawing shapes
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shape Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
    });
    // Create the pipeline layout (binds resources to the pipeline)
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shape Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    // Create the render pipeline for drawing rectangles
    let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shape Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[super::ColoredVertex::desc()],
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
        multisample: wgpu::MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    // Bundle everything into the Graphics struct
    let mut graphics = Graphics {
        device,
        queue,
        surface,
        surface_config,
        msaa_texture,
        swash_cache,
        viewport,
        atlas,
        text_renderer,
        uniform_buffer,
        bind_group,
        rect_pipeline,
    };
    // Ensure all resources are sized correctly
    resize_graphics(&mut graphics, width, height);
    graphics
}
