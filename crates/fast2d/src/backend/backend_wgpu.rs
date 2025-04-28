mod register_fonts;
pub use register_fonts::register_fonts;

mod canvas_wrapper;
pub use canvas_wrapper::CanvasWrapper;

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

#[derive(Debug)]
pub enum FontSystemInitError {
    DatabaseError(String),
    AlreadyInitialized,
    NoFontsProvided,
}

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
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rectangle.wgsl").into()),
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

pub fn draw_wgpu(gfx: &mut Graphics, objects: &[crate::Object2d]) {
    let output = match gfx.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(e) => {
            console::error_1(&JsValue::from_str(&format!("Error getting current texture: {:?}", e)));
            if e == wgpu::SurfaceError::Lost {
                gfx.surface.configure(&gfx.device, &gfx.surface_config);
                return;
            }
            return;
        }
    };

    let view = output.texture.create_view(&TextureViewDescriptor::default());
    let msaa_view = gfx.msaa_texture.create_view(&TextureViewDescriptor::default());

    let mut font_system = FONT_SYSTEM.get()
        .expect("FontSystem not initialized")
        .lock()
        .expect("Failed to lock FontSystem Mutex");

    let mut glyph_buffers: Vec<GlyphonBuffer> = Vec::new();

    for obj in objects {
        if let crate::Object2d::Text(text) = obj {
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let line_height_pixels = text.font_size * text.line_height_multiplier;
            let mut buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(text.font_size, line_height_pixels));
            buffer.set_size(&mut font_system, Some(text_width_f32), Some(text_height_f32));

            let family_owned: FamilyOwned = text.family.clone().into();
            let glyphon_family = match &family_owned {
                FamilyOwned::Name(name) => GlyphonFamily::Name(name.as_str()),
                FamilyOwned::SansSerif => GlyphonFamily::SansSerif,
                FamilyOwned::Serif => GlyphonFamily::Serif,
                FamilyOwned::Monospace => GlyphonFamily::Monospace,
                FamilyOwned::Cursive => GlyphonFamily::Cursive,
                FamilyOwned::Fantasy => GlyphonFamily::Fantasy,
            };

            let family_for_query = match &glyphon_family {
                GlyphonFamily::Name(name) => glyphon::fontdb::Family::Name(name),
                GlyphonFamily::SansSerif => glyphon::fontdb::Family::SansSerif,
                GlyphonFamily::Serif => glyphon::fontdb::Family::Serif,
                GlyphonFamily::Monospace => glyphon::fontdb::Family::Monospace,
                GlyphonFamily::Cursive => glyphon::fontdb::Family::Cursive,
                GlyphonFamily::Fantasy => glyphon::fontdb::Family::Fantasy,
            };

            let font_query = glyphon::fontdb::Query {
                families: &[family_for_query],
                ..Default::default()
            };

            let font_exists = font_system.db().query(&font_query).is_some();
            if !font_exists {
                #[cfg(target_arch = "wasm32")]
                {
                    let warning_message = format!("Warning: Font family '{:?}' not found. Falling back to default.", text.family);
                    web_sys::console::warn_1(&JsValue::from_str(&warning_message));
                }
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("Warning: Font family '{:?}' not found. Falling back to default.");
            }

            let glyphon_color = text.color.to_glyphon_color();
            let attrs = Attrs::new()
                .family(glyphon_family)
                .color(glyphon_color)
                .weight(font_weight_to_glyphon(text.weight))
                .style(if text.italic { glyphon::fontdb::Style::Italic } else { glyphon::fontdb::Style::Normal });
            buffer.set_text(&mut font_system, &text.text, &attrs, Shaping::Advanced);
            glyph_buffers.push(buffer);
        }
    }

    let mut text_areas: Vec<TextArea> = Vec::new();
    let mut buffer_idx = 0;
    for obj in objects {
        if let crate::Object2d::Text(text) = obj {
            let glyphon_color = text.color.to_glyphon_color();
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let text_area = TextArea {
                buffer: &glyph_buffers[buffer_idx],
                left: text.left,
                top: text.top,
                bounds: TextBounds {
                    left: text.left as i32,
                    top: text.top as i32,
                    right: (text.left + text_width_f32) as i32,
                    bottom: (text.top + text_height_f32) as i32,
                },
                default_color: glyphon_color,
                scale: 1.0,
                custom_glyphs: &[],
            };
            text_areas.push(text_area);
            buffer_idx += 1;
        }
    }

    match gfx.text_renderer.prepare(
        &gfx.device, &gfx.queue, &mut font_system, &mut gfx.atlas, &gfx.viewport,
        text_areas.into_iter(), &mut gfx.swash_cache,
    ) {
        Ok(_) => {}
        Err(e) => console::error_1(&JsValue::from_str(&format!("Error preparing text renderer: {:?}", e))),
    }

    let mut buffers: VertexBuffers<ColoredVertex, u32> = VertexBuffers::new();
    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    for obj in objects {
        match obj {
            crate::Object2d::Rectangle(rect) => {
                let linear_color = rect.color.to_linear();
                let mut builder = Path::builder();
                let rect_box = Box2D::new(point(rect.position.x, rect.position.y), point(rect.position.x + rect.size.width, rect.position.y + rect.size.height));
                if rect.border_radii.top_left > 0.0 || rect.border_radii.top_right > 0.0 || rect.border_radii.bottom_left > 0.0 || rect.border_radii.bottom_right > 0.0 {
                    builder.add_rounded_rectangle(&rect_box, &LyonBorderRadii { top_left: rect.border_radii.top_left, top_right: rect.border_radii.top_right, bottom_left: rect.border_radii.bottom_left, bottom_right: rect.border_radii.bottom_right }, Winding::Positive);
                } else {
                    builder.add_rectangle(&rect_box, Winding::Positive);
                }
                let path = builder.build();
                if rect.color.a > 0.0 {
                    fill_tessellator.tessellate_path(&path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                }
                if let (Some(border_width), Some(border_color_val)) = (rect.border_width, rect.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let linear_border_color = border_color_val.to_linear();
                        let options = StrokeOptions::default().with_line_width(border_width);
                        stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_border_color })).unwrap();
                    }
                }
            }
            crate::Object2d::Circle(circle) => {
                let linear_color = circle.color.to_linear();
                let mut builder = Path::builder();
                builder.add_circle(point(circle.center.x, circle.center.y), circle.radius, Winding::Positive);
                let path = builder.build();
                if circle.color.a > 0.0 {
                    fill_tessellator.tessellate_path(&path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                }
                if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let linear_border_color = border_color_val.to_linear();
                        let options = StrokeOptions::default().with_line_width(border_width);
                        stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_border_color })).unwrap();
                    }
                }
            }
            crate::Object2d::Line(line) => {
                let linear_color = line.color.to_linear();
                let mut builder = Path::builder();
                if line.points.len() >= 2 {
                    builder.begin(point(line.points[0].x, line.points[0].y));
                    for i in 1..line.points.len() {
                        builder.line_to(point(line.points[i].x, line.points[i].y));
                    }
                    builder.end(false);
                }
                let path = builder.build();
                if line.points.len() >= 2 && line.color.a > 0.0 {
                    let options = StrokeOptions::default().with_line_width(line.width).with_line_cap(LineCap::Round).with_line_join(LineJoin::Round);
                    stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                }
            }
            crate::Object2d::Text(_) => {}
        }
    }

    let vertex_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"), contents: bytemuck::cast_slice(&buffers.vertices), usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"), contents: bytemuck::cast_slice(&buffers.indices), usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = buffers.indices.len() as u32;

    let mut encoder = gfx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(&view),
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });

        if num_indices > 0 {
            render_pass.set_pipeline(&gfx.rect_pipeline);
            render_pass.set_bind_group(0, &gfx.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        match gfx.text_renderer.render(&gfx.atlas, &gfx.viewport, &mut render_pass) {
            Ok(_) => {}
            Err(e) => console::error_1(&JsValue::from_str(&format!("Error rendering text: {:?}", e))),
        }
    }
    gfx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

pub fn font_weight_to_glyphon(weight: crate::object_2d::text::FontWeight) -> glyphon::fontdb::Weight {
    use crate::object_2d::text::FontWeight::*;
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
