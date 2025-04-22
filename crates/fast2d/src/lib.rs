pub use zoon;

use zoon::wasm_bindgen::throw_str;
use zoon::web_sys::HtmlCanvasElement;
use zoon::Task;
use zoon::UnwrapThrowExt;

use std::future::Future;
use std::sync::Arc;
use std::borrow::Cow;

// Import glyphon types
use glyphon::{
    fontdb, Buffer, Cache, FontSystem, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextRenderer, Viewport,
};

// Import lyon types
use lyon::math::{Box2D}; // Removed Size, Point, point
use lyon::path::{Winding, Path}; // Added Path
use lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder}; // Added FillVertex, BuffersBuilder
// Removed simple_builder import

// Import wgpu types
use wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture, Color as WgpuColor};
use wgpu::util::DeviceExt;

// Declare the object_2d module and re-export structs
mod object_2d;
pub use object_2d::text::Text;
pub use object_2d::rectangle::Rectangle; // Add this line

const CANVAS_WIDTH: u32 = 350;
const CANVAS_HEIGHT: u32 = 350;
const MSAA_SAMPLE_COUNT: u32 = 4;

// Rectangle struct definition is removed from here

// Enum definition remains here
#[derive(Debug, Clone)]
pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle), // Uses the imported Rectangle
}

pub fn run(canvas: HtmlCanvasElement, objects: Vec<Object2d>) {
    Task::start(async move {
        let mut graphics = create_graphics(canvas).await;
        draw(&mut graphics, &objects)
    });
}

// Define vertex structure for rectangles (matches shader)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RectVertex {
    position: [f32; 2],
    color: [f32; 4], // Add color field
}

impl RectVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position attribute
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // Corresponds to @location(0) in shader
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color attribute
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1, // Corresponds to @location(1) in shader
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
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

        let font_system = { // Removed mut
            // NOTE: Smaller and compressed font would be probably better
            let font_data = include_bytes!("../fonts/FiraCode-Regular.ttf");
            // Consider loading other fonts needed by Text::family here
            FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(font_data))])
        };
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer = TextRenderer::new(
            &mut atlas,
            &device,
            MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false, // Set true for better anti-aliased transparency?
            },
            None,
        );

        // --- Rectangle Pipeline Setup ---
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rectangle Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/rectangle.wgsl"))), // Load from file
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rectangle Pipeline Layout"),
            bind_group_layouts: &[], // Add bind group layouts if using uniforms/textures
            push_constant_ranges: &[],
        });

        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[RectVertex::desc()], // Restore buffer layout
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Enable blending
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Lyon outputs CCW
                cull_mode: None, // Disable culling or set Some(wgpu::Face::Back)
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
            msaa_texture,

            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,

            // Remove old buffer fields
            // vertex_buffer,
            // index_buffer,
            // index_count,
            rect_pipeline, // Keep the pipeline
        }
    }
}

// Update draw function
fn draw(gfx: &mut Graphics, objects: &[Object2d]) {
    gfx.viewport.update(
        &gfx.queue,
        Resolution {
            width: gfx.surface_config.width,
            height: gfx.surface_config.height,
        },
    );

    // --- Text Preparation ---
    // Filter out Text objects first
    let text_objects: Vec<&Text> = objects.iter().filter_map(|obj| {
        if let Object2d::Text(text) = obj { Some(text) } else { None }
    }).collect();

    // Create owned Buffers
    let text_buffers: Vec<Buffer> = text_objects.iter().map(|&text_obj| { // Removed mut
        let mut buffer = Buffer::new(&mut gfx.font_system, text_obj.get_metrics());
        buffer.set_text(
            &mut gfx.font_system,
            text_obj.get_text(),
            &text_obj.get_attrs().as_attrs(),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut gfx.font_system, false);
        buffer // Return the owned buffer
    }).collect();

    // Create TextAreas borrowing from text_buffers
    let text_areas: Vec<TextArea> = text_objects.iter().zip(text_buffers.iter()).map(|(&text_obj, buffer)| {
        TextArea {
            buffer: buffer, // Reference the buffer living in text_buffers
            left: text_obj.get_left(),
            top: text_obj.get_top(),
            scale: 1.0,
            bounds: text_obj.get_text_bounds(),
            default_color: text_obj.get_glyphon_color(),
            custom_glyphs: &[],
        }
    }).collect();

    gfx.text_renderer
        .prepare(
            &gfx.device,
            &gfx.queue,
            &mut gfx.font_system,
            &mut gfx.atlas,
            &gfx.viewport,
            text_areas, // Pass the Vec<TextArea> which borrows from text_buffers
            &mut gfx.swash_cache,
        )
        .unwrap();

    // --- Rectangle Preparation ---
    let mut rect_tessellator = FillTessellator::new();
    let mut rect_geometry: VertexBuffers<RectVertex, u16> = VertexBuffers::new();
    // let mut has_rectangles = false; // Remove flag

    for obj in objects {
        if let Object2d::Rectangle(rect) = obj {
            // has_rectangles = true; // Remove flag

            let rect_color = [
                rect.color.r as f32,
                rect.color.g as f32,
                rect.color.b as f32,
                rect.color.a as f32,
            ];

            let mut path_builder = Path::builder();
            path_builder.add_rounded_rectangle(
                &Box2D::new(rect.position, rect.position + rect.size.to_vector()),
                &rect.border_radii,
                Winding::Positive,
            );
            let path = path_builder.build();

            rect_tessellator.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut rect_geometry, |vertex: FillVertex| {
                    RectVertex {
                        position: vertex.position().to_array(),
                        color: rect_color,
                    }
                }),
            ).unwrap();
        }
    }
    // Restore buffer creation
    let rect_vertex_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Rectangle Vertex Buffer"),
        contents: bytemuck::cast_slice(&rect_geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let rect_index_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Rectangle Index Buffer"),
        contents: bytemuck::cast_slice(&rect_geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let rect_index_count = rect_geometry.indices.len() as u32; // Use count again

    let frame = gfx.surface.get_current_texture().unwrap_throw();
    let swap_chain_view = frame.texture.create_view(&Default::default());
    let msaa_texture_view = gfx.msaa_texture.create_view(&Default::default());
    let mut encoder = gfx.device.create_command_encoder(&Default::default());

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_texture_view,
                resolve_target: Some(&swap_chain_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(WgpuColor::BLACK),
                    store: wgpu::StoreOp::Discard,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Restore drawing using buffers and indices
        if rect_index_count > 0 { // Use rect_index_count again
            rpass.set_pipeline(&gfx.rect_pipeline);
            rpass.set_vertex_buffer(0, rect_vertex_buffer.slice(..)); // Restore
            rpass.set_index_buffer(rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16); // Restore
            rpass.draw_indexed(0..rect_index_count, 0, 0..1); // Restore indexed draw
        }

        gfx.text_renderer
            .render(&gfx.atlas, &gfx.viewport, &mut rpass)
            .unwrap();
    }

    let command_buffer = encoder.finish();
    gfx.queue.submit([command_buffer]);
    frame.present();
    gfx.atlas.trim();
}

#[allow(dead_code)]
struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    msaa_texture: Texture,

    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,

    // Only store the pipeline for rectangles
    rect_pipeline: wgpu::RenderPipeline,
}

// Create a shaders directory and the rectangle shader file
// src/shaders/rectangle.wgsl
