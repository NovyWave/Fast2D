pub use zoon;

use zoon::wasm_bindgen::throw_str;
use zoon::web_sys::HtmlCanvasElement;
use zoon::Task;
use zoon::UnwrapThrowExt;

use std::future::Future;
use std::sync::Arc;
use std::borrow::Cow;

use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};

use lyon::math::{Box2D, Point, point};
use lyon::path::{Winding, builder::BorderRadii};
use lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers};
use lyon::tessellation::geometry_builder::simple_builder;

use wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture};
use wgpu::util::DeviceExt;

const CANVAS_WIDTH: u32 = 350;
const CANVAS_HEIGHT: u32 = 350;
const MSAA_SAMPLE_COUNT: u32 = 4;

pub struct Text {
    text: Cow<'static, str>,
}

impl Text {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            text: text.into(),
        }
    }
}

impl Into<Object2d> for Text {
    fn into(self) -> Object2d {
        Object2d::Text(self)
    }
}

pub struct Rectangle {
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Into<Object2d> for Rectangle {
    fn into(self) -> Object2d {
        Object2d::Rectangle(self)
    }
}

pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle),
}

pub fn run(canvas: HtmlCanvasElement, objects: Vec<Object2d>) {
    Task::start(async move {
        let mut graphics = create_graphics(canvas).await;
        draw(&mut graphics)
    });
}

fn create_graphics(
    canvas: HtmlCanvasElement,
) -> impl Future<Output = Graphics> + 'static {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(SurfaceTarget::Canvas(canvas))
        .unwrap_or_else(|e| throw_str(&format!("{e:#?}")));

    async move {
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
                    required_features: wgpu::Features::default(),
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

        let surface_config = surface
            .get_default_config(&adapter, CANVAS_WIDTH, CANVAS_HEIGHT)
            .unwrap_throw();

        surface.configure(&device, &surface_config);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        // Create multisample texture
        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Texture"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: swapchain_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });




        let mut font_system = {
            // NOTE: Smaller and compressed font would be probably better
            let font_data = include_bytes!("../fonts/FiraCode-Regular.ttf");
            FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(font_data))])
        };
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        // Use MSAA state for text renderer
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
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        text_buffer.set_text(
            &mut font_system,
            "Hello world!",
            &Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);





        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut geometry_builder = simple_builder(&mut geometry);
        let options = FillOptions::tolerance(0.1);
        let mut tessellator = FillTessellator::new();

        let mut builder = tessellator.builder(
            &options,
            &mut geometry_builder,
        );

        builder.add_rounded_rectangle(
            &Box2D { min: point(0.0, 0.0), max: point(100.0, 50.0) },
            &BorderRadii {
                top_left: 10.0,
                top_right: 5.0,
                bottom_left: 20.0,
                bottom_right: 25.0,
            },
            Winding::Positive,
        );

        builder.build().unwrap_throw();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&geometry.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&geometry.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = geometry.indices.len() as u32;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rectangle Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(r#"
                struct VertexInput {
                    @location(0) position: vec2<f32>,
                }

                struct VertexOutput {
                    @builtin(position) clip_position: vec4<f32>,
                }

                @vertex
                fn vs_main(in: VertexInput) -> VertexOutput {
                    var out: VertexOutput;
                    // Center the rectangle: subtract half size (50, 25) before scaling
                    let centered_pos = in.position - vec2<f32>(50.0, 25.0);
                    // Scale position to be relative to half the canvas size
                    let pos = centered_pos / vec2<f32>(175.0, 175.0);
                    // Map to clip space (y is typically inverted in clip space vs screen space)
                    out.clip_position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
                    return out;
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red color
                }
            "#)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rectangle Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: <_>::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Point>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: <_>::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Changed to Ccw because of potential y-flip effect
                cull_mode: None, // Disable culling for simplicity or set to Back if needed
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            // Use MSAA state for rectangle pipeline
            multisample: MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false, // Usually false for opaque geometry
            },
            multiview: None,
            cache: None,
        });

        Graphics {
            device,
            queue,
            surface,
            surface_config,
            msaa_texture,

            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
            
            vertex_buffer,
            index_buffer,
            index_count,
            rect_pipeline,
        }
    }
}

fn draw(gfx: &mut Graphics) {
    gfx.viewport.update(
        &gfx.queue,
        Resolution {
            width: gfx.surface_config.width,
            height: gfx.surface_config.height,
        },
    );

    gfx.text_renderer
        .prepare(
            &gfx.device,
            &gfx.queue,
            &mut gfx.font_system,
            &mut gfx.atlas,
            &gfx.viewport,
            [TextArea {
                buffer: &gfx.text_buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 600,
                    bottom: 160,
                },
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut gfx.swash_cache,
        )
        .unwrap();

    let frame = gfx.surface.get_current_texture().unwrap_throw();
    let swap_chain_view = frame.texture.create_view(&Default::default());
    let msaa_texture_view = gfx.msaa_texture.create_view(&Default::default()); // Create view from stored texture
    let mut encoder = gfx.device.create_command_encoder(&Default::default());

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_texture_view, // Render to MSAA texture
                resolve_target: Some(&swap_chain_view), // Resolve to swap chain texture
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Discard, // Discard MSAA texture content after resolve
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        rpass.set_pipeline(&gfx.rect_pipeline);
        rpass.set_vertex_buffer(0, gfx.vertex_buffer.slice(..));
        rpass.set_index_buffer(gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..gfx.index_count, 0, 0..1);
        
        gfx.text_renderer
            .render(&gfx.atlas, &gfx.viewport, &mut rpass)
            .unwrap();
    }

    let command_buffer = encoder.finish();
    gfx.queue.submit([command_buffer]);
    frame.present();

    gfx.atlas.trim();
}

// fn resized(&mut self, size: PhysicalSize<u32>) {
//     let MaybeGraphics::Graphics(gfx) = &mut self.graphics else {
//         return;
//     };
//     gfx.surface_config.width = size.width;
//     gfx.surface_config.height = size.height;
//     gfx.surface.configure(&gfx.device, &gfx.surface_config);
// }

#[allow(dead_code)]
struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    msaa_texture: Texture, // Add field for MSAA texture

    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,
    
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    rect_pipeline: wgpu::RenderPipeline,
}

// impl ApplicationHandler<Graphics> for Application {
//     fn window_event(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//         _window_id: WindowId,
//         event: WindowEvent,
//     ) {
//         match event {
//             WindowEvent::Resized(size) => self.resized(size),
//             WindowEvent::RedrawRequested => self.draw(),
//             WindowEvent::CloseRequested => event_loop.exit(),
//             _ => (),
//         }
//     }

//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         if let MaybeGraphics::Builder(builder) = &mut self.graphics {
//             builder.build_and_send(event_loop);
//         }
//     }

//     fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
//         self.graphics = MaybeGraphics::Graphics(graphics);
//     }
// }
