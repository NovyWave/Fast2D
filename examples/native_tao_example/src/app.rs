use anyhow::Result;
use tao::window::Window;
use std::sync::Arc;

use crate::examples::examples;

/// Main application struct that coordinates all components
pub struct NativeApp {
    window: Arc<Window>,
    canvas_wrapper: fast2d::CanvasWrapper,
    scroll_offset: f32,
    current_size: (u32, u32),
}

impl NativeApp {
    /// Create a new native application with column layout
    pub async fn new(window: Window) -> Result<Self> {
        let window = Arc::new(window);
        
        println!("Initializing Fast2D native canvas...");
        let window_size = window.inner_size();
        
        // Create WGPU instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Fast2D Native Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .expect("Failed to create device");

        // Create single Fast2D CanvasWrapper that will show all examples in column layout
        let mut canvas_wrapper = fast2d::CanvasWrapper::new_with_surface(
            surface, device, queue, adapter, window_size.width, window_size.height
        ).await;

        // Initialize with all examples arranged in a column
        if let Err(e) = Self::update_column_layout(&mut canvas_wrapper, 0.0) {
            eprintln!("Warning: Initial layout failed: {:?}", e);
        }
        
        println!("Fast2D native canvas initialized!");
        
        Ok(Self {
            window,
            canvas_wrapper,
            scroll_offset: 0.0,
            current_size: (window_size.width, window_size.height),
        })
    }
    
    /// Update the canvas to show all examples in a column layout
    fn update_column_layout(canvas_wrapper: &mut fast2d::CanvasWrapper, scroll_offset: f32) -> Result<(), wgpu::SurfaceError> {
        let mut all_objects = Vec::new();
        
        let panel_height = 250.0; // Each panel is 250px tall (smaller to fit better)
        let panel_spacing = 30.0; // Gap between panels
        
        // Create each example at the correct Y offset
        for i in 0..3 {
            let y_offset = (i as f32 * (panel_height + panel_spacing)) - scroll_offset + 40.0;
            
            // Add a title text for each example
            let title = match i {
                0 => "Rectangle Example",
                1 => "Face Example", 
                2 => "Sine Wave Example",
                _ => "Example",
            };
            
            all_objects.push(
                fast2d::Text::new()
                    .text(title)
                    .position(10.0, y_offset - 30.0)
                    .size(200.0, 30.0)
                    .color(255, 255, 255, 1.0)
                    .font_size(18.0)
                    .family(fast2d::Family::name("Inter"))
                    .weight(fast2d::FontWeight::Bold)
                    .into()
            );
            
            // Create objects for each example with Y offset
            match i {
                0 => {
                    // Rectangle example
                    all_objects.push(
                        fast2d::Rectangle::new()
                            .position(50., y_offset + 50.)
                            .size(200., 150.)
                            .color(50, 0, 100, 1.0)
                            .into()
                    );
                    all_objects.push(
                        fast2d::Text::new()
                            .text("Simple Rectangle")
                            .position(10., y_offset + 50.)
                            .size(150., 50.)
                            .color(255, 255, 255, 1.0)
                            .font_size(16.)
                            .family(fast2d::Family::name("Inter"))
                            .into()
                    );
                }
                1 => {
                    // Face example - simplified version
                    all_objects.push(
                        fast2d::Circle::new()
                            .center(150., y_offset + 100.)
                            .radius(60.)
                            .color(255, 220, 177, 1.0)
                            .into()
                    );
                    all_objects.push(
                        fast2d::Text::new()
                            .text("Face Example")
                            .position(10., y_offset + 10.)
                            .size(150., 50.)
                            .color(255, 255, 255, 1.0)
                            .font_size(16.)
                            .family(fast2d::Family::name("Inter"))
                            .into()
                    );
                }
                2 => {
                    // Sine wave example
                    let mut points = Vec::new();
                    let amplitude = 50.;
                    let frequency = 0.01;
                    let wave_y = y_offset + 150.;
                    let steps = 100;
                    for j in 0..=steps {
                        let x = (j as f32 / steps as f32) * 350.;
                        let y = wave_y + amplitude * (x * frequency * 2. * std::f32::consts::PI).sin();
                        points.push((x, y));
                    }
                    all_objects.push(
                        fast2d::Line::new()
                            .points(&points)
                            .color(0, 255, 255, 1.0)
                            .width(3.)
                            .into()
                    );
                    all_objects.push(
                        fast2d::Text::new()
                            .text("Sine Wave Example")
                            .position(10., y_offset + 10.)
                            .size(150., 50.)
                            .color(255, 255, 255, 1.0)
                            .font_size(16.)
                            .family(fast2d::Family::name("Inter"))
                            .into()
                    );
                }
                _ => {}
            }
            
            // Add a separator line between panels
            if i < 2 { // Don't add separator after last panel
                all_objects.push(
                    fast2d::Line::new()
                        .points(&[(10.0, y_offset + panel_height + 10.0), (640.0, y_offset + panel_height + 10.0)])
                        .color(128, 128, 128, 0.5)
                        .width(1.0)
                        .into()
                );
            }
        }
        
        canvas_wrapper.update_objects(|objects| {
            *objects = all_objects;
        })
    }
    
    /// Standard WGPU resize handler - simple and immediate
    pub fn handle_resize(&mut self, width: u32, height: u32) -> Result<()> {
        // Skip zero dimensions 
        if width == 0 || height == 0 {
            return Ok(());
        }
        
        // Only process if size actually changed
        if self.current_size.0 == width && self.current_size.1 == height {
            return Ok(());
        }
        
        println!("Resizing: {}x{} -> {}x{}", self.current_size.0, self.current_size.1, width, height);
        self.current_size = (width, height);
        
        // Standard pattern: just reconfigure surface, don't draw here
        self.canvas_wrapper.resize_only(width, height);
        
        Ok(())
    }
    
    /// Get current window size
    pub fn window_size(&self) -> (u32, u32) {
        self.current_size
    }
    
    /// Render the current frame  
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Standard WGPU pattern: propagate surface errors to event loop
        match self.canvas_wrapper.render() {
            Ok(_) => Ok(()),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Return surface error so event loop can handle resize
                Err(Box::new(wgpu::SurfaceError::Outdated))
            }
            Err(e) => {
                eprintln!("Render error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
    
    /// Scroll the column view up
    pub fn scroll_up(&mut self) {
        self.scroll_offset = (self.scroll_offset - 40.0).max(0.0);
        if let Err(e) = Self::update_column_layout(&mut self.canvas_wrapper, self.scroll_offset) {
            eprintln!("Error updating layout during scroll: {:?}", e);
        }
        println!("Scrolled up, offset: {}", self.scroll_offset);
    }
    
    /// Scroll the column view down
    pub fn scroll_down(&mut self) {
        let panel_height = 250.0;
        let panel_spacing = 30.0;
        let max_scroll: f32 = 3.0 * (panel_height + panel_spacing) - 500.0; // Allow scrolling to see all content
        self.scroll_offset = (self.scroll_offset + 40.0).min(max_scroll.max(0.0));
        if let Err(e) = Self::update_column_layout(&mut self.canvas_wrapper, self.scroll_offset) {
            eprintln!("Error updating layout during scroll: {:?}", e);
        }
        println!("Scrolled down, offset: {}", self.scroll_offset);
    }
    
    /// Request a redraw
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}