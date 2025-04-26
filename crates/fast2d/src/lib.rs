// --- Compile-time checks for mutually exclusive features ---

// Error if multiple rendering backends are selected
#[cfg(all(feature = "webgl", feature = "webgpu"))]
compile_error!("Feature 'webgl' and 'webgpu' cannot be enabled at the same time. Please choose one rendering backend.");

#[cfg(all(feature = "webgl", feature = "canvas"))]
compile_error!("Feature 'webgl' and 'canvas' cannot be enabled at the same time. Please choose one rendering backend.");

#[cfg(all(feature = "webgpu", feature = "canvas"))]
compile_error!("Feature 'webgpu' and 'canvas' cannot be enabled at the same time. Please choose one rendering backend.");

// Error if no rendering backend is selected
#[cfg(not(any(feature = "webgl", feature = "webgpu", feature = "canvas")))]
compile_error!("One rendering backend feature ('webgl', 'webgpu', or 'canvas') must be enabled.");

// --- End of compile-time checks ---

use web_sys::{HtmlCanvasElement, wasm_bindgen::UnwrapThrowExt};

use std::borrow::Cow;

// Import lyon types
use lyon::math::point;
use lyon::path::{Path, Winding}; // Added Path back
use lyon::path::builder::BorderRadii;
use lyon::math::Box2D;
use lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder, StrokeTessellator, StrokeOptions, StrokeVertex, LineCap, LineJoin};

// Import wgpu types
use wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture, BindGroupLayout, BindGroup, Buffer as WgpuBuffer};
use wgpu::util::DeviceExt;

use std::sync::{OnceLock, Mutex}; // Import Mutex

// Import glyphon types
use glyphon::{
    Cache, FontSystem, Shaping, Buffer as GlyphonBuffer,
    SwashCache, TextAtlas, TextRenderer, Viewport, TextArea,
    Attrs, Color as GlyphonColor, TextBounds, Resolution, FamilyOwned, Metrics // Removed CacheKeyFlags
};

// Re-export Family to avoid direct dependency on glyphon in client code
pub use glyphon::Family;
// Removed: pub use object_2d::types::FamilyOwned;

// Define MSAA_SAMPLE_COUNT constant here
const MSAA_SAMPLE_COUNT: u32 = 4; // Multisampling for anti-aliasing

// --- Modify FontSystem static ---
// Wrap FontSystem in a Mutex to allow mutable access
static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

#[derive(Debug)]
pub enum FontSystemInitError {
    DatabaseError(String), // Error loading fonts into the database
    AlreadyInitialized,
    NoFontsProvided,
}

/// Initializes the cosmic-text FontSystem with provided font data.
/// Must be called once before text rendering.
pub fn init_font_system(font_data: Vec<&'static [u8]>) -> Result<(), FontSystemInitError> {
    if font_data.is_empty() {
        return Err(FontSystemInitError::NoFontsProvided);
    }

    // Create a FontSystem (adjust locale/db as needed)
    let mut font_system = FontSystem::new();
    let db = font_system.db_mut();
    for data in font_data {
        // Loading might return errors, collect them or handle appropriately
        db.load_font_data(data.to_vec()); // Consider error handling here
    }
    // You might want more robust error checking on font loading

    // Wrap the initialized font_system in a Mutex before setting
    FONT_SYSTEM.set(Mutex::new(font_system))
        .map_err(|_| FontSystemInitError::AlreadyInitialized)
}

// --- Remove Internal helper to get FontSystem ---
// fn get_font_system() -> &'static FontSystem { ... }
// Access will now be done via FONT_SYSTEM.get().unwrap().lock().unwrap() within draw

// Declare the object_2d module and re-export structs
mod object_2d;
pub use object_2d::text::Text;
pub use object_2d::rectangle::Rectangle;
pub use object_2d::circle::Circle;
pub use object_2d::line::Line;

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
        // Ensure width and height are at least 1
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        self.canvas = Some(canvas.clone());
        // Pass width and height to create_graphics
        self.graphics = Some(create_graphics(canvas, width, height).await);
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

            // Update viewport with Resolution struct
            graphics.viewport.update(
                &graphics.queue,
                Resolution { width: new_width, height: new_height }
            );

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


async fn create_graphics(canvas: HtmlCanvasElement, width: u32, height: u32) -> Graphics {
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


    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width, // Use passed width
        height, // Use passed height
        present_mode: surface_caps.present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);


    // --- Uniform Buffer Setup ---
    let uniforms = CanvasUniforms {
        width: width as f32, // Use passed width
        height: height as f32, // Use passed height
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


    // Create multisample texture using passed dimensions
    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("MSAA Texture"),
        size: wgpu::Extent3d {
            width, // Use passed width
            height, // Use passed height
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: surface_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    // --- Text Rendering Setup ---
    // Remove font_system retrieval here; it will be accessed in draw()
    // let font_system = get_font_system();

    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    // Create Viewport using ::new
    let mut viewport = Viewport::new(&device, &cache);
    // Update viewport with initial resolution immediately after creation
    viewport.update(&queue, Resolution { width, height });
    let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
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

    // --- Shape Pipeline Setup ---
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shape Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/rectangle.wgsl"))),
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
                format: surface_format,
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
        msaa_texture,

        // Remove font_system field
        // font_system,
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


// Update draw function
fn draw(gfx: &mut Graphics, objects: &[Object2d]) {
    let output = match gfx.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(e) => {
            eprintln!("Error getting current texture: {:?}", e);
            if e == wgpu::SurfaceError::Lost {
                 gfx.surface.configure(&gfx.device, &gfx.surface_config);
                 return;
            }
            return;
        }
    };
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let msaa_view = gfx.msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());


    // --- Text Preparation ---
    // Acquire mutable lock on the global FontSystem
    let mut font_system = FONT_SYSTEM.get()
        .expect("FontSystem not initialized")
        .lock()
        .expect("Failed to lock FontSystem Mutex");

    // Stage 1: Create and store all owned buffers
    let mut glyph_buffers: Vec<GlyphonBuffer> = Vec::new();
    for obj in objects {
        if let Object2d::Text(text) = obj {
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;

            // Calculate absolute line height in pixels
            let line_height_pixels = text.font_size * text.line_height_multiplier;

            let mut buffer = GlyphonBuffer::new(
                &mut font_system,
                // Pass calculated absolute line height
                Metrics::new(text.font_size, line_height_pixels),
            );

            // Set buffer size - Pass mutable reference from lock guard
            buffer.set_size(
                &mut font_system,
                Some(text_width_f32),
                Some(text_height_f32),
            );

            // Match on FamilyOwned - Access family field
            let family_name = match &text.family {
                FamilyOwned::Name(name) => Family::Name(name.as_str()),
                _ => Family::SansSerif,
            };

            // Get color using named fields and convert to u8
            let glyphon_color = GlyphonColor::rgba(
                (text.color.r * 255.0).clamp(0.0, 255.0) as u8, // Use .r field
                (text.color.g * 255.0).clamp(0.0, 255.0) as u8, // Use .g field
                (text.color.b * 255.0).clamp(0.0, 255.0) as u8, // Use .b field
                (text.color.a * 255.0).clamp(0.0, 255.0) as u8, // Use .a field
            );

            // Set text on the buffer - Access text field
            buffer.set_text(
                &mut font_system,
                &text.text, // Access text field
                &Attrs::new()
                    .family(family_name)
                    .color(glyphon_color),
                Shaping::Advanced,
            );

            glyph_buffers.push(buffer);
        }
    }

    // Stage 2: Create TextAreas borrowing from the owned buffers
    let mut text_areas: Vec<TextArea> = Vec::new();
    let mut buffer_idx = 0;
    for obj in objects {
        if let Object2d::Text(text) = obj {
            // Get color using named fields and convert to u8
            let glyphon_color = GlyphonColor::rgba(
                (text.color.r * 255.0).clamp(0.0, 255.0) as u8, // Use .r field
                (text.color.g * 255.0).clamp(0.0, 255.0) as u8, // Use .g field
                (text.color.b * 255.0).clamp(0.0, 255.0) as u8, // Use .b field
                (text.color.a * 255.0).clamp(0.0, 255.0) as u8, // Use .a field
            );
            // Access width and height fields
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;

            // Construct TextArea using a reference to the buffer in the vector
            let text_area = TextArea {
                buffer: &glyph_buffers[buffer_idx],
                // Access left and top fields
                left: text.left,
                top: text.top,
                bounds: TextBounds {
                     // Access left and top fields
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

    // Prepare text renderer using TextAreas - Pass mutable reference from lock guard
    match gfx.text_renderer.prepare(
        &gfx.device,
        &gfx.queue,
        &mut font_system, // Pass mutable reference
        &mut gfx.atlas,
        &gfx.viewport,
        text_areas.into_iter(), // Pass owned TextAreas
        &mut gfx.swash_cache,
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error preparing text renderer: {:?}", e);
        }
    }
    // MutexGuard `font_system` is dropped here, releasing the lock

    // --- Shape Tessellation ---
    let mut buffers: VertexBuffers<ColoredVertex, u32> = VertexBuffers::new();
    // Removed unused fill_builder and stroke_builder declarations
    // BuffersBuilder is created inline below when tessellating

    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    for obj in objects {
        match obj {
            Object2d::Rectangle(rect) => {
                let color = [
                    rect.color.r as f32,
                    rect.color.g as f32,
                    rect.color.b as f32,
                    rect.color.a as f32,
                ];
                let mut builder = Path::builder(); // Use imported Path
                // Use position and size fields
                let rect_box = Box2D::new(
                     point(rect.position.x, rect.position.y),
                     point(rect.position.x + rect.size.width, rect.position.y + rect.size.height),
                 );

                // Use border_radii fields
                if rect.border_radii.top_left > 0.0 || rect.border_radii.top_right > 0.0 || rect.border_radii.bottom_left > 0.0 || rect.border_radii.bottom_right > 0.0 {
                     builder.add_rounded_rectangle(
                         &rect_box,
                         // Use border_radii fields directly
                         &BorderRadii {
                             top_left: rect.border_radii.top_left,
                             top_right: rect.border_radii.top_right,
                             bottom_left: rect.border_radii.bottom_left,
                             bottom_right: rect.border_radii.bottom_right,
                         },
                         Winding::Positive,
                     );
                } else {
                    builder.add_rectangle(
                        &rect_box,
                        Winding::Positive,
                    );
                }
                let path = builder.build();

                // Fill
                if rect.color.a > 0.0 {
                    fill_tessellator.tessellate_path(
                        &path,
                        &FillOptions::default(),
                        // Inline BuffersBuilder creation
                        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                             ColoredVertex {
                                 position: [vertex.position().x, vertex.position().y],
                                 color,
                             }
                         }),
                    ).unwrap();
                }

                // Correctly handle Option for border_width and border_color
                if let (Some(border_width), Some(border_color_val)) = (rect.border_width, rect.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let border_color = [
                            border_color_val.r as f32,
                            border_color_val.g as f32,
                            border_color_val.b as f32,
                            border_color_val.a as f32,
                        ];
                        let options = StrokeOptions::default()
                            .with_line_width(border_width); // Use unwrapped border_width

                        stroke_tessellator.tessellate_path(
                            &path,
                            &options,
                            // Inline BuffersBuilder creation
                             &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| {
                                  ColoredVertex {
                                      position: [vertex.position().x, vertex.position().y],
                                      color: border_color,
                                  }
                              }),
                        ).unwrap();
                    }
                }
            }
            Object2d::Circle(circle) => {
                 let color = [
                     circle.color.r as f32,
                     circle.color.g as f32,
                     circle.color.b as f32,
                     circle.color.a as f32,
                 ];
                 let mut builder = Path::builder(); // Use imported Path
                 // Use circle.center.x and circle.center.y
                 builder.add_circle(
                     point(circle.center.x, circle.center.y),
                     circle.radius,
                     Winding::Positive,
                 );
                 let path = builder.build();

                 // Fill
                 if circle.color.a > 0.0 {
                     fill_tessellator.tessellate_path(
                         &path,
                         &FillOptions::default(),
                         // Inline BuffersBuilder creation
                         &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                              ColoredVertex {
                                  position: [vertex.position().x, vertex.position().y],
                                  color,
                              }
                          }),
                     ).unwrap();
                 }

                 // Correctly handle Option for border_width and border_color
                 if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                     if border_width > 0.0 && border_color_val.a > 0.0 {
                         let border_color = [
                             border_color_val.r as f32,
                             border_color_val.g as f32,
                             border_color_val.b as f32,
                             border_color_val.a as f32,
                         ];
                         let options = StrokeOptions::default()
                             .with_line_width(border_width); // Use unwrapped border_width

                         stroke_tessellator.tessellate_path(
                             &path,
                             &options,
                             // Inline BuffersBuilder creation
                              &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| {
                                   ColoredVertex {
                                       position: [vertex.position().x, vertex.position().y],
                                       color: border_color,
                                   }
                               }),
                         ).unwrap();
                     }
                 }
            }
            Object2d::Line(line) => {
                 let color = [
                     line.color.r as f32,
                     line.color.g as f32,
                     line.color.b as f32,
                     line.color.a as f32,
                 ];
                 let mut builder = Path::builder();
                 // Assuming line.points is Vec<SomePointType { x: f32, y: f32 }>
                 // Check if there are at least two points
                 if line.points.len() >= 2 {
                     // Use .x and .y fields for point()
                     builder.begin(point(line.points[0].x, line.points[0].y));
                     // Iterate starting from the second point
                     for i in 1..line.points.len() {
                         // Use .x and .y fields for point()
                         builder.line_to(point(line.points[i].x, line.points[i].y));
                     }
                     builder.end(false);
                 }
                 let path = builder.build();

                 // Check again if there were enough points to build a valid path for tessellation
                 if line.points.len() >= 2 {
                    let options = StrokeOptions::default()
                        .with_line_width(line.width)
                        .with_line_cap(LineCap::Round)
                        .with_line_join(LineJoin::Round);

                    stroke_tessellator.tessellate_path(
                        &path,
                        &options,
                        // Inline BuffersBuilder creation
                         &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| {
                              ColoredVertex {
                                  position: [vertex.position().x, vertex.position().y],
                                  color,
                              }
                          }),
                    ).unwrap();
                 }
            }
            Object2d::Text(_) => {
                // Text is handled separately by glyphon
            }
        }
    }


    // --- Buffer Creation and Render Pass ---
    let vertex_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&buffers.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&buffers.indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = buffers.indices.len() as u32;


    let mut encoder = gfx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        // Use MSAA view for drawing shapes and text
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_view, // Render to MSAA texture
                resolve_target: Some(&view), // Resolve MSAA to the final view
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0, g: 0.0, b: 0.0, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Draw shapes (if any indices exist)
        if num_indices > 0 {
            render_pass.set_pipeline(&gfx.rect_pipeline);
            render_pass.set_bind_group(0, &gfx.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        // Render text using the prepared renderer
        match gfx.text_renderer.render(&gfx.atlas, &gfx.viewport, &mut render_pass) {
             Ok(_) => {}
             Err(e) => {
                 eprintln!("Error rendering text: {:?}", e);
             }
         }

    } // render_pass is dropped here

    gfx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

// ... (resize and other functions) ...

#[allow(dead_code)]
struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    msaa_texture: Texture,

    // Remove font_system field
    // font_system: &'static FontSystem,
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

// Example function demonstrating conditional backend logic
pub fn initialize_renderer() {
    #[cfg(feature = "canvas")]
    {
        // --- Canvas Backend Initialization ---
        println!("Initializing Fast2D with Canvas backend.");
        // Get canvas element, get 2D context using web-sys
        // Store context for drawing operations
        // ... canvas-specific setup ...

        // Example using web-sys (ensure web-sys is imported)
        /*
        use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document.get_element_by_id("fast2d-canvas") // Assuming a canvas with this ID exists
            .expect("should have canvas element")
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .expect("element should be a HtmlCanvasElement");

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        context.set_fill_style(&"blue".into());
        context.fill_rect(10.0, 10.0, 100.0, 100.0);
        */
    }

    #[cfg(not(feature = "canvas"))]
    {
        // --- WGPU/WebGL Backend Initialization ---
        println!("Initializing Fast2D with WGPU/WebGL backend.");
        // Initialize wgpu, create surface, adapter, device, queue
        // ... wgpu-specific setup ...
    }
}

pub fn draw_frame() {
    #[cfg(feature = "canvas")]
    {
        // --- Canvas Drawing Logic ---
        // Use the stored CanvasRenderingContext2d to draw shapes, images, etc.
        // ... canvas drawing commands ...
    }

    #[cfg(not(feature = "canvas"))]
    {
        // --- WGPU/WebGL Drawing Logic ---
        // Create command encoder, render pass, set pipelines, draw calls, submit queue
        // ... wgpu drawing commands ...
    }
}

// ... rest of the library code ...
