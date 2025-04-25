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
use lyon::math::{Box2D, Vector, Size}; // Added Vector, Size
use lyon::path::{Winding, Path};
use lyon::path::builder::BorderRadii; // Added BorderRadii back
use lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder, StrokeTessellator, StrokeOptions, StrokeVertex, LineCap, LineJoin}; // Added stroke types

// Import wgpu types
use wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture, Color as WgpuColor, BindGroupLayout, BindGroup, Buffer as WgpuBuffer}; // Added BindGroupLayout, BindGroup, WgpuBuffer
use wgpu::util::DeviceExt;

// Declare the object_2d module and re-export structs
mod object_2d;
pub use object_2d::text::Text;
pub use object_2d::rectangle::Rectangle; // Add this line
pub use object_2d::circle::Circle; // Added
pub use object_2d::line::Line; // Add this line

const CANVAS_WIDTH: u32 = 350;
const CANVAS_HEIGHT: u32 = 350;
const MSAA_SAMPLE_COUNT: u32 = 4; // Multisampling for anti-aliasing

// Define the uniform structure (must match WGSL and be 16-byte aligned)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    width: f32,
    height: f32,
    // Add padding to meet 16-byte alignment requirement
    _padding1: f32,
    _padding2: f32,
}

// Enum definition remains here
#[derive(Debug, Clone)]
pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle), // Uses the imported Rectangle
    Circle(Circle), // Added
    Line(Line), // Added Line variant
}

pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas: Option<HtmlCanvasElement>,
    graphics: Option<Graphics>,
}

impl CanvasWrapper {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            canvas: None,
            graphics: None,
        }
    }

    pub async fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        self.canvas = Some(canvas.clone());
        self.graphics = Some(create_graphics(canvas).await);
        self.draw();
    }

    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        self.draw();
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        if let Some(graphics) = &mut self.graphics {
            // Ensure width and height are not zero, which can cause issues
            let new_width = width.max(1);
            let new_height = height.max(1);

            graphics.surface_config.width = new_width;
            graphics.surface_config.height = new_height;
            graphics.surface.configure(&graphics.device, &graphics.surface_config);

            // Recreate the MSAA texture with the new size
            graphics.msaa_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Texture"),
                size: wgpu::Extent3d {
                    width: new_width,
                    height: new_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: MSAA_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: graphics.surface_config.format, // Use the current surface format
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            // Update the uniform buffer
            let uniforms = CanvasUniforms {
                width: new_width as f32,
                height: new_height as f32,
                _padding1: 0.0, // Initialize padding
                _padding2: 0.0, // Initialize padding
            };
            graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        }
        self.draw();
    }

    fn draw(&mut self) {
        if let Some(graphics) = &mut self.graphics {
            draw(graphics, &self.objects);
        }
    }
}


pub fn run(canvas: HtmlCanvasElement, objects: Vec<Object2d>) {
    Task::start(async move {
        let mut graphics = create_graphics(canvas).await;
        draw(&mut graphics, &objects)
    });
}

// Define vertex structure for rectangles (matches shader) - Renamed
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ColoredVertex { // Renamed from RectVertex
    position: [f32; 2],
    color: [f32; 4],
}

impl ColoredVertex { // Renamed from RectVertex
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress, // Use renamed struct
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

        // --- Uniform Buffer Setup ---
        let uniforms = CanvasUniforms {
            width: surface_config.width as f32,
            height: surface_config.height as f32,
            _padding1: 0.0, // Initialize padding
            _padding2: 0.0, // Initialize padding
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Canvas Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, // Add COPY_DST
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Canvas Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX, // Visible only to vertex shader
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
            bind_group_layouts: &[&bind_group_layout], // Use the created bind group layout
            push_constant_ranges: &[],
        });

        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[ColoredVertex::desc()], // Use renamed struct desc
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

            uniform_buffer, // Add uniform buffer
            bind_group_layout, // Add bind group layout
            bind_group, // Add bind group

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


    // --- Shape Tessellation ---
    // Combine all shape geometry into a single buffer
    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();
    let mut geometry: VertexBuffers<ColoredVertex, u16> = VertexBuffers::new();

    for obj in objects {
        match obj {
            Object2d::Rectangle(rect) => {
                // ... (Rectangle tessellation logic - add to `geometry`) ...
                let mut outer_path_builder = Path::builder();
                outer_path_builder.add_rounded_rectangle(
                    &Box2D::new(rect.position, rect.position + rect.size.to_vector()),
                    &rect.border_radii,
                    Winding::Positive,
                );
                let outer_path = outer_path_builder.build(); // Path for the stroke

                // --- Tessellate Fill (using potentially inset path) ---
                let fill_color = [
                    rect.color.r as f32,
                    rect.color.g as f32,
                    rect.color.b as f32,
                    rect.color.a as f32,
                ];

                let fill_path = if let Some(border_width) = rect.border_width {
                    // Inset the path for fill if there's a border
                    let half_border_width = border_width / 2.0;
                    let inner_pos = rect.position + Vector::new(half_border_width, half_border_width);
                    let inner_size = Size::new(
                        (rect.size.width - border_width).max(0.0),
                        (rect.size.height - border_width).max(0.0)
                    );
                    let inner_radii = BorderRadii {
                        top_left: (rect.border_radii.top_left - half_border_width).max(0.0),
                        top_right: (rect.border_radii.top_right - half_border_width).max(0.0),
                        bottom_right: (rect.border_radii.bottom_right - half_border_width).max(0.0),
                        bottom_left: (rect.border_radii.bottom_left - half_border_width).max(0.0),
                    };

                    let mut inner_path_builder = Path::builder();
                    inner_path_builder.add_rounded_rectangle(
                        &Box2D::new(inner_pos, inner_pos + inner_size.to_vector()),
                        &inner_radii,
                        Winding::Positive,
                    );
                    inner_path_builder.build()
                } else {
                    outer_path.clone()
                };

                // Add fill geometry
                fill_tessellator.tessellate_path(
                    &fill_path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| { // Add to geometry
                        ColoredVertex {
                            position: vertex.position().to_array(),
                            color: fill_color,
                        }
                    }),
                ).unwrap();

                // --- Tessellate Border (using outer path) ---
                if let (Some(border_width), Some(border_color_wgpu)) = (rect.border_width, rect.border_color) {
                    let border_color = [
                        border_color_wgpu.r as f32,
                        border_color_wgpu.g as f32,
                        border_color_wgpu.b as f32,
                        border_color_wgpu.a as f32,
                    ];
                    let mut stroke_options = StrokeOptions::default();
                    stroke_options.line_width = border_width;
                    stroke_options.start_cap = LineCap::Round;
                    stroke_options.end_cap = LineCap::Round;
                    stroke_options.line_join = LineJoin::Round;

                    // Add border geometry
                    stroke_tessellator.tessellate_path(
                        &outer_path,
                        &stroke_options,
                        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| { // Add to geometry
                            ColoredVertex {
                                position: vertex.position().to_array(),
                                color: border_color,
                            }
                        }),
                    ).unwrap();
                }
            }
            Object2d::Circle(circle) => {
                // ... (Circle tessellation logic - add to `geometry`) ...
                let mut outer_path_builder = Path::builder();
                outer_path_builder.add_circle(circle.center, circle.radius, Winding::Positive);
                let outer_path = outer_path_builder.build(); // Path for the stroke

                // --- Tessellate Fill (using potentially inset path) ---
                let fill_color = [
                    circle.color.r as f32,
                    circle.color.g as f32,
                    circle.color.b as f32,
                    circle.color.a as f32,
                ];

                if let Some(border_width) = circle.border_width {
                    // Inset the path for fill if there's a border
                    let half_border_width = border_width / 2.0;
                    let inner_radius = (circle.radius - half_border_width).max(0.0);

                    // Only build and tessellate if inner radius is positive
                    if inner_radius > f32::EPSILON {
                        let mut inner_path_builder = Path::builder();
                        inner_path_builder.add_circle(circle.center, inner_radius, Winding::Positive);
                        let fill_path = inner_path_builder.build();

                        // Add fill geometry
                        fill_tessellator.tessellate_path(
                            &fill_path,
                            &FillOptions::default(),
                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| { // Add to geometry
                                ColoredVertex {
                                    position: vertex.position().to_array(),
                                    color: fill_color,
                                }
                            }),
                        ).unwrap();
                    }
                    // If inner_radius is zero or less, do nothing for fill
                } else {
                    // No border, tessellate the outer path for fill
                    // Add fill geometry
                    fill_tessellator.tessellate_path(
                        &outer_path, // Use outer_path directly
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| { // Add to geometry
                                ColoredVertex {
                                    position: vertex.position().to_array(),
                                    color: fill_color,
                                }
                            }),
                    ).unwrap();
                }


                // --- Tessellate Border (using outer path) ---
                if let (Some(border_width), Some(border_color_wgpu)) = (circle.border_width, circle.border_color) {
                    let border_color = [
                        border_color_wgpu.r as f32,
                        border_color_wgpu.g as f32,
                        border_color_wgpu.b as f32,
                        border_color_wgpu.a as f32,
                    ];
                    let mut stroke_options = StrokeOptions::default();
                    stroke_options.line_width = border_width;
                    stroke_options.start_cap = LineCap::Round;
                    stroke_options.end_cap = LineCap::Round;
                    stroke_options.line_join = LineJoin::Round;

                    // Add border geometry
                    stroke_tessellator.tessellate_path(
                        &outer_path,
                        &stroke_options,
                        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| { // Add to geometry
                            ColoredVertex {
                                position: vertex.position().to_array(),
                                color: border_color,
                            }
                        }),
                    ).unwrap();
                }
            }
            Object2d::Line(line) => {
                if line.points.len() < 2 { continue; } // Need at least two points for a line

                let line_color = [
                    line.color.r as f32,
                    line.color.g as f32,
                    line.color.b as f32,
                    line.color.a as f32,
                ];

                let mut path_builder = Path::builder();
                path_builder.begin(line.points[0]);
                for point in line.points.iter().skip(1) {
                    path_builder.line_to(*point);
                }
                path_builder.end(false); // Uncomment this line to properly finish the path segment
                let path = path_builder.build();

                let mut stroke_options = StrokeOptions::default();
                stroke_options.line_width = line.width; // Use line.width
                stroke_options.start_cap = LineCap::Round; // Or Butt, Square
                stroke_options.end_cap = LineCap::Round;   // Or Butt, Square
                stroke_options.line_join = LineJoin::Round; // Or Miter, Bevel

                // Add line geometry
                stroke_tessellator.tessellate_path(
                    &path,
                    &stroke_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| { // Add to geometry
                        ColoredVertex {
                            position: vertex.position().to_array(),
                            color: line_color,
                        }
                    }),
                ).unwrap();
            }
            Object2d::Text(_) => {
                // Text is handled separately
            }
        }
    }

    // --- Create Combined Shape Buffers ---
    let vertex_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shape Vertex Buffer"), // Combined buffer
        contents: bytemuck::cast_slice(&geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shape Index Buffer"), // Combined buffer
        contents: bytemuck::cast_slice(&geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let index_count = geometry.indices.len() as u32;

    // --- Render Pass ---
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

        // Draw Shapes (Rectangles, Circles, Lines)
        if index_count > 0 {
            rpass.set_pipeline(&gfx.rect_pipeline);
            rpass.set_bind_group(0, &gfx.bind_group, &[]); // Set uniforms
            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..index_count, 0, 0..1);
        }

        // Render text on top
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

    // Uniforms for canvas size
    uniform_buffer: WgpuBuffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,

    // Only store the pipeline for rectangles/shapes
    rect_pipeline: wgpu::RenderPipeline,
}

// Create a shaders directory and the rectangle shader file
// src/shaders/rectangle.wgsl
