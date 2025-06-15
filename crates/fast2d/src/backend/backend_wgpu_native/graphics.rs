//! Graphics module for Fast2D using WGPU (Native) backend.
//!
//! This module manages all GPU resources, pipelines, and rendering state needed to draw 2D graphics efficiently on native platforms.
//! It provides the same functionality as the web backend but uses native WGPU surfaces instead of HTML Canvas elements.

use wgpu::{Device, Queue, Surface, SurfaceConfiguration, Texture, BindGroup, Buffer as WgpuBuffer};
use super::MSAA_SAMPLE_COUNT;
use glyphon::Viewport;
use bytemuck;
use glyphon::{Cache, SwashCache, TextAtlas, TextRenderer};
use wgpu::util::DeviceExt;

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

/// Holds all GPU resources and state needed for rendering on native platforms.
///
/// This struct is the main entry point for all graphics operations. It owns the device, queue, surface, and all other GPU objects.
pub struct Graphics {
    /// The GPU device (main handle to the graphics card)
    pub device: Device,
    /// The GPU queue (used to submit commands to the device)
    pub queue: Queue,
    /// The surface to render to (native window surface)
    pub surface: Surface<'static>,
    /// Configuration for the surface (size, format, etc)
    pub surface_config: SurfaceConfiguration,
    /// Texture used for multisample anti-aliasing (MSAA)
    pub msaa_texture: Texture,
    /// Glyphon font cache for fast text rendering
    pub swash_cache: glyphon::SwashCache,
    /// Glyphon viewport for text layout
    pub viewport: glyphon::Viewport,
    /// Glyphon text atlas for glyph storage
    pub text_atlas: glyphon::TextAtlas,
    /// Glyphon text renderer for drawing text
    pub text_renderer: glyphon::TextRenderer,
    /// Main render pipeline for colored geometry
    pub render_pipeline: wgpu::RenderPipeline,
    /// Bind group layout for uniforms
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform buffer for canvas parameters
    pub uniform_buffer: WgpuBuffer,
    /// Bind group for uniforms
    pub uniform_bind_group: BindGroup,
}

/// Creates a new Graphics context with a native WGPU surface.
///
/// This function initializes all GPU resources needed for 2D rendering using a pre-created WGPU surface.
/// Unlike the web version that takes an HtmlCanvasElement, this takes raw WGPU objects.
///
/// # Arguments
/// * `surface` - The WGPU surface to render to (typically created from a native window)
/// * `device` - The WGPU device (graphics card handle)
/// * `queue` - The WGPU queue (command submission)
/// * `width` - Initial width in pixels
/// * `height` - Initial height in pixels
///
/// # Returns
/// A fully initialized Graphics struct ready for rendering.
pub async fn create_graphics(
    surface: Surface<'static>, 
    device: Device,
    queue: Queue,
    width: u32, 
    height: u32
) -> Graphics {
    panic!("create_graphics without adapter is not supported. Use create_graphics_with_adapter instead.");
}

/// Creates a new Graphics context with a native WGPU surface and adapter.
///
/// This function initializes all GPU resources needed for 2D rendering using a pre-created WGPU surface.
/// Unlike the web version that takes an HtmlCanvasElement, this takes raw WGPU objects.
///
/// # Arguments
/// * `surface` - The WGPU surface to render to (typically created from a native window)
/// * `device` - The WGPU device (graphics card handle)
/// * `queue` - The WGPU queue (command submission)
/// * `adapter` - The WGPU adapter (needed for surface capabilities)
/// * `width` - Initial width in pixels
/// * `height` - Initial height in pixels
///
/// # Returns
/// A fully initialized Graphics struct ready for rendering.
pub async fn create_graphics_with_adapter(
    surface: Surface<'static>, 
    device: Device,
    queue: Queue,
    adapter: wgpu::Adapter,
    width: u32, 
    height: u32
) -> Graphics {
    // Get surface capabilities and choose a suitable format
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    // Configure the surface with resize-optimized settings
    let present_mode = if cfg!(windows) && surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
        wgpu::PresentMode::Immediate // Better for resize on Windows
    } else if surface_caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
        wgpu::PresentMode::Fifo // VSync - most compatible
    } else {
        surface_caps.present_modes[0]
    };
    
    let alpha_mode = if surface_caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
        wgpu::CompositeAlphaMode::Opaque
    } else {
        surface_caps.alpha_modes[0]
    };
    
    let surface_config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    // Create MSAA texture for anti-aliasing
    let msaa_texture = create_msaa_texture(&device, &surface_config);

    // Initialize Glyphon for text rendering
    let mut swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let viewport = Viewport::new(&device, &cache);
    let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
    let text_renderer = TextRenderer::new(
        &mut text_atlas,
        &device,
        wgpu::MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        None,
    );

    // Create shader module
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fast2D Native Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../backend_wgpu/shaders.wgsl").into()),
    });

    // Create bind group layout for uniforms
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        label: Some("uniform_bind_group_layout"),
    });

    // Create pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    // Create render pipeline
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[super::ColoredVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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

    // Create uniform buffer
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[CanvasUniforms {
            width: width as f32,
            height: height as f32,
            _padding1: 0.0,
            _padding2: 0.0,
        }]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Create bind group
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("uniform_bind_group"),
    });

    Graphics {
        device,
        queue,
        surface,
        surface_config,
        msaa_texture,
        swash_cache,
        viewport,
        text_atlas,
        text_renderer,
        render_pipeline,
        bind_group_layout,
        uniform_buffer,
        uniform_bind_group,
    }
}

/// Handles resizing of the graphics context for native surfaces.
///
/// # Arguments
/// * `graphics` - Mutable reference to the Graphics context
/// * `width` - The new width in pixels
/// * `height` - The new height in pixels
pub fn resize_graphics(graphics: &mut Graphics, width: u32, height: u32) {
    // Skip invalid sizes
    if width == 0 || height == 0 {
        println!("Warning: Skipping resize with zero dimensions: {}x{}", width, height);
        return;
    }
    
    // Skip if size hasn't actually changed
    if graphics.surface_config.width == width && graphics.surface_config.height == height {
        return;
    }
    
    println!("Configuring surface: {}x{}", width, height);
    graphics.surface_config.width = width;
    graphics.surface_config.height = height;
    
    // Configure surface - this is the most likely place for resize issues
    graphics.surface.configure(&graphics.device, &graphics.surface_config);
    
    // Recreate MSAA texture with new size
    println!("Creating MSAA texture: {}x{}", width, height);
    graphics.msaa_texture = create_msaa_texture(&graphics.device, &graphics.surface_config);
    println!("MSAA texture created successfully");
    
    // Update uniform buffer with new dimensions
    let uniforms = CanvasUniforms {
        width: width as f32,
        height: height as f32,
        _padding1: 0.0,
        _padding2: 0.0,
    };
    graphics.queue.write_buffer(
        &graphics.uniform_buffer,
        0,
        bytemuck::cast_slice(&[uniforms]),
    );
    
    println!("Surface resize completed: {}x{}", width, height);
}

/// Creates an MSAA texture for anti-aliasing.
fn create_msaa_texture(device: &Device, surface_config: &SurfaceConfiguration) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: surface_config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("MSAA Texture"),
        view_formats: &[],
    })
}