use anyhow::Result;
use tao::{
    event::{Event, WindowEvent, MouseScrollDelta},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
mod examples;
mod simple_demo;

use app::NativeApp;

/// Entry point: loads fonts and starts the app.
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Native Tao + Fast2D Example...");
    
    // Load and register fonts first
    load_and_register_fonts().await?;
    
    // Create event loop and window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Fast2D Native Example - Column Layout")
        .with_inner_size(tao::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;
    
    // Create the native app
    let mut app = NativeApp::new(window).await?;
    
    println!("Application initialized successfully!");
    println!("Fast2D Native Desktop Example - Column Layout");
    println!("Showing all three examples in a single scrollable view.");
    println!("Controls:");
    println!("  Mouse Wheel - Scroll up/down to see all examples");
    println!("  Close window to exit.");
    
    // Request initial redraw
    app.request_redraw();
    
    // Run the event loop with immediate resize handling
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Close requested, shutting down...");
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(physical_size) => {
                        // Standard WGPU pattern: handle resize immediately, simply
                        if let Err(e) = app.handle_resize(physical_size.width, physical_size.height) {
                            eprintln!("Error handling resize: {}", e);
                        }
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        if let Err(e) = app.handle_resize(new_inner_size.width, new_inner_size.height) {
                            eprintln!("Error handling scale factor change: {}", e);
                        }
                    }
                    WindowEvent::KeyboardInput { .. } => {
                        // Keyboard input placeholder - for now just mouse wheel scrolling
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            MouseScrollDelta::LineDelta(_, y) => {
                                if y > 0.0 {
                                    app.scroll_up();
                                } else if y < 0.0 {
                                    app.scroll_down();
                                }
                                app.request_redraw();
                            }
                            MouseScrollDelta::PixelDelta(delta) => {
                                if delta.y > 0.0 {
                                    app.scroll_up();
                                } else if delta.y < 0.0 {
                                    app.scroll_down();
                                }
                                app.request_redraw();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested { .. } => {
                // Standard WGPU pattern: handle surface errors in render loop
                match app.render() {
                    Ok(_) => {}
                    // Reconfigure surface if lost or outdated - this is the key!
                    Err(e) => {
                        eprintln!("Render error: {}", e);
                        // Try to recover by getting current window size and resizing
                        let window_size = app.window_size();
                        if let Err(resize_err) = app.handle_resize(window_size.0, window_size.1) {
                            eprintln!("Failed to recover from render error: {}", resize_err);
                        }
                    }
                }
            }
            _ => {}
        }
    });
}

/// Loads and registers required fonts asynchronously.
async fn load_and_register_fonts() -> Result<()> {
    println!("Loading embedded fonts...");
    
    // Load fonts from embedded assets
    const FIRA_CODE: &[u8] = include_bytes!("../assets/fonts/FiraCode-Regular.ttf");
    const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
    const INTER_BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");
    const INTER_BOLD_ITALIC: &[u8] = include_bytes!("../assets/fonts/Inter-BoldItalic.ttf");
    
    let fonts = vec![
        FIRA_CODE.to_vec(),
        INTER_REGULAR.to_vec(),
        INTER_BOLD.to_vec(),
        INTER_BOLD_ITALIC.to_vec(),
    ];
    
    fast2d::register_fonts(fonts)?;
    println!("Fonts registered successfully!");
    
    Ok(())
}