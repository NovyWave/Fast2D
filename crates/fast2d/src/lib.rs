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

// Correct the import for console
use web_sys::wasm_bindgen::UnwrapThrowExt;
#[cfg(feature = "canvas")]
use web_sys::CanvasRenderingContext2d;

// --- Conditional Imports ---

#[cfg(any(feature = "webgl", feature = "webgpu"))]
use {
    lyon::path::{Path, Winding},
    lyon::path::builder::BorderRadii as LyonBorderRadii, // Alias lyon's BorderRadii
    lyon::math::Box2D,
    lyon::tessellation::{FillTessellator, FillOptions, VertexBuffers, FillVertex, BuffersBuilder, StrokeTessellator, StrokeOptions, StrokeVertex, LineCap, LineJoin},
    // Remove Color as WgpuColor
    wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget, Texture, BindGroupLayout, BindGroup, Buffer as WgpuBuffer, TextureViewDescriptor},
    wgpu::util::DeviceExt,
    // Remove OnceLock, Mutex, FontSystem from here (they are now imported only where needed)
    // Remove FamilyOwned as GlyphonFamilyOwned
    glyphon::{
        Cache, Shaping, Buffer as GlyphonBuffer,
        SwashCache, TextAtlas, TextRenderer, Viewport, TextArea,
        Attrs, TextBounds, Resolution, Metrics, Family as GlyphonFamily, // Import GlyphonFamily
        ColorMode // Import ColorMode
    },
    bytemuck, // Import the crate itself
    web_sys::HtmlCanvasElement,
    web_sys::console,
    web_sys::wasm_bindgen::JsValue,
    lyon::math::point,
};

// --- Conditional Re-exports/Constants/Statics ---
// Remove glyphon::Family re-export, use shared types instead
// #[cfg(not(feature = "canvas"))]
// pub use glyphon::Family;

#[cfg(any(feature = "webgl", feature = "webgpu"))]
const MSAA_SAMPLE_COUNT: u32 = 4;

#[cfg(any(feature = "webgl", feature = "webgpu"))]
pub static FONT_SYSTEM: std::sync::OnceLock<std::sync::Mutex<glyphon::FontSystem>> = std::sync::OnceLock::new();

#[cfg(any(feature = "webgl", feature = "webgpu"))]
#[derive(Debug)]
pub enum FontSystemInitError {
    DatabaseError(String),
    AlreadyInitialized,
    NoFontsProvided,
}

// --- Conditional Structs ---
#[cfg(any(feature = "webgl", feature = "webgpu"))]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    width: f32,
    height: f32,
    _padding1: f32,
    _padding2: f32,
}

#[cfg(any(feature = "webgl", feature = "webgpu"))]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ColoredVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[cfg(any(feature = "webgl", feature = "webgpu"))]
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

#[cfg(any(feature = "webgl", feature = "webgpu"))]
#[allow(dead_code)]
struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    is_srgb: bool, // Add flag for surface format
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
pub use object_2d::types::Family;
pub use crate::object_2d::text::FontWeight;

#[cfg(any(feature = "webgl", feature = "webgpu"))]
pub use object_2d::FamilyOwned; // Re-export conditionally

// Enum definition remains here (shared)
#[derive(Debug, Clone)]
pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Line),
}

// --- CanvasWrapper moved to canvas_wrapper.rs ---
mod canvas_wrapper;
pub use canvas_wrapper::CanvasWrapper;
mod register_fonts;
pub use register_fonts::register_fonts;

// --- WGPU/WebGL Backend Implementation ---
#[cfg(any(feature = "webgl", feature = "webgpu"))]
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

    // --- REMOVE Get Adapter Features --- <<<<<<<<<<<< REMOVED
    // let features = adapter.features();
    // let supports_view_formats = features.contains(wgpu::Features::SURFACE_VIEW_FORMATS);
    // console::log_1(&JsValue::from_str(&format!(
    //     "Fast2D: Adapter supports SURFACE_VIEW_FORMATS: {}",
    //     supports_view_formats
    // )));

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WGPU Device"),
                memory_hints: wgpu::MemoryHints::default(),
                // Remove conditional feature request
                required_features: wgpu::Features::empty(), // Request no extra features for simplicity now
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
    
    // Force linear (non-sRGB) format for the surface
    let preferred_linear_formats = [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Bgra8Unorm,
    ];
    let surface_format = preferred_linear_formats.iter()
        .copied()
        .find(|format| surface_caps.formats.contains(format))
        .unwrap_or(surface_caps.formats[0]);
    let is_srgb = false; // Always treat as linear

    // --- REMOVE View Formats Logic (Conditional) --- <<<<<<<<<<<< REMOVED
    // let mut config_view_formats = vec![];
    // let mut texture_view_formats_slice: Vec<wgpu::TextureFormat> = vec![];
    let target_format = surface_format; // Target format must match surface format
    // console::log_1(&JsValue::from_str(&format!(
    //     "Fast2D: Pipeline target and Surface config: {:?}",
    //     target_format
    // )));
    // --- End REMOVAL ---

    // Log the chosen format
    // console::log_1(&JsValue::from_str(&format!(
    //     "Fast2D: Using surface format ({:?}). sRGB={}",
    //     surface_format, is_srgb // Use stored value
    // )));

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format, // Use the chosen sRGB (or fallback) format for the surface itself
        width,
        height,
        present_mode: surface_caps.present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0], // Use default alpha mode
        view_formats: vec![], // Set to empty as feature is not used/available <<<<<<<<<<<< REVERTED
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


    // --- MSAA Texture --- <<<<<<<<<<<< REVERTED
    // Create multisample texture using the surface_format, but allow viewing with all formats
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
        format: surface_format, // Create with the surface format
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[], // Set to empty <<<<<<<<<<<< REVERTED
    });

    // --- Text Rendering Setup --- <<<<<<<<<<<< REVERTED
    let swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let mut viewport = Viewport::new(&device, &cache);
    viewport.update(&queue, Resolution { width, height });

    // --- Use original ColorMode logic (Accurate for sRGB) ---
    let color_mode = if is_srgb {
        ColorMode::Accurate
    } else {
        ColorMode::Web // Default, performs internal sRGB->linear
    };

    // Create atlas using the *target_format* (non-sRGB if surface is sRGB)
    let mut atlas = TextAtlas::with_color_mode(
        &device,
        &queue,
        &cache,
        target_format, // Use target_format (== surface_format)
        color_mode,
    );

    // Create TextRenderer using the configured atlas
    let text_renderer = TextRenderer::new(
        &mut atlas,
        &device,
        MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        None, // No depth/stencil
    );

    // --- Shape Pipeline Setup ---
    // Load base shader (no modification needed)
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
                format: target_format, // Use target_format (== surface_format)
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
        surface_config, // Store the original config
        is_srgb, // Store the flag
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

#[cfg(any(feature = "webgl", feature = "webgpu"))]
fn draw_wgpu(gfx: &mut Graphics, objects: &[Object2d]) {
    let output = match gfx.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(e) => {
            // Use console::error_1
            console::error_1(&JsValue::from_str(&format!("Error getting current texture: {:?}", e)));
            if e == wgpu::SurfaceError::Lost {
                 gfx.surface.configure(&gfx.device, &gfx.surface_config);
                 return;
            }
            return;
        }
    };

    // --- Removed target_is_srgb check ---

    // Create views using default descriptor
    let view = output.texture.create_view(&TextureViewDescriptor::default());
    let msaa_view = gfx.msaa_texture.create_view(&TextureViewDescriptor::default());


    // --- Text Preparation ---
    let mut font_system = FONT_SYSTEM.get()
        .expect("FontSystem not initialized")
        .lock()
        .expect("Failed to lock FontSystem Mutex");

    let mut glyph_buffers: Vec<GlyphonBuffer> = Vec::new();

    // --- WebGL-specific blending adjustment for text ---
    // (No longer needed, remove unused variable)

    for obj in objects {
        if let Object2d::Text(text) = obj {
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let line_height_pixels = text.font_size * text.line_height_multiplier;
            let mut buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(text.font_size, line_height_pixels));
            buffer.set_size(&mut font_system, Some(text_width_f32), Some(text_height_f32));

            // Convert our Family enum to FamilyOwned, then to GlyphonFamily
            let family_owned: FamilyOwned = text.family.clone().into(); // Convert Family -> FamilyOwned
            let glyphon_family = match &family_owned {
                FamilyOwned::Name(name) => GlyphonFamily::Name(name.as_str()),
                FamilyOwned::SansSerif => GlyphonFamily::SansSerif,
                FamilyOwned::Serif => GlyphonFamily::Serif,
                FamilyOwned::Monospace => GlyphonFamily::Monospace,
                FamilyOwned::Cursive => GlyphonFamily::Cursive,
                FamilyOwned::Fantasy => GlyphonFamily::Fantasy,
            };

            // Create a glyphon::fontdb::Family variant with a longer lifetime
            let family_for_query = match &glyphon_family {
                GlyphonFamily::Name(name) => glyphon::fontdb::Family::Name(name),
                GlyphonFamily::SansSerif => glyphon::fontdb::Family::SansSerif,
                GlyphonFamily::Serif => glyphon::fontdb::Family::Serif,
                GlyphonFamily::Monospace => glyphon::fontdb::Family::Monospace,
                GlyphonFamily::Cursive => glyphon::fontdb::Family::Cursive,
                GlyphonFamily::Fantasy => glyphon::fontdb::Family::Fantasy,
            };

            // Create a glyphon::fontdb::Query using the longer-lived family
            let font_query = glyphon::fontdb::Query {
                families: &[family_for_query], // Borrow from the longer-lived variable
                ..Default::default()
            };

            // Check if the font family exists in the database using the glyphon::fontdb::Query
            let font_exists = font_system.db().query(&font_query).is_some();
            if !font_exists {
                #[cfg(target_arch = "wasm32")]
                {
                    let warning_message = format!("Warning: Font family '{:?}' not found. Falling back to default.", text.family);
                    web_sys::console::warn_1(&JsValue::from_str(&warning_message));
                }
                // Optionally, log to stderr on non-wasm targets
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("Warning: Font family '{:?}' not found. Falling back to default.");
            }


            let glyphon_color = text.color.to_glyphon_color(); // Remove premultiplication
            // Use glyphon_family here, which might be a fallback if the original wasn't found
            // Glyphon itself will handle fallback if the specific family isn't found,
            // but the warning above informs the user.
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
        if let Object2d::Text(text) = obj {
            let glyphon_color = text.color.to_glyphon_color(); // Remove premultiplication

            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let text_area = TextArea {
                buffer: &glyph_buffers[buffer_idx],
                left: text.left,
                // Align the top of the text area with the rectangle (no artificial offset)
                top: text.top,
                bounds: TextBounds {
                    left: text.left as i32,
                    top: text.top as i32,
                    right: (text.left + text_width_f32) as i32,
                    bottom: (text.top + text_height_f32) as i32,
                },
                default_color: glyphon_color, // Use adjusted sRGB color
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
        // Use console::error_1
        Err(e) => console::error_1(&JsValue::from_str(&format!("Error preparing text renderer: {:?}", e))),
    }

    // --- Shape Tessellation ---
    let mut buffers: VertexBuffers<ColoredVertex, u32> = VertexBuffers::new();
    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    for obj in objects {
        match obj {
            Object2d::Rectangle(rect) => {
                // Convert input sRGB color to linear for vertex buffer
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
                        // Convert border color to linear
                        let linear_border_color = border_color_val.to_linear();
                        let options = StrokeOptions::default().with_line_width(border_width);
                        stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_border_color })).unwrap();
                    }
                }
            }
            Object2d::Circle(circle) => {
                 // Convert input sRGB color to linear
                 let linear_color = circle.color.to_linear();
                 let mut builder = Path::builder();
                 builder.add_circle(point(circle.center.x, circle.center.y), circle.radius, Winding::Positive);
                 let path = builder.build();
                 if circle.color.a > 0.0 {
                     fill_tessellator.tessellate_path(&path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                 }
                 if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                     if border_width > 0.0 && border_color_val.a > 0.0 {
                         // Convert border color to linear
                         let linear_border_color = border_color_val.to_linear();
                         let options = StrokeOptions::default().with_line_width(border_width);
                         stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_border_color })).unwrap();
                     }
                 }
            }
            Object2d::Line(line) => {
                 // Convert input sRGB color to linear
                 let linear_color = line.color.to_linear();
                 let mut builder = Path::builder();
                 if line.points.len() >= 2 {
                     builder.begin(point(line.points[0].x, line.points[0].y));
                     for i in 1..line.points.len() {
                         builder.line_to(point(line.points[i].x, line.points[i].y));
                     }
                     builder.end(false); // Don't close the path for a line
                 }
                 let path = builder.build();
                 if line.points.len() >= 2 && line.color.a > 0.0 { // Check alpha here
                    let options = StrokeOptions::default().with_line_width(line.width).with_line_cap(LineCap::Round).with_line_join(LineJoin::Round);
                    stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
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
                view: &msaa_view, // Render to MSAA view (surface_format)
                resolve_target: Some(&view), // Resolve to the surface view (surface_format)
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });

        if num_indices > 0 {
            render_pass.set_pipeline(&gfx.rect_pipeline); // Pipeline targets surface_format
            render_pass.set_bind_group(0, &gfx.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..)); // Contains linear colors
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        // Render text (Glyphon renders to the same render pass, targeting surface_format)
        match gfx.text_renderer.render(&gfx.atlas, &gfx.viewport, &mut render_pass) {
             Ok(_) => {}
             Err(e) => console::error_1(&JsValue::from_str(&format!("Error rendering text: {:?}", e))),
         }
    }
    gfx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

#[cfg(any(feature = "webgl", feature = "webgpu"))]
impl From<&crate::Family> for crate::FamilyOwned {
    fn from(family: &crate::Family) -> Self {
        match family {
            crate::Family::Name(name) => crate::FamilyOwned::Name(name.clone().to_owned().into()),
            crate::Family::SansSerif => crate::FamilyOwned::SansSerif,
            crate::Family::Serif => crate::FamilyOwned::Serif,
            crate::Family::Monospace => crate::FamilyOwned::Monospace,
            crate::Family::Cursive => crate::FamilyOwned::Cursive,
            crate::Family::Fantasy => crate::FamilyOwned::Fantasy,
        }
    }
}

#[cfg(any(feature = "webgl", feature = "webgpu"))]
impl From<Family> for FamilyOwned {
    fn from(family: Family) -> Self {
        match family {
            Family::Name(name) => FamilyOwned::Name(name.into()),
            Family::SansSerif => FamilyOwned::SansSerif,
            Family::Serif => FamilyOwned::Serif,
            Family::Monospace => FamilyOwned::Monospace,
            Family::Cursive => FamilyOwned::Cursive,
            Family::Fantasy => FamilyOwned::Fantasy,
        }
    }
}

#[cfg(any(feature = "webgl", feature = "webgpu"))]
fn font_weight_to_glyphon(weight: crate::object_2d::text::FontWeight) -> glyphon::fontdb::Weight {
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

pub async fn fetch_file(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{window, Response};
    use web_sys::js_sys::Uint8Array;

    let win = window().ok_or("No window")?;
    let resp_value = JsFuture::from(win.fetch_with_str(url))
        .await
        .map_err(|_| "fetch failed".to_string())?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "response cast failed".to_string())?;
    let buffer_promise = resp.array_buffer().map_err(|_| "array_buffer failed".to_string())?;
    let buffer_value = JsFuture::from(buffer_promise)
        .await
        .map_err(|_| "array_buffer promise failed".to_string())?;
    let buffer = buffer_value.dyn_into::<web_sys::js_sys::ArrayBuffer>().map_err(|_| "buffer cast failed".to_string())?;
    let u8arr = Uint8Array::new(&buffer);
    let mut bytes = vec![0u8; u8arr.length() as usize];
    u8arr.copy_to(&mut bytes[..]);
    Ok(bytes)
}

#[cfg(feature = "canvas")]
pub(crate) fn draw_canvas(ctx: &CanvasRenderingContext2d, objects: &[Object2d]) {
    // Set default state (optional, but good practice)
    ctx.set_fill_style_str("black"); // Default fill
    ctx.set_stroke_style_str("black"); // Default stroke
    ctx.set_line_width(1.0);

    for obj in objects {
        match obj {
            Object2d::Rectangle(rect) => {
                // Set fill color
                let fill_color = rect.color.to_canvas_rgba();
                ctx.set_fill_style_str(&fill_color);
                // TODO: Add rounded rectangle support if radii > 0 using path API
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
                ).unwrap_throw();
                // Fill
                if circle.color.a > 0.0 {
                    let fill_color = circle.color.to_canvas_rgba();
                    ctx.set_fill_style_str(&fill_color);
                    ctx.fill();
                }
                // Border
                if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let stroke_color = border_color_val.to_canvas_rgba();
                        ctx.set_stroke_style_str(&stroke_color);
                        ctx.set_line_width(border_width as f64);
                        ctx.stroke();
                    }
                }
            }
            Object2d::Line(line) => {
                if line.points.len() >= 2 && line.color.a > 0.0 {
                    let stroke_color = line.color.to_canvas_rgba();
                    ctx.set_stroke_style_str(&stroke_color);
                    ctx.set_line_width(line.width as f64);
                    ctx.begin_path();
                    ctx.move_to(line.points[0].x as f64, line.points[0].y as f64);
                    for i in 1..line.points.len() {
                        ctx.line_to(line.points[i].x as f64, line.points[i].y as f64);
                    }
                    ctx.stroke();
                }
            }
            Object2d::Text(text) => {
                if text.color.a > 0.0 {
                    let fill_color = text.color.to_canvas_rgba();
                    ctx.set_fill_style_str(&fill_color);
                    let font_style = if text.italic { "italic" } else { "normal" };
                    let font_weight = font_weight_to_css(&text.weight);
                    let font_str = format!("{} {} {}px {}", font_style, font_weight, text.font_size, text.family);
                    ctx.set_font(&font_str);
                    let max_width = text.width;
                    let line_height = text.font_size * text.line_height_multiplier;
                    let words: Vec<&str> = text.text.split_whitespace().collect();
                    let mut lines: Vec<String> = Vec::new();
                    let mut current_line = String::new();
                    for word in words {
                        let test_line = if current_line.is_empty() {
                            word.to_string()
                        } else {
                            format!("{} {}", current_line, word)
                        };
                        let metrics = ctx.measure_text(&test_line).unwrap_throw();
                        if metrics.width() <= max_width as f64 || current_line.is_empty() {
                            current_line = test_line;
                        } else {
                            lines.push(current_line);
                            current_line = word.to_string();
                        }
                    }
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                    let mut y = text.top;
                    for line in lines {
                        let metrics = ctx.measure_text(&line).unwrap_throw();
                        let ascent = metrics.actual_bounding_box_ascent();
                        let font_box_ascent = metrics.font_bounding_box_ascent();
                        let gap = font_box_ascent - ascent;
                        let line_gap = if gap > 0.0 && gap < 1.0 { gap } else { 0.0 };
                        ctx.fill_text(&line, text.left as f64, y as f64 + ascent + line_gap).unwrap_throw();
                        y += line_height;
                        if y > text.top + text.height {
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "canvas")]
fn font_weight_to_css(weight: &crate::object_2d::text::FontWeight) -> &'static str {
    use crate::object_2d::text::FontWeight::*;
    match weight {
        Thin => "100",
        ExtraLight => "200",
        Light => "300",
        Regular => "400",
        Medium => "500",
        SemiBold => "600",
        Bold => "700",
        ExtraBold => "800",
        Black => "900",
    }
}
