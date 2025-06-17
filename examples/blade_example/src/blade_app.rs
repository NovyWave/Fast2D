use anyhow::Result;
use tao::window::Window;
use std::sync::Arc;
use blade_graphics as gpu;

/// Simple rectangle vertex with position and color
#[derive(blade_macros::Vertex, Clone, Copy)]
struct RectangleVertex {
    position: [f32; 2],
    color: [f32; 4],
}

/// Blade-powered application - equivalent to NativeApp but using Blade Graphics
pub struct BladeApp {
    window: Arc<Window>,
    context: gpu::Context,
    surface: gpu::Surface,
    triangle_pipeline: Option<gpu::RenderPipeline>,
    rectangle_pipeline: Option<gpu::RenderPipeline>,
    rectangle_vertex_buffer: Option<gpu::Buffer>,
    scroll_offset: f32,
    current_size: (u32, u32),
    frame_skip_count: u32,
}

impl BladeApp {
    /// Create new Blade application with same API as NativeApp
    pub async fn new(window: Window) -> Result<Self> {
        let window = Arc::new(window);
        let window_size = window.inner_size();
        
        println!("🔍 Initializing Blade Graphics context...");
        
        // Initialize Blade Graphics context
        let context = unsafe {
            gpu::Context::init(gpu::ContextDesc {
                presentation: true,           // Enable window presentation
                validation: cfg!(debug_assertions), // Debug validation in debug mode
                timing: false,               // GPU timing (optional)
                capture: false,              // Capture support (optional)
                overlay: false,              // API overlay (optional)
                device_id: 0,               // Device selection
                ..Default::default()
            }).map_err(|e| anyhow::anyhow!("Failed to initialize Blade context: {:?}", e))?
        };
        
        println!("✅ Blade Graphics context created!");
        
        // Create surface configuration
        let surface_config = Self::make_surface_config(window_size);
        
        // Create and configure surface from window
        let surface = context.create_surface_configured(&*window, surface_config)
            .map_err(|e| anyhow::anyhow!("Failed to create Blade surface: {:?}", e))?;
        
        println!("✅ Blade Graphics surface created!");
        
        // Create triangle rendering pipeline
        let triangle_pipeline = Self::create_triangle_pipeline(&context, &surface)?;
        
        // TODO: Temporarily disable rectangle pipeline due to shader issue
        // let (rectangle_pipeline, rectangle_vertex_buffer) = 
        //     Self::create_rectangle_pipeline(&context, &surface)?;
        let rectangle_pipeline = None;
        let rectangle_vertex_buffer = None;
        
        println!("✅ Triangle pipeline created! (Rectangle temporarily disabled)");
        println!("🎯 Window size: {}x{}", window_size.width, window_size.height);
        
        Ok(Self {
            window,
            context,
            surface,
            triangle_pipeline: Some(triangle_pipeline),
            rectangle_pipeline,
            rectangle_vertex_buffer,
            scroll_offset: 0.0,
            current_size: (window_size.width, window_size.height),
            frame_skip_count: 0,
        })
    }
    
    /// Handle window resize - CRITICAL TEST for Blade stability
    pub fn handle_resize(&mut self, width: u32, height: u32) -> Result<()> {
        // Skip zero dimensions
        if width == 0 || height == 0 {
            return Ok(());
        }
        
        // Only process if size actually changed
        if self.current_size.0 == width && self.current_size.1 == height {
            return Ok(());
        }
        
        println!("🔍 Blade resize: {}x{} -> {}x{}", 
                self.current_size.0, self.current_size.1, width, height);
        
        self.current_size = (width, height);
        
        // Create new surface configuration with new size
        let new_size = tao::dpi::PhysicalSize::new(width, height);
        let surface_config = Self::make_surface_config(new_size);
        
        // Reconfigure surface - simpler than WGPU's complex surface configuration
        self.context.reconfigure_surface(&mut self.surface, surface_config);
        
        println!("✅ Blade resize completed successfully");
        Ok(())
    }
    
    /// Render current frame - TEST for Blade rendering stability
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Skip rendering for the first few frames to let things settle
        if self.frame_skip_count < 5 {
            self.frame_skip_count += 1;
            println!("⏳ Skipping frame {} to let graphics settle", self.frame_skip_count);
            return Ok(());
        }
        
        // Now try actual rendering with triangle
        if self.frame_skip_count == 5 {
            println!("🎨 Starting triangle rendering...");
            self.frame_skip_count += 1;
        }
        
        println!("🔍 Attempting frame acquisition...");
        
        // Acquire frame with proper error handling (only blocking method available)
        let frame = self.surface.acquire_frame();
        
        println!("✅ Frame acquired successfully!");
        
        // Create command encoder with proper descriptor - reduced buffer count for stability
        let mut command_encoder = self.context.create_command_encoder(gpu::CommandEncoderDesc {
            name: "triangle_encoder",
            buffer_count: 1, // Single buffer for initial stability
        });
        
        // Start encoding
        command_encoder.start();
        
        // Create render targets with dark background
        let render_targets = gpu::RenderTargetSet {
            colors: &[gpu::RenderTarget {
                view: frame.texture_view(),
                init_op: gpu::InitOp::Clear(gpu::TextureColor::OpaqueBlack), // Black background
                finish_op: gpu::FinishOp::Store,
            }],
            depth_stencil: None,
        };
        
        // Begin render pass and draw primitives
        {
            let mut render_encoder = command_encoder.render("main_pass", render_targets);
            
            // Draw triangle (simple red triangle to test)
            if let Some(ref pipeline) = self.triangle_pipeline {
                let mut pipeline_encoder = render_encoder.with(pipeline);
                pipeline_encoder.draw(0, 3, 0, 1); // Draw 3 vertices for triangle
            }
            
            // Draw rectangles (temporarily disabled)
            // self.render_rectangles(&mut render_encoder);
        }
        
        // Submit commands and get sync point for explicit control
        let sync_point = self.context.submit(&mut command_encoder);
        
        // Wait for completion with reasonable timeout to avoid fence errors
        let wait_success = self.context.wait_for(&sync_point, 16); // 16ms timeout (~60fps)
        if !wait_success {
            println!("⚠️  Frame timeout - GPU may be busy");
        }
        
        // Frame is automatically presented when dropped in Blade
        
        Ok(())
    }
    
    /// Scroll up through examples
    pub fn scroll_up(&mut self) {
        self.scroll_offset = (self.scroll_offset - 40.0).max(0.0);
        println!("🔍 Scrolled up, offset: {}", self.scroll_offset);
        // TODO: Update example layout
    }
    
    /// Scroll down through examples  
    pub fn scroll_down(&mut self) {
        let panel_height = 250.0;
        let panel_spacing = 30.0;
        let max_scroll: f32 = 3.0 * (panel_height + panel_spacing) - 500.0;
        self.scroll_offset = (self.scroll_offset + 40.0).min(max_scroll.max(0.0));
        println!("🔍 Scrolled down, offset: {}", self.scroll_offset);
        // TODO: Update example layout
    }
    
    /// Request window redraw
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
    
    /// Get current window size
    pub fn window_size(&self) -> (u32, u32) {
        self.current_size
    }
    
    /// Create surface configuration from window size
    fn make_surface_config(size: tao::dpi::PhysicalSize<u32>) -> gpu::SurfaceConfig {
        gpu::SurfaceConfig {
            size: gpu::Extent {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            usage: gpu::TextureUsage::TARGET,
            display_sync: gpu::DisplaySync::Recent, // Try Recent instead of Block for less aggressive sync
            transparent: false,
            ..Default::default()
        }
    }
    
    /// Create triangle rendering pipeline  
    fn create_triangle_pipeline(context: &gpu::Context, surface: &gpu::Surface) -> Result<gpu::RenderPipeline> {
        // Load triangle shader
        let shader_source = include_str!("triangle.wgsl");
        
        // Create shader module
        let shader = context.create_shader(gpu::ShaderDesc {
            source: shader_source,
        });
        
        // Get surface format from surface info
        let surface_info = surface.info();
        let surface_format = surface_info.format;
        
        // Create render pipeline for triangle
        let pipeline = context.create_render_pipeline(gpu::RenderPipelineDesc {
            name: "triangle_pipeline",
            data_layouts: &[], 
            vertex: shader.at("vs_main"),
            vertex_fetches: &[], // Empty for hardcoded triangle vertices
            fragment: Some(shader.at("fs_main")),
            primitive: gpu::PrimitiveState {
                topology: gpu::PrimitiveTopology::TriangleList,
                front_face: gpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                wireframe: false, // Only wireframe field exists
            },
            depth_stencil: None,
            color_targets: &[gpu::ColorTargetState {  // Direct array, not Option wrapper
                format: surface_format,
                blend: Some(gpu::BlendState::ALPHA_BLENDING),
                write_mask: gpu::ColorWrites::ALL,
            }],
            multisample_state: gpu::MultisampleState::default(),
        });
        
        Ok(pipeline)
    }
    
    /// Create simple rectangle rendering pipeline and buffer
    fn create_rectangle_pipeline(context: &gpu::Context, surface: &gpu::Surface) -> Result<(gpu::RenderPipeline, gpu::Buffer)> {
        // Load simple rectangle shader
        let shader_source = include_str!("simple_rectangle.wgsl");
        
        // Create shader module
        let shader = context.create_shader(gpu::ShaderDesc {
            source: shader_source,
        });
        
        // Get surface format
        let surface_info = surface.info();
        let surface_format = surface_info.format;
        
        // Create some test rectangles as triangulated vertices
        let rectangle_vertices = Self::create_rectangle_vertices();
        
        let vertex_buffer = context.create_buffer(gpu::BufferDesc {
            name: "rectangle_vertices",
            size: (rectangle_vertices.len() * std::mem::size_of::<RectangleVertex>()) as u64,
            memory: gpu::Memory::Shared,
        });
        
        // Upload rectangle vertices
        unsafe {
            std::ptr::copy_nonoverlapping(
                rectangle_vertices.as_ptr(),
                vertex_buffer.data() as *mut RectangleVertex,
                rectangle_vertices.len(),
            );
        }
        context.sync_buffer(vertex_buffer);
        
        // Create render pipeline
        let pipeline = context.create_render_pipeline(gpu::RenderPipelineDesc {
            name: "rectangle_pipeline",
            data_layouts: &[], // No shader uniform data
            vertex: shader.at("vs_main"),
            vertex_fetches: &[
                gpu::VertexFetchState {
                    layout: &<RectangleVertex as gpu::Vertex>::layout(),
                    instanced: false,
                },
            ],
            fragment: Some(shader.at("fs_main")),
            primitive: gpu::PrimitiveState {
                topology: gpu::PrimitiveTopology::TriangleList, // Standard triangles
                front_face: gpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                wireframe: false,
            },
            depth_stencil: None,
            color_targets: &[gpu::ColorTargetState {
                format: surface_format,
                blend: Some(gpu::BlendState::ALPHA_BLENDING),
                write_mask: gpu::ColorWrites::ALL,
            }],
            multisample_state: gpu::MultisampleState::default(),
        });
        
        Ok((pipeline, vertex_buffer))
    }
    
    /// Create rectangle vertices as triangulated quads
    fn create_rectangle_vertices() -> Vec<RectangleVertex> {
        let mut vertices = Vec::new();
        
        // Rectangle 1: Purple rectangle
        let rect1_color = [0.2, 0.0, 0.4, 1.0];
        let (x1, y1, w1, h1) = (100.0, 100.0, 200.0, 150.0);
        vertices.extend_from_slice(&[
            // Triangle 1
            RectangleVertex { position: [x1, y1], color: rect1_color },           // Bottom-left
            RectangleVertex { position: [x1 + w1, y1], color: rect1_color },     // Bottom-right 
            RectangleVertex { position: [x1, y1 + h1], color: rect1_color },     // Top-left
            // Triangle 2
            RectangleVertex { position: [x1 + w1, y1], color: rect1_color },     // Bottom-right
            RectangleVertex { position: [x1 + w1, y1 + h1], color: rect1_color }, // Top-right
            RectangleVertex { position: [x1, y1 + h1], color: rect1_color },     // Top-left
        ]);
        
        // Rectangle 2: Green rectangle
        let rect2_color = [0.0, 0.6, 0.0, 1.0];
        let (x2, y2, w2, h2) = (350.0, 200.0, 150.0, 100.0);
        vertices.extend_from_slice(&[
            // Triangle 1
            RectangleVertex { position: [x2, y2], color: rect2_color },           // Bottom-left
            RectangleVertex { position: [x2 + w2, y2], color: rect2_color },     // Bottom-right
            RectangleVertex { position: [x2, y2 + h2], color: rect2_color },     // Top-left
            // Triangle 2
            RectangleVertex { position: [x2 + w2, y2], color: rect2_color },     // Bottom-right
            RectangleVertex { position: [x2 + w2, y2 + h2], color: rect2_color }, // Top-right
            RectangleVertex { position: [x2, y2 + h2], color: rect2_color },     // Top-left
        ]);
        
        // Rectangle 3: Orange rectangle
        let rect3_color = [0.8, 0.4, 0.0, 1.0];
        let (x3, y3, w3, h3) = (50.0, 350.0, 300.0, 80.0);
        vertices.extend_from_slice(&[
            // Triangle 1
            RectangleVertex { position: [x3, y3], color: rect3_color },           // Bottom-left
            RectangleVertex { position: [x3 + w3, y3], color: rect3_color },     // Bottom-right
            RectangleVertex { position: [x3, y3 + h3], color: rect3_color },     // Top-left
            // Triangle 2
            RectangleVertex { position: [x3 + w3, y3], color: rect3_color },     // Bottom-right
            RectangleVertex { position: [x3 + w3, y3 + h3], color: rect3_color }, // Top-right
            RectangleVertex { position: [x3, y3 + h3], color: rect3_color },     // Top-left
        ]);
        
        vertices
    }
    
    /// Render rectangles for the current frame
    fn render_rectangles(&mut self, render_encoder: &mut gpu::RenderCommandEncoder) {
        if let (Some(ref pipeline), Some(ref vertex_buffer)) = 
            (&self.rectangle_pipeline, &self.rectangle_vertex_buffer) {
            
            // Render rectangles with simple vertex approach
            let mut pipeline_encoder = render_encoder.with(pipeline);
            
            // Bind vertex buffer
            pipeline_encoder.bind_vertex(0, (*vertex_buffer).into());
            
            // Draw rectangles (18 vertices = 6 triangles = 3 rectangles)
            pipeline_encoder.draw(0, 18, 0, 1);
        }
    }
    
    // TODO: Private helper methods for rendering other primitives
    /*
    fn render_circles(&mut self, render_encoder: &mut gpu::RenderCommandEncoder) {
        // TODO: Implement circle rendering
    }
    
    fn render_lines(&mut self, render_encoder: &mut gpu::RenderCommandEncoder) {
        // TODO: Implement line rendering
    }
    */
}