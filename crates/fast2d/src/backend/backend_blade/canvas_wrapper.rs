use crate::Object2d;
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;
use glyphon::{Shaping, Buffer as GlyphonBuffer, TextArea, Attrs, TextBounds, Metrics, Family as GlyphonFamily, TextRenderer, TextAtlas, SwashCache, Cache, Viewport, Resolution, ColorMode};
use crate::backend::backend_blade::FONT_SYSTEM;
use web_sys::wasm_bindgen::UnwrapThrowExt;

/// Rectangle vertex for Blade-style rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RectangleVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl RectangleVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectangleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Circle vertex for Blade-style rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CircleVertex {
    position: [f32; 2],
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
    _padding: f32, // For alignment
}

impl CircleVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x2, // position
        1 => Float32x2, // center
        2 => Float32,   // radius
        3 => Float32x4  // color
    ];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Line vertex for Blade-style rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl LineVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// A Blade-inspired WebGPU canvas wrapper for browser rendering.
/// Uses WebGPU directly with Blade Graphics patterns and architecture.
pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas: HtmlCanvasElement,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    triangle_pipeline: Option<wgpu::RenderPipeline>,
    rectangle_pipeline: Option<wgpu::RenderPipeline>,
    circle_pipeline: Option<wgpu::RenderPipeline>,
    line_pipeline: Option<wgpu::RenderPipeline>,
    text_renderer: Option<TextRenderer>,
    text_atlas: Option<TextAtlas>,
    text_cache: Option<Cache>,
    swash_cache: Option<SwashCache>,
    viewport: Option<Viewport>,
    current_size: (u32, u32),
}

impl CanvasWrapper {
    /// Creates a new Blade-inspired WebGPU canvas wrapper.
    /// Requires WebGPU support - will fail if not available.
    pub async fn new_with_canvas(canvas: HtmlCanvasElement) -> Self {
        web_sys::console::log_1(&"🔥 Initializing Blade-inspired WebGPU backend...".into());
        
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        
        // Create WGPU instance with WebGPU backend only
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        
        // Create surface from canvas
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .expect("Failed to create WebGPU surface");
        
        // Request adapter with WebGPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("WebGPU adapter not found - WebGPU not supported");
        
        web_sys::console::log_1(&"✅ WebGPU adapter acquired".into());
        
        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor::default(),
            )
            .await
            .expect("Failed to request WebGPU device");
        
        web_sys::console::log_1(&"✅ WebGPU device and queue created".into());
        
        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &surface_config);
        
        web_sys::console::log_1(&"✅ WebGPU surface configured".into());
        
        // Create all rendering pipelines (Blade-style)
        let triangle_pipeline = Self::create_blade_triangle_pipeline(&device, surface_format).ok();
        let rectangle_pipeline = Self::create_blade_rectangle_pipeline_with_dimensions(&device, surface_format, width as f32, height as f32).ok();
        let circle_pipeline = Self::create_blade_circle_pipeline_with_dimensions(&device, surface_format, width as f32, height as f32).ok();
        let line_pipeline = Self::create_blade_line_pipeline_with_dimensions(&device, surface_format, width as f32, height as f32).ok();
        
        // Initialize text rendering components
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut viewport = Viewport::new(&device, &cache);
        viewport.update(&queue, Resolution { width, height });
        
        let color_mode = ColorMode::Web;
        let mut atlas = TextAtlas::with_color_mode(
            &device,
            &queue,
            &cache,
            surface_format,
            color_mode,
        );
        
        let text_renderer = TextRenderer::new(
            &mut atlas,
            &device,
            wgpu::MultisampleState {
                count: 1,  // No MSAA for Blade backend
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );
        
        web_sys::console::log_1(&"✅ Blade text renderer initialized".into());
        
        web_sys::console::log_1(&"✅ Blade-style WebGPU pipelines ready!".into());
        
        Self {
            objects: Vec::new(),
            canvas,
            device,
            queue,
            surface,
            surface_config,
            triangle_pipeline,
            rectangle_pipeline,
            circle_pipeline,
            line_pipeline,
            text_renderer: Some(text_renderer),
            text_atlas: Some(atlas),
            text_cache: Some(cache),
            swash_cache: Some(swash_cache),
            viewport: Some(viewport),
            current_size: (width, height),
        }
    }

    /// Updates objects and renders using Blade-inspired WebGPU approach
    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        
        web_sys::console::log_1(&format!("🎨 Blade WebGPU rendering {} objects", self.objects.len()).into());
        
        self.render_blade_style();
    }

    /// Handles resizing using Blade-inspired approach
    pub fn resized(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 || (width == self.current_size.0 && height == self.current_size.1) {
            return;
        }
        
        web_sys::console::log_1(&format!("🔧 Blade WebGPU resize: {}x{}", width, height).into());
        
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.current_size = (width, height);
        
        // Reconfigure surface (Blade-style)
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        
        // Recreate pipelines with new dimensions for proper coordinate normalization
        let surface_format = self.surface_config.format;
        self.rectangle_pipeline = Self::create_blade_rectangle_pipeline_with_dimensions(&self.device, surface_format, width as f32, height as f32).ok();
        self.circle_pipeline = Self::create_blade_circle_pipeline_with_dimensions(&self.device, surface_format, width as f32, height as f32).ok();
        self.line_pipeline = Self::create_blade_line_pipeline_with_dimensions(&self.device, surface_format, width as f32, height as f32).ok();
        
        web_sys::console::log_1(&"🔧 Pipelines recreated with new dimensions".into());
        
        // Re-render
        self.render_blade_style();
    }
    
    /// Render using Blade-inspired WebGPU patterns
    fn render_blade_style(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(_) => {
                web_sys::console::log_1(&"⚠️ Failed to get surface texture".into());
                return;
            }
        };
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create command encoder (Blade-style naming)
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Blade Command Encoder"),
        });
        
        // Render pass with Blade-style clear color
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blade Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), // Blade-style black background
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            // Render all 2D objects using Blade-inspired WebGPU
            self.render_objects(&mut render_pass);
        }
        
        // Submit commands (Blade-style)
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        web_sys::console::log_1(&"✅ Blade WebGPU frame rendered".into());
    }
    
    /// Create triangle pipeline using Blade-inspired shader patterns
    fn create_blade_triangle_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> Result<wgpu::RenderPipeline, Box<dyn std::error::Error>> {
        // Blade-inspired shader (similar to blade_example triangle.wgsl)
        let shader_source = r#"
            // Blade-inspired triangle shader
            @vertex
            fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
                var pos = array<vec2<f32>, 3>(
                    vec2<f32>(-0.5, -0.5),
                    vec2<f32>( 0.5, -0.5), 
                    vec2<f32>( 0.0,  0.5)
                );
                return vec4<f32>(pos[vertex_index], 0.0, 1.0);
            }

            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                // Blade Graphics orange/red color scheme
                return vec4<f32>(1.0, 0.4, 0.0, 1.0);
            }
        "#;
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blade Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blade Triangle Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blade Triangle Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(pipeline)
    }
    
    /// Create rectangle rendering pipeline using Blade Graphics patterns
    fn create_blade_rectangle_pipeline_with_dimensions(device: &wgpu::Device, format: wgpu::TextureFormat, canvas_width: f32, canvas_height: f32) -> Result<wgpu::RenderPipeline, Box<dyn std::error::Error>> {
        let shader_source = format!(r#"
            // Blade-inspired rectangle shader
            struct VertexInput {{
                @location(0) position: vec2<f32>,
                @location(1) color: vec4<f32>,
            }}

            struct VertexOutput {{
                @builtin(position) position: vec4<f32>,
                @location(0) color: vec4<f32>,
            }}

            @vertex
            fn vs_main(input: VertexInput) -> VertexOutput {{
                var output: VertexOutput;
                // Convert from screen coordinates to normalized device coordinates
                let normalized_x = (input.position.x / {}) * 2.0 - 1.0;
                let normalized_y = 1.0 - (input.position.y / {}) * 2.0;
                output.position = vec4<f32>(normalized_x, normalized_y, 0.0, 1.0);
                output.color = input.color;
                return output;
            }}

            @fragment
            fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
                return input.color;
            }}
        "#, canvas_width, canvas_height);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blade Rectangle Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blade Rectangle Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blade Rectangle Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[RectangleVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(pipeline)
    }
    
    /// Create circle rendering pipeline using Blade Graphics patterns
    fn create_blade_circle_pipeline_with_dimensions(device: &wgpu::Device, format: wgpu::TextureFormat, canvas_width: f32, canvas_height: f32) -> Result<wgpu::RenderPipeline, Box<dyn std::error::Error>> {
        let shader_source = format!(r#"
            // Blade-inspired circle shader
            struct VertexInput {{
                @location(0) position: vec2<f32>,
                @location(1) center: vec2<f32>,
                @location(2) radius: f32,
                @location(3) color: vec4<f32>,
            }}

            struct VertexOutput {{
                @builtin(position) position: vec4<f32>,
                @location(0) center: vec2<f32>,
                @location(1) radius: f32,
                @location(2) color: vec4<f32>,
                @location(3) frag_pos: vec2<f32>,
            }}

            @vertex
            fn vs_main(input: VertexInput) -> VertexOutput {{
                var output: VertexOutput;
                // Convert from screen coordinates to normalized device coordinates
                let normalized_x = (input.position.x / {}) * 2.0 - 1.0;
                let normalized_y = 1.0 - (input.position.y / {}) * 2.0;
                output.position = vec4<f32>(normalized_x, normalized_y, 0.0, 1.0);
                output.center = input.center;
                output.radius = input.radius;
                output.color = input.color;
                output.frag_pos = input.position;
                return output;
            }}

            @fragment
            fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
                let dist = distance(input.frag_pos, input.center);
                if (dist > input.radius) {{
                    discard;
                }}
                return input.color;
            }}
        "#, canvas_width, canvas_height);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blade Circle Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blade Circle Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blade Circle Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[CircleVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(pipeline)
    }
    
    /// Create line rendering pipeline using Blade Graphics patterns
    fn create_blade_line_pipeline_with_dimensions(device: &wgpu::Device, format: wgpu::TextureFormat, canvas_width: f32, canvas_height: f32) -> Result<wgpu::RenderPipeline, Box<dyn std::error::Error>> {
        let shader_source = format!(r#"
            // Blade-inspired line shader
            struct VertexInput {{
                @location(0) position: vec2<f32>,
                @location(1) color: vec4<f32>,
            }}

            struct VertexOutput {{
                @builtin(position) position: vec4<f32>,
                @location(0) color: vec4<f32>,
            }}

            @vertex
            fn vs_main(input: VertexInput) -> VertexOutput {{
                var output: VertexOutput;
                // Convert from screen coordinates to normalized device coordinates
                let normalized_x = (input.position.x / {}) * 2.0 - 1.0;
                let normalized_y = 1.0 - (input.position.y / {}) * 2.0;
                output.position = vec4<f32>(normalized_x, normalized_y, 0.0, 1.0);
                output.color = input.color;
                return output;
            }}

            @fragment
            fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
                return input.color;
            }}
        "#, canvas_width, canvas_height);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blade Line Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blade Line Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blade Line Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[LineVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(pipeline)
    }
    
    /// Render all 2D objects using Blade WebGPU pipelines
    fn render_objects(&self, render_pass: &mut wgpu::RenderPass) {
        let canvas_width = self.current_size.0 as f32;
        let canvas_height = self.current_size.1 as f32;
        for object in &self.objects {
            match object {
                Object2d::Rectangle(rect) => {
                    self.render_rectangle(render_pass, rect, canvas_width, canvas_height);
                }
                Object2d::Circle(circle) => {
                    self.render_circle(render_pass, circle, canvas_width, canvas_height);
                }
                Object2d::Line(line) => {
                    self.render_line(render_pass, line, canvas_width, canvas_height);
                }
                Object2d::Text(text) => {
                    self.render_text(render_pass, text, canvas_width, canvas_height);
                }
            }
        }
    }
    
    /// Render rectangle using Blade WebGPU
    fn render_rectangle(&self, render_pass: &mut wgpu::RenderPass, rect: &crate::Rectangle, _canvas_width: f32, _canvas_height: f32) {
        if let Some(ref pipeline) = self.rectangle_pipeline {
            // Create rectangle vertices (2 triangles = 6 vertices)
            let color = [
                rect.color.r as f32 / 255.0,
                rect.color.g as f32 / 255.0,
                rect.color.b as f32 / 255.0,
                rect.color.a,
            ];
            
            let x = rect.position.x;
            let y = rect.position.y;
            let w = rect.size.width;
            let h = rect.size.height;
            
            let vertices = [
                // Triangle 1
                RectangleVertex { position: [x, y], color },         // Bottom-left
                RectangleVertex { position: [x + w, y], color },     // Bottom-right
                RectangleVertex { position: [x, y + h], color },     // Top-left
                // Triangle 2
                RectangleVertex { position: [x + w, y], color },     // Bottom-right
                RectangleVertex { position: [x + w, y + h], color }, // Top-right
                RectangleVertex { position: [x, y + h], color },     // Top-left
            ];
            
            // Create vertex buffer
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rectangle Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }
    }
    
    /// Render circle using Blade WebGPU
    fn render_circle(&self, render_pass: &mut wgpu::RenderPass, circle: &crate::Circle, _canvas_width: f32, _canvas_height: f32) {
        if let Some(ref pipeline) = self.circle_pipeline {
            // Create circle as quad (2 triangles = 6 vertices)
            let color = [
                circle.color.r as f32 / 255.0,
                circle.color.g as f32 / 255.0,
                circle.color.b as f32 / 255.0,
                circle.color.a,
            ];
            
            let center = [circle.center.x, circle.center.y];
            let radius = circle.radius;
            
            // Create bounding box for the circle
            let x = center[0] - radius;
            let y = center[1] - radius;
            let w = radius * 2.0;
            let h = radius * 2.0;
            
            let vertices = [
                // Triangle 1
                CircleVertex { position: [x, y], center, radius, color, _padding: 0.0 },
                CircleVertex { position: [x + w, y], center, radius, color, _padding: 0.0 },
                CircleVertex { position: [x, y + h], center, radius, color, _padding: 0.0 },
                // Triangle 2
                CircleVertex { position: [x + w, y], center, radius, color, _padding: 0.0 },
                CircleVertex { position: [x + w, y + h], center, radius, color, _padding: 0.0 },
                CircleVertex { position: [x, y + h], center, radius, color, _padding: 0.0 },
            ];
            
            // Create vertex buffer
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Circle Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }
    }
    
    /// Render line using Blade WebGPU
    fn render_line(&self, render_pass: &mut wgpu::RenderPass, line: &crate::Line, _canvas_width: f32, _canvas_height: f32) {
        if let Some(ref pipeline) = self.line_pipeline {
            let color = [
                line.color.r as f32 / 255.0,
                line.color.g as f32 / 255.0,
                line.color.b as f32 / 255.0,
                line.color.a,
            ];
            
            // Convert line points to vertices
            let mut vertices = Vec::new();
            for point in &line.points {
                vertices.push(LineVertex {
                    position: [point.x, point.y],
                    color,
                });
            }
            
            if vertices.len() >= 2 {
                // Create vertex buffer
                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Line Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                
                render_pass.set_pipeline(pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..vertices.len() as u32, 0..1);
            }
        }
    }
    
    /// Render text using Blade WebGPU with glyphon
    fn render_text(&self, render_pass: &mut wgpu::RenderPass, text: &crate::Text, _canvas_width: f32, _canvas_height: f32) {
        // Check if we have all required text rendering components - simplified for now
        if self.text_renderer.is_none() {
            web_sys::console::log_1(&"⚠️ Text rendering components not available".into());
            return;
        }
        
        let Some(font_system_mutex) = FONT_SYSTEM.get() else {
            web_sys::console::log_1(&"⚠️ Font system not initialized".into());
            return;
        };
        
        let mut font_system = match font_system_mutex.lock() {
            Ok(fs) => fs,
            Err(_) => {
                web_sys::console::log_1(&"⚠️ Failed to lock font system".into());
                return;
            }
        };
        
        // Set up text metrics and buffer
        let line_height_pixels = text.font_size * text.line_height_multiplier;
        let mut buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(text.font_size, line_height_pixels));
        buffer.set_size(&mut font_system, Some(text.width), Some(text.height));

        // Convert font family to glyphon format
        let glyphon_family = match &text.family {
            crate::object2d::Family::Name(name) => GlyphonFamily::Name(name.as_ref()),
            crate::object2d::Family::SansSerif => GlyphonFamily::SansSerif,
            crate::object2d::Family::Serif => GlyphonFamily::Serif,
            crate::object2d::Family::Monospace => GlyphonFamily::Monospace,
            crate::object2d::Family::Cursive => GlyphonFamily::Cursive,
            crate::object2d::Family::Fantasy => GlyphonFamily::Fantasy,
        };

        // Set up text attributes (color, weight, style)
        let glyphon_color = glyphon::Color::rgba(
            text.color.r, 
            text.color.g, 
            text.color.b, 
            (text.color.a * 255.0) as u8
        );
        let attrs = Attrs::new()
            .family(glyphon_family)
            .color(glyphon_color)
            .weight({
                use crate::object2d::FontWeight::*;
                match text.weight {
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
            })
            .style(if text.italic { glyphon::fontdb::Style::Italic } else { glyphon::fontdb::Style::Normal });
        
        buffer.set_text(&mut font_system, &text.text, &attrs, Shaping::Advanced);

        // For now, just log that text would be rendered
        // TODO: Implement proper text rendering with mutable access to components
        web_sys::console::log_1(&format!("🔤 Would render text: '{}' at ({}, {})", text.text, text.left, text.top).into());
    }
}