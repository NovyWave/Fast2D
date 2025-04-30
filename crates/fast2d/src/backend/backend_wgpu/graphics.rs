use wgpu::{Device, Queue, Surface, SurfaceConfiguration, Texture, BindGroup, Buffer as WgpuBuffer};
use super::MSAA_SAMPLE_COUNT;
use glyphon::Viewport;
use bytemuck;
use wgpu::SurfaceTarget;
use web_sys::HtmlCanvasElement;
use glyphon::{Cache, SwashCache, TextAtlas, TextRenderer, Resolution, ColorMode};
use wgpu::util::DeviceExt;
use web_sys::wasm_bindgen::UnwrapThrowExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    width: f32,
    height: f32,
    _padding1: f32,
    _padding2: f32,
}

pub struct Graphics {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub msaa_texture: Texture,
    pub swash_cache: glyphon::SwashCache,
    pub viewport: glyphon::Viewport,
    pub atlas: glyphon::TextAtlas,
    pub text_renderer: glyphon::TextRenderer,
    pub uniform_buffer: WgpuBuffer,
    pub bind_group: BindGroup,
    pub rect_pipeline: wgpu::RenderPipeline,
}

pub fn resize_graphics(graphics: &mut Graphics, width: u32, height: u32) {
    let new_width = width.max(1);
    let new_height = height.max(1);
    graphics.surface_config.width = new_width;
    graphics.surface_config.height = new_height;
    graphics.surface.configure(&graphics.device, &graphics.surface_config);
    graphics.msaa_texture = create_msaa_texture(&graphics.device, new_width, new_height, graphics.surface_config.format);
    graphics.viewport.update(&graphics.queue, glyphon::Resolution { width: new_width, height: new_height });
    let uniforms = CanvasUniforms { width: new_width as f32, height: new_height as f32, _padding1: 0.0, _padding2: 0.0 };
    graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
}

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

    // Create a dummy 1x1 MSAA texture; will be replaced by resize_graphics
    let msaa_texture = create_msaa_texture(&device, 1, 1, surface_format);

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

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shape Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
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
    resize_graphics(&mut graphics, width, height);
    graphics
}
