// --- Compile-time checks for mutually exclusive features ---

// Error if more than one rendering backend is selected
#[cfg(any(
    all(feature = "webgl", feature = "webgpu"),
    all(feature = "webgl", feature = "canvas"),
    all(feature = "webgpu", feature = "canvas")
))]
compile_error!("Only one rendering backend feature ('webgl', 'webgpu', or 'canvas') can be enabled at a time.");

// Error if no rendering backend is selected
#[cfg(not(any(feature = "webgl", feature = "webgpu", feature = "canvas")))]
compile_error!("One rendering backend feature ('webgl', 'webgpu', or 'canvas') must be enabled.");

// --- End of compile-time checks ---

// Remove JsCast from the import
use web_sys::{HtmlCanvasElement, wasm_bindgen::UnwrapThrowExt};
use cfg_if::cfg_if; // Use for conditional fields/logic

// --- Conditional Imports ---
#[cfg(feature = "canvas")]
use web_sys::{CanvasRenderingContext2d, wasm_bindgen::JsCast}; // Add JsCast here

#[cfg(not(feature = "canvas"))]
use {
    std::borrow::Cow,
    // Use shared Point type instead of lyon::math::point directly in tessellation if possible,
    // or convert within draw_wgpu. For now, keep lyon imports needed for tessellation.
    lyon::math::point, // Keep for tessellation for now
    lyon::path::{Path, Winding},
    lyon::path::builder::BorderRadii as LyonBorderRadii, // Alias lyon's BorderRadii
    lyon::math::Box2D,
    lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder, StrokeTessellator, StrokeOptions, StrokeVertex, LineCap, LineJoin},
    // Remove Color as WgpuColor
    wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture, BindGroupLayout, BindGroup, Buffer as WgpuBuffer},
    wgpu::util::DeviceExt,
    std::sync::{OnceLock, Mutex},
    // Remove FamilyOwned as GlyphonFamilyOwned
    glyphon::{
        Cache, FontSystem, Shaping, Buffer as GlyphonBuffer,
        SwashCache, TextAtlas, TextRenderer, Viewport, TextArea,
        Attrs, Color as GlyphonColor, TextBounds, Resolution, Metrics, Family as GlyphonFamily // Import GlyphonFamily
    },
    bytemuck, // Import the crate itself
};

// --- Conditional Re-exports/Constants/Statics ---
// Remove glyphon::Family re-export, use shared types instead
// #[cfg(not(feature = "canvas"))]
// pub use glyphon::Family;

#[cfg(not(feature = "canvas"))]
const MSAA_SAMPLE_COUNT: u32 = 4;

#[cfg(not(feature = "canvas"))]
static FONT_SYSTEM: OnceLock<Mutex<FontSystem>> = OnceLock::new();

#[cfg(not(feature = "canvas"))]
#[derive(Debug)]
pub enum FontSystemInitError {
    DatabaseError(String),
    AlreadyInitialized,
    NoFontsProvided,
}

#[cfg(not(feature = "canvas"))]
/// Initializes the cosmic-text FontSystem (WGPU/WebGL only).
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

// --- Conditional Structs ---
#[cfg(not(feature = "canvas"))]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    width: f32,
    height: f32,
    _padding1: f32,
    _padding2: f32,
}

#[cfg(not(feature = "canvas"))]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ColoredVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[cfg(not(feature = "canvas"))]
impl ColoredVertex {
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

#[cfg(not(feature = "canvas"))]
#[allow(dead_code)]
struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    msaa_texture: Texture,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    uniform_buffer: WgpuBuffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    rect_pipeline: wgpu::RenderPipeline,
}

// --- Shared Structs/Enums ---
// Declare the object_2d module and re-export structs (shared)
mod object_2d;
pub use object_2d::text::Text;
pub use object_2d::rectangle::Rectangle;
pub use object_2d::circle::Circle;
pub use object_2d::line::Line;
pub use object_2d::types::{Color, Point, Size, BorderRadii as ObjBorderRadii}; // Re-export shared types

#[cfg(not(feature = "canvas"))]
pub use object_2d::FamilyOwned; // Re-export conditionally

// Enum definition remains here (shared)
#[derive(Debug, Clone)]
pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Line),
}

// --- CanvasWrapper with Conditional Backend ---
pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas_element: Option<HtmlCanvasElement>, // Store the element itself

    // Conditional backend state
    #[cfg(feature = "canvas")]
    context: Option<CanvasRenderingContext2d>,
    #[cfg(not(feature = "canvas"))]
    graphics: Option<Graphics>,
}

impl CanvasWrapper {
    pub fn new() -> Self {
        cfg_if! {
            if #[cfg(feature = "canvas")] {
                Self {
                    objects: Vec::new(),
                    canvas_element: None,
                    context: None,
                }
            } else {
                Self {
                    objects: Vec::new(),
                    canvas_element: None,
                    graphics: None,
                }
            }
        }
    }

    // Suppress unused variable warnings only when 'canvas' feature is active
    #[cfg_attr(feature = "canvas", allow(unused_variables))]
    pub async fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        self.canvas_element = Some(canvas.clone());

        cfg_if! {
            if #[cfg(feature = "canvas")] {
                // Get 2D rendering context
                let context_object = canvas
                    .get_context("2d")
                    .unwrap_throw() // Handle potential errors
                    .unwrap_throw() // Handle Option<Object>
                    .dyn_into::<CanvasRenderingContext2d>() // JsCast is now in scope via conditional import
                    .unwrap_throw(); // Handle incorrect type
                self.context = Some(context_object);
                println!("Fast2D: Initialized with Canvas backend.");
            } else {
                // Initialize WGPU graphics (uses width, height)
                self.graphics = Some(create_graphics(canvas, width, height).await);
                 println!("Fast2D: Initialized with WGPU/WebGL backend.");
            }
        }
        self.draw(); // Initial draw
    }

    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        self.draw();
    }

    // Suppress unused variable warnings only when 'canvas' feature is active
    #[cfg_attr(feature = "canvas", allow(unused_variables))]
    pub fn resized(&mut self, width: u32, height: u32) {
        cfg_if! {
            if #[cfg(feature = "canvas")] {
                // For canvas, resizing the element externally is enough.
                // We just need to redraw.
                self.draw();
            } else {
                // WGPU requires reconfiguration (uses width, height)
                if let Some(graphics) = &mut self.graphics {
                    let new_width = width.max(1);
                    let new_height = height.max(1);

                    graphics.surface_config.width = new_width;
                    graphics.surface_config.height = new_height;
                    graphics.surface.configure(&graphics.device, &graphics.surface_config);

                    // Recreate MSAA texture
                    graphics.msaa_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("MSAA Texture"),
                        size: wgpu::Extent3d { width: new_width, height: new_height, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: MSAA_SAMPLE_COUNT,
                        dimension: wgpu::TextureDimension::D2,
                        format: graphics.surface_config.format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });

                    // Update viewport
                    graphics.viewport.update(&graphics.queue, Resolution { width: new_width, height: new_height });

                    // Update uniform buffer
                    let uniforms = CanvasUniforms { width: new_width as f32, height: new_height as f32, _padding1: 0.0, _padding2: 0.0 };
                    graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

                    self.draw(); // Redraw after WGPU resize
                }
            }
        }
    }

    fn draw(&mut self) {
        cfg_if! {
            if #[cfg(feature = "canvas")] {
                if let Some(context) = &self.context {
                    if let Some(canvas) = &self.canvas_element {
                         // Clear canvas before drawing
                         context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
                         draw_canvas(context, &self.objects);
                    }
                }
            } else {
                if let Some(graphics) = &mut self.graphics {
                    draw_wgpu(graphics, &self.objects); // Call the WGPU draw function
                }
            }
        }
    }
}

// --- Canvas Backend Implementation ---
#[cfg(feature = "canvas")]
fn draw_canvas(ctx: &CanvasRenderingContext2d, objects: &[Object2d]) {
    // Set default state (optional, but good practice)
    // Use correct non-deprecated methods
    ctx.set_fill_style_str("black"); // Default fill
    ctx.set_stroke_style_str("black"); // Default stroke
    ctx.set_line_width(1.0);

    for obj in objects {
        match obj {
            Object2d::Rectangle(rect) => {
                // Set fill color
                let fill_color = rect.color.to_canvas_rgba();
                // Use correct non-deprecated method
                ctx.set_fill_style_str(&fill_color);

                // TODO: Add rounded rectangle support if radii > 0 using path API
                // For now, just draw a simple rectangle
                ctx.fill_rect(
                    rect.position.x as f64,
                    rect.position.y as f64,
                    rect.size.width as f64,
                    rect.size.height as f64,
                );

                // Handle border
                if let (Some(border_width), Some(border_color_val)) = (rect.border_width, rect.border_color) {
                     if border_width > 0.0 && border_color_val.a > 0.0 {
                         let stroke_color = border_color_val.to_canvas_rgba();
                         // Use correct non-deprecated method
                         ctx.set_stroke_style_str(&stroke_color);
                         ctx.set_line_width(border_width as f64);
                         ctx.stroke_rect(
                             rect.position.x as f64,
                             rect.position.y as f64,
                             rect.size.width as f64,
                             rect.size.height as f64,
                         );
                     }
                 }
            }
            Object2d::Circle(circle) => {
                ctx.begin_path();
                ctx.arc(
                    circle.center.x as f64,
                    circle.center.y as f64,
                    circle.radius as f64,
                    0.0,
                    std::f64::consts::PI * 2.0,
                ).unwrap_throw(); // Error handling for arc

                // Fill
                if circle.color.a > 0.0 {
                    let fill_color = circle.color.to_canvas_rgba();
                    // Use correct non-deprecated method
                    ctx.set_fill_style_str(&fill_color);
                    ctx.fill();
                }

                // Border
                if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                     if border_width > 0.0 && border_color_val.a > 0.0 {
                         let stroke_color = border_color_val.to_canvas_rgba();
                         // Use correct non-deprecated method
                         ctx.set_stroke_style_str(&stroke_color);
                         ctx.set_line_width(border_width as f64);
                         ctx.stroke(); // Stroke the path defined by arc
                     }
                 }
                 // ctx.close_path(); // Not strictly necessary after fill/stroke for a circle arc
            }
            Object2d::Line(line) => {
                 if line.points.len() >= 2 && line.color.a > 0.0 {
                     let stroke_color = line.color.to_canvas_rgba();
                     // Use correct non-deprecated method
                     ctx.set_stroke_style_str(&stroke_color);
                     ctx.set_line_width(line.width as f64);
                     // TODO: Set line cap/join if needed (ctx.set_line_cap, ctx.set_line_join)

                     ctx.begin_path();
                     ctx.move_to(line.points[0].x as f64, line.points[0].y as f64);
                     for i in 1..line.points.len() {
                         ctx.line_to(line.points[i].x as f64, line.points[i].y as f64);
                     }
                     ctx.stroke(); // Stroke the defined path
                 }
            }
            Object2d::Text(text) => {
                if text.color.a > 0.0 {
                     let fill_color = text.color.to_canvas_rgba();
                     // Use correct non-deprecated method
                     ctx.set_fill_style_str(&fill_color);

                     // Use the family string directly
                     let font_str = format!("{}px {}", text.font_size, text.family);
                     ctx.set_font(&font_str);

                     // Simple fillText - doesn't handle wrapping or line height multiplier
                     ctx.fill_text(&text.text, text.left as f64, text.top as f64 + text.font_size as f64) // Adjust baseline
                        .unwrap_throw();
                 }
            }
        }
    }
}

// --- WGPU/WebGL Backend Implementation ---
#[cfg(not(feature = "canvas"))]
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
        view_formats: vec![], // Use vec![] or Vec::new()
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
    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let mut viewport = Viewport::new(&device, &cache);
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

#[cfg(not(feature = "canvas"))]
// Renamed from draw to draw_wgpu
fn draw_wgpu(gfx: &mut Graphics, objects: &[Object2d]) {
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
    let mut font_system = FONT_SYSTEM.get()
        .expect("FontSystem not initialized")
        .lock()
        .expect("Failed to lock FontSystem Mutex");

    let mut glyph_buffers: Vec<GlyphonBuffer> = Vec::new();
    for obj in objects {
        if let Object2d::Text(text) = obj {
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let line_height_pixels = text.font_size * text.line_height_multiplier;
            let mut buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(text.font_size, line_height_pixels));
            buffer.set_size(&mut font_system, Some(text_width_f32), Some(text_height_f32));
            let family_name = match &text.family {
                FamilyOwned::Name(name) => GlyphonFamily::Name(name.as_str()),
                _ => GlyphonFamily::SansSerif,
            };
            let glyphon_color = GlyphonColor::rgba(
                (text.color.r * 255.0).clamp(0.0, 255.0) as u8,
                (text.color.g * 255.0).clamp(0.0, 255.0) as u8,
                (text.color.b * 255.0).clamp(0.0, 255.0) as u8,
                (text.color.a * 255.0).clamp(0.0, 255.0) as u8,
            );
            buffer.set_text(&mut font_system, &text.text, &Attrs::new().family(family_name).color(glyphon_color), Shaping::Advanced);
            glyph_buffers.push(buffer);
        }
    }

    let mut text_areas: Vec<TextArea> = Vec::new();
    let mut buffer_idx = 0;
    for obj in objects {
        if let Object2d::Text(text) = obj {
            let glyphon_color = GlyphonColor::rgba(
                (text.color.r * 255.0).clamp(0.0, 255.0) as u8,
                (text.color.g * 255.0).clamp(0.0, 255.0) as u8,
                (text.color.b * 255.0).clamp(0.0, 255.0) as u8,
                (text.color.a * 255.0).clamp(0.0, 255.0) as u8,
            );
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
        Err(e) => eprintln!("Error preparing text renderer: {:?}", e),
    }

    // --- Shape Tessellation ---
    let mut buffers: VertexBuffers<ColoredVertex, u32> = VertexBuffers::new();
    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    for obj in objects {
        match obj {
            Object2d::Rectangle(rect) => {
                let color = [rect.color.r as f32, rect.color.g as f32, rect.color.b as f32, rect.color.a as f32];
                let mut builder = Path::builder();
                let rect_box = Box2D::new(point(rect.position.x, rect.position.y), point(rect.position.x + rect.size.width, rect.position.y + rect.size.height));
                if rect.border_radii.top_left > 0.0 || rect.border_radii.top_right > 0.0 || rect.border_radii.bottom_left > 0.0 || rect.border_radii.bottom_right > 0.0 {
                     builder.add_rounded_rectangle(&rect_box, &LyonBorderRadii { top_left: rect.border_radii.top_left, top_right: rect.border_radii.top_right, bottom_left: rect.border_radii.bottom_left, bottom_right: rect.border_radii.bottom_right }, Winding::Positive);
                } else {
                    builder.add_rectangle(&rect_box, Winding::Positive);
                }
                let path = builder.build();
                if rect.color.a > 0.0 {
                    fill_tessellator.tessellate_path(&path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color })).unwrap();
                }
                if let (Some(border_width), Some(border_color_val)) = (rect.border_width, rect.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let border_color = [border_color_val.r as f32, border_color_val.g as f32, border_color_val.b as f32, border_color_val.a as f32];
                        let options = StrokeOptions::default().with_line_width(border_width);
                        stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: border_color })).unwrap();
                    }
                }
            }
            Object2d::Circle(circle) => {
                 let color = [circle.color.r as f32, circle.color.g as f32, circle.color.b as f32, circle.color.a as f32];
                 let mut builder = Path::builder();
                 builder.add_circle(point(circle.center.x, circle.center.y), circle.radius, Winding::Positive);
                 let path = builder.build();
                 if circle.color.a > 0.0 {
                     fill_tessellator.tessellate_path(&path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color })).unwrap();
                 }
                 if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                     if border_width > 0.0 && border_color_val.a > 0.0 {
                         let border_color = [border_color_val.r as f32, border_color_val.g as f32, border_color_val.b as f32, border_color_val.a as f32];
                         let options = StrokeOptions::default().with_line_width(border_width);
                         stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: border_color })).unwrap();
                     }
                 }
            }
            Object2d::Line(line) => {
                 let color = [line.color.r as f32, line.color.g as f32, line.color.b as f32, line.color.a as f32];
                 let mut builder = Path::builder();
                 if line.points.len() >= 2 {
                     builder.begin(point(line.points[0].x, line.points[0].y));
                     for i in 1..line.points.len() {
                         builder.line_to(point(line.points[i].x, line.points[i].y));
                     }
                     builder.end(false);
                 }
                 let path = builder.build();
                 if line.points.len() >= 2 {
                    let options = StrokeOptions::default().with_line_width(line.width).with_line_cap(LineCap::Round).with_line_join(LineJoin::Round);
                    stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color })).unwrap();
                 }
            }
            Object2d::Text(_) => {} // Handled by glyphon
        }
    }

    // --- Buffer Creation and Render Pass ---
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
                view: &msaa_view, resolve_target: Some(&view),
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
             Err(e) => eprintln!("Error rendering text: {:?}", e),
         }
    }
    gfx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
