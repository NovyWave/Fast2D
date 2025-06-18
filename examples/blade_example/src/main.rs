use anyhow::Result;
use tao::{
    event::{Event, WindowEvent, MouseScrollDelta},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod blade_app;
mod blade_examples;

use blade_app::BladeApp;

/// Entry point: Blade Graphics + Tao windowing experiment
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔪 Starting Blade + Fast2D Experiment...");
    
    // Load and register fonts (same as WGPU version)
    load_and_register_fonts().await?;
    
    // Create event loop and window (unchanged from Tao example)
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("🔺 MAGENTA TRIANGLE - LOOK FOR THIS WINDOW")
        .with_inner_size(tao::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;
    
    // Create the Blade-powered app
    let mut app = BladeApp::new(window).await?;
    
    println!("✅ Blade Graphics initialized successfully!");
    println!("🎯 Fast2D Blade Experiment - Testing resize stability");
    println!("Controls:");
    println!("  Mouse Wheel - Scroll up/down to see all examples");
    println!("  Window Resize - Main test: smooth without white flashing");
    println!("  Close window to exit.");
    
    // Request initial draw immediately
    app.request_redraw();
    
    // Also request redraw after a delay to ensure it happens
    std::thread::sleep(std::time::Duration::from_millis(100));
    app.request_redraw();
    
    // Event loop - use Wait instead of Poll to reduce CPU usage and frame pressure
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
                        // CRITICAL TEST: Blade resize handling
                        println!("🔍 Testing Blade resize: {:?}", physical_size);
                        if let Err(e) = app.handle_resize(physical_size.width, physical_size.height) {
                            eprintln!("❌ Blade resize error: {}", e);
                        } else {
                            println!("✅ Blade resize successful");
                            app.request_redraw(); // Request redraw after successful resize
                        }
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        if let Err(e) = app.handle_resize(new_inner_size.width, new_inner_size.height) {
                            eprintln!("❌ Blade scale change error: {}", e);
                        }
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
                // TEST: Blade rendering stability
                match app.render() {
                    Ok(_) => {
                        // Success - no logging to reduce noise
                    }
                    Err(e) => {
                        eprintln!("🔍 Blade render error: {}", e);
                        // TODO: Implement Blade-specific error recovery
                        // Goal: Simpler than WGPU's complex error handling
                    }
                }
            }
            Event::MainEventsCleared => {
                // With Wait mode, this should rarely trigger
                // Only request a redraw occasionally for testing
                static mut FRAME_COUNT: u32 = 0;
                unsafe {
                    FRAME_COUNT += 1;
                    if FRAME_COUNT % 1000 == 0 { // Much less frequent with Wait mode
                        println!("🎯 Occasional redraw request #{}", FRAME_COUNT / 1000);
                        app.request_redraw();
                    }
                }
            }
            _ => {}
        }
    });
}

/// Load fonts for text rendering - reuse from WGPU version
async fn load_and_register_fonts() -> Result<()> {
    println!("Loading embedded fonts for Blade backend...");
    
    // TODO: Determine if Blade backend can reuse Fast2D font system
    // Or if custom implementation needed
    
    // For now, use existing Fast2D font loading
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
    
    // TODO: Verify compatibility with Blade backend
    fast2d::register_fonts(fonts)?;
    println!("✅ Fonts registered for Blade backend!");
    
    Ok(())
}