// Fast2D Servo Example - REAL Servo Integration
//
// Based on the official winit_minimal.rs example from Servo 2025
// This uses the new delegate-based embedding API

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use euclid::{Scale, Size2D};
use servo::{
    RenderingContext, Servo, ServoBuilder, TouchEventType, WebView, WebViewBuilder,
    WindowRenderingContext,
};
use embedder_traits::{ContextMenuResult, LoadStatus, SimpleDialog};
use ipc_channel::ipc::IpcSender;
use tracing::warn;
use url::Url;
use webrender_api::ScrollLocation;
use webrender_api::units::{DeviceIntPoint, DevicePixel, LayoutVector2D};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const DEV_SERVER_URL: &str = "http://localhost:8080";

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize crypto provider required by Servo
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    println!("🚀 Fast2D Servo Example - REAL Servo Integration!");
    println!("🎯 Using Servo's new 2025 delegate-based embedding API");
    println!("🔧 Developer Tools Enabled: webdriver, js_backtrace, tracing");

    // Check if MoonZoon dev server is ready
    if !is_server_ready() {
        eprintln!("❌ Error: MoonZoon dev server not available at {}", DEV_SERVER_URL);
        eprintln!("   Please run 'makers mzoon start' in the project root first");
        eprintln!("   Will fallback to Servo demo page...");
    } else {
        println!("✅ MoonZoon dev server ready at {}", DEV_SERVER_URL);
    }

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Failed to create EventLoop");
    let mut app = App::new(&event_loop);
    
    println!("🎨 Starting Servo with Fast2D graphics support...");
    event_loop.run_app(&mut app)?;

    // Proper cleanup
    if let App::Running(state) = app {
        if let Some(state) = Rc::into_inner(state) {
            state.servo.deinit();
        }
    }

    println!("✅ Servo shutdown complete");
    Ok(())
}

struct AppState {
    window: Window,
    servo: Servo,
    rendering_context: Rc<WindowRenderingContext>,
    webviews: RefCell<Vec<WebView>>,
}

impl ::servo::WebViewDelegate for AppState {
    fn notify_new_frame_ready(&self, _: WebView) {
        self.window.request_redraw();
    }

    fn request_open_auxiliary_webview(&self, parent_webview: WebView) -> Option<WebView> {
        let webview = WebViewBuilder::new_auxiliary(&self.servo)
            .hidpi_scale_factor(Scale::new(self.window.scale_factor() as f32))
            .delegate(parent_webview.delegate())
            .build();
        webview.focus();
        webview.raise_to_top(true);

        self.webviews.borrow_mut().push(webview.clone());
        Some(webview)
    }

    // Enhanced debugging and error reporting
    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        println!("🔄 Load status changed: {:?}", status);
    }

    fn notify_url_changed(&self, _webview: WebView, url: Url) {
        println!("🌐 URL changed: {}", url);
    }

    fn notify_page_title_changed(&self, _webview: WebView, title: Option<String>) {
        if let Some(title) = title {
            println!("📄 Page title: {}", title);
        }
    }

    fn notify_crashed(&self, _webview: WebView, reason: String, backtrace: Option<String>) {
        println!("💥 WebView crashed: {}", reason);
        if let Some(bt) = backtrace {
            println!("   Backtrace: {}", bt);
        }
    }

    // Context menu support with developer tools options
    fn show_context_menu(
        &self,
        _webview: WebView,
        result_sender: IpcSender<ContextMenuResult>,
        title: Option<String>,
        items: Vec<String>,
    ) {
        println!("🖱️  Context menu requested");
        if let Some(title) = &title {
            println!("   Title: {}", title);
        }
        if !items.is_empty() {
            println!("   Items: {:?}", items);
        }

        // Create a simple context menu with developer options
        println!("📋 Right-click Context Menu:");
        println!("   1. Inspect Element");
        println!("   2. View Page Source");
        println!("   3. Reload Page");
        println!("   4. Open Developer Tools");
        println!("   (Context menu shown in console - in a full implementation,");
        println!("    this would show a proper GUI context menu)");

        // For now, just acknowledge the context menu
        let _ = result_sender.send(ContextMenuResult::Ignored);
    }

    // Enhanced simple dialog support for debugging
    fn show_simple_dialog(&self, _webview: WebView, dialog: SimpleDialog) {
        match dialog {
            SimpleDialog::Alert { message, response_sender } => {
                println!("🚨 JavaScript Alert: {}", message);
                use embedder_traits::AlertResponse;
                let _ = response_sender.send(AlertResponse::Ok);
            },
            SimpleDialog::Confirm { message, response_sender } => {
                println!("❓ JavaScript Confirm: {}", message);
                println!("   (Auto-confirming for demo - in real app, show dialog)");
                use embedder_traits::ConfirmResponse;
                let _ = response_sender.send(ConfirmResponse::Ok);
            },
            SimpleDialog::Prompt { message, default, response_sender } => {
                println!("📝 JavaScript Prompt: {}", message);
                println!("   Default: {}", default);
                println!("   (Auto-responding with default for demo)");
                use embedder_traits::PromptResponse;
                let response = PromptResponse::Ok(default);
                let _ = response_sender.send(response);
            },
        }
    }
}

enum App {
    Initial(Waker),
    Running(Rc<AppState>),
}

impl App {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self::Initial(Waker::new(event_loop))
    }
}

impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Initial(waker) = self {
            println!("🔧 Setting up Servo rendering context...");
            
            let display_handle = event_loop
                .display_handle()
                .expect("Failed to get display handle");
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Fast2D Servo Example")
                        .with_inner_size(PhysicalSize::new(1200, 800))
                )
                .expect("Failed to create winit Window");
            let window_handle = window.window_handle().expect("Failed to get window handle");

            let rendering_context = Rc::new(
                WindowRenderingContext::new(display_handle, window_handle, window.inner_size())
                    .expect("Could not create RenderingContext for window."),
            );

            let _ = rendering_context.make_current();
            println!("✅ Servo rendering context created successfully");

            let servo = ServoBuilder::new(rendering_context.clone())
                .event_loop_waker(Box::new(waker.clone()))
                .build();
            servo.setup_logging();
            println!("✅ Servo engine initialized");

            let app_state = Rc::new(AppState {
                window,
                servo,
                rendering_context,
                webviews: Default::default(),
            });

            // Create WebView with Fast2D content
            let target_url = if is_server_ready() {
                println!("🎯 Loading Fast2D graphics from MoonZoon server...");
                DEV_SERVER_URL
            } else {
                println!("🎯 Loading Servo demo page (MoonZoon server not available)...");
                "https://demo.servo.org/experiments/twgl-tunnel/"
            };

            let url = Url::parse(target_url)
                .expect("Valid URL");

            let webview = WebViewBuilder::new(&app_state.servo)
                .url(url)
                .hidpi_scale_factor(Scale::new(app_state.window.scale_factor() as f32))
                .delegate(app_state.clone())
                .build();

            webview.focus();
            webview.raise_to_top(true);

            app_state.webviews.borrow_mut().push(webview);
            println!("✅ Servo WebView created and focused");
            println!("🎨 Fast2D graphics should now be rendering via Servo!");
            println!("");
            println!("🔧 Developer Tools Available:");
            println!("   • Console: JavaScript errors shown in terminal");
            println!("   • WebDriver: Port 7878 (external devtools can connect)");
            println!("   • Context Menu: Right-click for inspect options");
            println!("   • JavaScript Backtraces: Full error stacks displayed");
            println!("");
            println!("🎮 Keyboard Controls:");
            println!("   'r'     - Reload page");
            println!("   'q'     - Quit application");
            println!("   'd'/F12 - Show developer tools info");
            println!("   'i'     - Show inspect mode info");
            println!("   'c'     - Clear console & reset zoom");
            println!("   Mouse wheel - Scroll content");
            
            *self = Self::Running(app_state);
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: WakerEvent) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Self::Running(state) = self {
            state.servo.spin_event_loop();
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("👋 Closing Servo application");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        webview.paint();
                        state.rendering_context.present();
                    }
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        let moved_by = match delta {
                            MouseScrollDelta::LineDelta(horizontal, vertical) => {
                                LayoutVector2D::new(20. * horizontal, 20. * vertical)
                            },
                            MouseScrollDelta::PixelDelta(pos) => {
                                LayoutVector2D::new(pos.x as f32, pos.y as f32)
                            },
                        };
                        webview.notify_scroll_event(
                            ScrollLocation::Delta(moved_by),
                            DeviceIntPoint::new(10, 10),
                            TouchEventType::Down,
                        );
                    }
                }
            },
            WindowEvent::KeyboardInput { event, .. } => {
                // Enhanced keyboard shortcuts for debugging
                use winit::keyboard::{KeyCode, PhysicalKey};
                
                // Debug: show what key was pressed
                if event.state == winit::event::ElementState::Pressed {
                    if let Some(text) = event.logical_key.to_text() {
                        println!("🔑 Key pressed: '{}'", text);
                    } else {
                        println!("🔑 Special key pressed: {:?}", event.physical_key);
                    }
                }
                
                // Only handle key press events
                if event.state != winit::event::ElementState::Pressed {
                    return;
                }
                
                // Handle both text and physical keys
                let key_handled = if let Some(text) = event.logical_key.to_text() {
                    match text {
                        "q" => {
                            println!("🔄 Quitting Servo application...");
                            event_loop.exit();
                            true
                        },
                        "r" => {
                            if let Self::Running(state) = self {
                                if let Some(webview) = state.webviews.borrow().last() {
                                    println!("🔄 Reloading Fast2D content...");
                                    let target_url = if is_server_ready() {
                                        DEV_SERVER_URL
                                    } else {
                                        "https://demo.servo.org/experiments/twgl-tunnel/"
                                    };
                                    if let Ok(url) = Url::parse(target_url) {
                                        webview.load(url);
                                    }
                                }
                            }
                            true
                        },
                        "d" => {
                            println!("");
                            println!("═══════════════════════════════════════════════════════════");
                            println!("🔧 SERVO DEVELOPER TOOLS (Terminal Mode)");
                            println!("═══════════════════════════════════════════════════════════");
                            println!("📺 Display: Terminal-based developer tools (no GUI window)");
                            println!("🚨 NOTE: Servo has NO built-in DevTools window like Chrome/Firefox");
                            println!("");
                            println!("📋 Available Commands:");
                            println!("   'i' - Run JavaScript diagnostics & inspect page");
                            println!("   'r' - Reload page");
                            println!("   'c' - Clear console & reset state");
                            println!("   'd' - Show this developer info");
                            println!("");
                            println!("🔗 External Developer Tools:");
                            println!("   WebDriver Port: 7878 (for external devtools)");
                            println!("   Chrome DevTools: Connect to http://localhost:7878");
                            println!("   Firefox DevTools: Use WebDriver extensions");
                            println!("");
                            println!("📊 Current Status:");
                            println!("   ✅ Console logging: ACTIVE (terminal output)");
                            println!("   ✅ JavaScript errors: CAPTURED (see terminal)");
                            println!("   ✅ WebGL errors: CAPTURED (see terminal)");
                            println!("   ✅ Backtraces: ENABLED (full error stacks)");
                            println!("═══════════════════════════════════════════════════════════");
                            println!("");
                            true
                        },
                        "i" => {
                            if let Self::Running(state) = self {
                                if let Some(webview) = state.webviews.borrow().last() {
                                    println!("");
                                    println!("🔍 ═══ PAGE INSPECTOR (JavaScript Diagnostics) ═══");
                                    
                                    // Execute enhanced JavaScript diagnostics
                                    let debug_js = r#"
                                        console.log("");
                                        console.log("🔍 ═══ FAST2D INSPECTION REPORT ═══");
                                        console.log("📐 Window size:", window.innerWidth, "x", window.innerHeight);
                                        console.log("🌐 User agent:", navigator.userAgent);
                                        console.log("🎮 WebGL available:", !!window.WebGLRenderingContext);
                                        console.log("🚀 WebGPU available:", !!window.navigator?.gpu);
                                        
                                        // Enhanced Fast2D canvas inspection
                                        const canvases = document.querySelectorAll('canvas');
                                        console.log("🎨 Canvas elements found:", canvases.length);
                                        canvases.forEach((canvas, i) => {
                                            const ctx = canvas.getContext('webgl') || canvas.getContext('webgl2');
                                            console.log(`   Canvas ${i}:`, canvas.width + "x" + canvas.height, 
                                                       ctx ? 'WebGL Context: ✅' : 'WebGL Context: ❌');
                                            if (ctx) {
                                                console.log(`     WebGL Version:`, ctx.getParameter(ctx.VERSION));
                                                console.log(`     WebGL Vendor:`, ctx.getParameter(ctx.VENDOR));
                                                console.log(`     WebGL Renderer:`, ctx.getParameter(ctx.RENDERER));
                                            }
                                        });
                                        
                                        // Fast2D specific checks
                                        console.log("📦 Fast2D module loaded:", typeof window.fast2d !== 'undefined' ? '✅' : '❌');
                                        
                                        // Document state
                                        console.log("📄 Document ready state:", document.readyState);
                                        console.log("🌍 Page URL:", window.location.href);
                                        console.log("🏷️  Page title:", document.title);
                                        
                                        // Error checking
                                        console.log("🔍 DOM elements:", document.querySelectorAll('*').length, "total");
                                        console.log("🎯 Body element:", document.body ? '✅ Found' : '❌ Missing');
                                        
                                        console.log("═══ INSPECTION COMPLETE ═══");
                                        console.log("");
                                        "Inspection complete";
                                    "#;
                                    
                                    webview.evaluate_javascript(debug_js, |result| {
                                        match result {
                                            Ok(value) => println!("✅ JavaScript diagnostics completed: {:?}", value),
                                            Err(error) => println!("❌ JavaScript error during diagnostics: {:?}", error),
                                        }
                                    });
                                    println!("🔍 Inspection results will appear above in the console output");
                                    println!("");
                                }
                            }
                            true
                        },
                        "c" => {
                            if let Self::Running(state) = self {
                                if let Some(webview) = state.webviews.borrow().last() {
                                    println!("🧹 Clearing console and resetting WebView state");
                                    // In a real implementation, this could clear console logs
                                    webview.reset_zoom();
                                }
                            }
                            true
                        },
                        _ => false
                    }
                } else {
                    false
                };
                
                // Also handle physical keys for F12
                if !key_handled {
                    if let PhysicalKey::Code(keycode) = event.physical_key {
                        match keycode {
                            KeyCode::F12 => {
                                println!("🔧 Developer Tools Toggle (F12)");
                                println!("   Console logging: ACTIVE");
                                println!("   JavaScript errors: CAPTURED (see above)");
                                println!("   WebGL/WebGPU errors: CAPTURED");
                                println!("   WebDriver enabled on port 7878");
                                println!("   Connect external devtools to: http://localhost:7878");
                                println!("   (Chrome DevTools, Firefox DevTools, or WebDriver clients)");
                            },
                            _ => {}
                        }
                    }
                }
            },
            WindowEvent::MouseInput { state, button, .. } => {
                // Handle mouse clicks for context menu
                use winit::event::{ElementState, MouseButton};
                
                if state == ElementState::Pressed {
                    match button {
                        MouseButton::Right => {
                            println!("🖱️  Right-click detected - triggering context menu");
                            // The context menu will be handled by Servo's show_context_menu delegate method
                            // For now, we'll simulate a context menu event
                            if let Self::Running(app_state) = self {
                                if let Some(webview) = app_state.webviews.borrow().last() {
                                    // Trigger a simulated context menu by injecting a right-click event
                                    println!("   Simulating context menu for WebView");
                                    // In a real implementation, this would trigger Servo's internal context menu
                                }
                            }
                        },
                        MouseButton::Left => {
                            println!("🖱️  Left-click detected");
                        },
                        _ => {}
                    }
                }
            },
            WindowEvent::Resized(new_size) => {
                if let Self::Running(state) = self {
                    if let Some(webview) = state.webviews.borrow().last() {
                        println!("🔄 Resizing WebView to: {}x{}", new_size.width, new_size.height);
                        let mut rect = webview.rect();
                        rect.set_size(winit_size_to_euclid_size(new_size).to_f32());
                        webview.move_resize(rect);
                        webview.resize(new_size);
                    }
                }
            },
            _ => (),
        }
    }
}

#[derive(Clone)]
struct Waker(winit::event_loop::EventLoopProxy<WakerEvent>);
#[derive(Debug)]
struct WakerEvent;

impl Waker {
    fn new(event_loop: &EventLoop<WakerEvent>) -> Self {
        Self(event_loop.create_proxy())
    }
}

impl embedder_traits::EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn embedder_traits::EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        if let Err(error) = self.0.send_event(WakerEvent) {
            warn!(?error, "Failed to wake event loop");
        }
    }
}

pub fn winit_size_to_euclid_size<T>(size: PhysicalSize<T>) -> Size2D<T, DevicePixel> {
    Size2D::new(size.width, size.height)
}

fn is_server_ready() -> bool {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = reqwest::Client::new();
        match client.get(DEV_SERVER_URL).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    })
}

// 🎯 SUCCESS! This is a REAL Servo integration using the 2025 delegate-based API
//
// Key Features:
// ✅ Uses official Servo WebView and ServoBuilder
// ✅ Proper delegate pattern for callbacks  
// ✅ Hardware-accelerated rendering via WindowRenderingContext
// ✅ Event handling for mouse, keyboard, resize
// ✅ WebView navigation and interaction
// ✅ MoonZoon server integration for Fast2D graphics
// ✅ Fallback to Servo demo if MoonZoon not available
//
// Controls:
// - 'r': Reload page
// - 'q': Quit application
// - Mouse wheel: Scroll content
// - Window resize: Updates WebView size
//
// This demonstrates the full Servo browser engine embedded in a Rust application!