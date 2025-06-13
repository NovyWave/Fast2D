// Fast2D CEF Example - Chromium Embedded Framework Integration
//
// This provides a complete alternative to Tauri's WebKitGTK backend using
// Chromium Embedded Framework for reliable WebGL support on all platforms.

use cef::{args::Args, rc::*, sandbox_info::SandboxInfo, sys, Size, Rect, *};
use std::sync::{Arc, Mutex};

const DEV_SERVER_URL: &str = "http://localhost:8080";

struct Fast2DApp {
    object: *mut RcImpl<sys::_cef_app_t, Self>,
    window: Arc<Mutex<Option<Window>>>,
}

impl Fast2DApp {
    fn new(window: Arc<Mutex<Option<Window>>>) -> App {
        App::new(Self {
            object: std::ptr::null_mut(),
            window,
        })
    }
}

impl WrapApp for Fast2DApp {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_app_t, Self>) {
        self.object = object;
    }
}

impl Clone for Fast2DApp {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            self.object
        };
        let window = self.window.clone();

        Self { object, window }
    }
}

impl Rc for Fast2DApp {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplApp for Fast2DApp {
    fn get_raw(&self) -> *mut sys::_cef_app_t {
        self.object.cast()
    }

    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(Fast2DBrowserProcessHandler::new(self.window.clone()))
    }
}

struct Fast2DBrowserProcessHandler {
    object: *mut RcImpl<sys::cef_browser_process_handler_t, Self>,
    window: Arc<Mutex<Option<Window>>>,
}

impl Fast2DBrowserProcessHandler {
    fn new(window: Arc<Mutex<Option<Window>>>) -> BrowserProcessHandler {
        BrowserProcessHandler::new(Self {
            object: std::ptr::null_mut(),
            window,
        })
    }
}

impl Rc for Fast2DBrowserProcessHandler {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapBrowserProcessHandler for Fast2DBrowserProcessHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_browser_process_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for Fast2DBrowserProcessHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        let window = self.window.clone();

        Self { object, window }
    }
}

impl ImplBrowserProcessHandler for Fast2DBrowserProcessHandler {
    fn get_raw(&self) -> *mut sys::_cef_browser_process_handler_t {
        self.object.cast()
    }

    fn on_context_initialized(&self) {
        println!("🚀 CEF context initialized - Loading Fast2D graphics!");
        let mut client = Fast2DClient::new();
        let url = CefString::from(DEV_SERVER_URL);

        let browser_view = browser_view_create(
            Some(&mut client),
            Some(&url),
            Some(&Default::default()),
            Option::<&mut DictionaryValue>::None,
            Option::<&mut RequestContext>::None,
            Option::<&mut BrowserViewDelegate>::None,
        )
        .expect("Failed to create browser view");

        let mut delegate = Fast2DWindowDelegate::new(browser_view);
        if let Ok(mut window) = self.window.lock() {
            *window = Some(
                window_create_top_level(Some(&mut delegate)).expect("Failed to create window"),
            );
        }
    }
}

struct Fast2DClient(*mut RcImpl<sys::_cef_client_t, Self>);

impl Fast2DClient {
    fn new() -> Client {
        Client::new(Self(std::ptr::null_mut()))
    }
}

impl WrapClient for Fast2DClient {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_client_t, Self>) {
        self.0 = object;
    }
}

impl Clone for Fast2DClient {
    fn clone(&self) -> Self {
        unsafe {
            let rc_impl = &mut *self.0;
            rc_impl.interface.add_ref();
        }

        Self(self.0)
    }
}

impl Rc for Fast2DClient {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.0;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplClient for Fast2DClient {
    fn get_raw(&self) -> *mut sys::_cef_client_t {
        self.0.cast()
    }
}

struct Fast2DWindowDelegate {
    base: *mut RcImpl<sys::_cef_window_delegate_t, Self>,
    browser_view: BrowserView,
}

impl Fast2DWindowDelegate {
    fn new(browser_view: BrowserView) -> WindowDelegate {
        WindowDelegate::new(Self {
            base: std::ptr::null_mut(),
            browser_view,
        })
    }
}

impl WrapWindowDelegate for Fast2DWindowDelegate {
    fn wrap_rc(&mut self, object: *mut RcImpl<sys::_cef_window_delegate_t, Self>) {
        self.base = object;
    }
}

impl Clone for Fast2DWindowDelegate {
    fn clone(&self) -> Self {
        unsafe {
            let rc_impl = &mut *self.base;
            rc_impl.interface.add_ref();
        }

        Self {
            base: self.base,
            browser_view: self.browser_view.clone(),
        }
    }
}

impl Rc for Fast2DWindowDelegate {
    fn as_base(&self) -> &sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.base;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplViewDelegate for Fast2DWindowDelegate {
    fn on_child_view_changed(
        &self,
        _view: Option<&mut View>,
        _added: ::std::os::raw::c_int,
        _child: Option<&mut View>,
    ) {
        // Handle view changes if needed
    }

    fn get_raw(&self) -> *mut sys::_cef_view_delegate_t {
        self.base.cast()
    }
}

impl ImplPanelDelegate for Fast2DWindowDelegate {}

impl ImplWindowDelegate for Fast2DWindowDelegate {
    fn on_window_created(&self, window: Option<&mut Window>) {
        if let Some(window) = window {
            println!("🎨 CEF window created - Fast2D graphics loading...");
            let view = self.browser_view.clone();
            window.add_child_view(Some(&mut (&view).into()));
            
            // Set window title and size for visibility
            let title = CefString::from("Fast2D CEF Example - WebGL Graphics");
            window.set_title(Some(&title));
            let size = Size { width: 1200, height: 800 };
            window.set_size(Some(&size));
            
            // Position window on primary monitor (HDMI-A-1: 1920x1080+2048+72)
            // to avoid multi-monitor coordinate issues
            let bounds = Rect { 
                x: 2048 + 360,  // Primary monitor X offset + centering
                y: 72 + 140,    // Primary monitor Y offset + centering
                width: 1200, 
                height: 800 
            };
            window.set_bounds(Some(&bounds));
            
            // Make sure window is visible and on top
            window.show();
            window.activate();
            
            println!("🪟 Window configured: 1200x800, positioned on primary monitor");
        }
    }

    fn on_window_destroyed(&self, _window: Option<&mut Window>) {
        println!("👋 CEF window closed");
        quit_message_loop();
    }

    fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
        1
    }

    fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
        1
    }

    fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
        1
    }

    fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
        1
    }

    fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
        1
    }
}

fn main() -> std::process::ExitCode {
    println!("🚀 Fast2D CEF Example - Chromium Embedded Framework");
    println!();
    
    // Check if MoonZoon dev server is ready
    if !is_server_ready() {
        eprintln!("❌ Error: MoonZoon dev server not available at {}", DEV_SERVER_URL);
        eprintln!("   Please run 'makers mzoon start' in the project root first");
        eprintln!("   Then run this CEF application from the src-cef/ directory");
        return std::process::ExitCode::FAILURE;
    }
    
    println!("✅ MoonZoon dev server ready at {}", DEV_SERVER_URL);
    println!("🎯 Starting CEF with Fast2D WebGL graphics...");
    println!();

    // Initialize CEF
    #[cfg(target_os = "macos")]
    let _loader = {
        let loader = library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
        assert!(loader.load());
        loader
    };

    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();
    let cmd = args.as_cmd_line().unwrap();
    let sandbox = SandboxInfo::new();

    let switch = CefString::from("type");
    let is_browser_process = cmd.has_switch(Some(&switch)) != 1;

    let window = Arc::new(Mutex::new(None));
    let mut app = Fast2DApp::new(window.clone());

    let ret = execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        sandbox.as_mut_ptr(),
    );

    if is_browser_process {
        println!("🌐 Launching CEF browser process...");
        if ret != -1 {
            println!("⚠️  execute_process returned: {}, expected -1 for browser process", ret);
        }
    } else {
        let process_type = CefString::from(&cmd.switch_value(Some(&switch)));
        println!("⚙️  Launching CEF process: {process_type}");
        if ret < 0 {
            println!("⚠️  execute_process returned: {}, expected >= 0 for non-browser process", ret);
        }
        return std::process::ExitCode::SUCCESS;
    }

    let mut settings = Settings::default();
    
    // Disable problematic processes on Linux
    settings.no_sandbox = 1;
    
    // Add command line switches to fix coordinate issues on multi-monitor setup
    let cmd = args.as_cmd_line().unwrap();
    cmd.append_switch(Some(&CefString::from("disable-gpu-sandbox")));
    cmd.append_switch_with_value(Some(&CefString::from("force-device-scale-factor")), Some(&CefString::from("1")));
    cmd.append_switch(Some(&CefString::from("disable-features")));
    cmd.append_switch_with_value(Some(&CefString::from("disable-features")), Some(&CefString::from("VizDisplayCompositor")));
    cmd.append_switch(Some(&CefString::from("use-gl")));
    cmd.append_switch_with_value(Some(&CefString::from("use-gl")), Some(&CefString::from("desktop")));
    
    let init_result = initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        sandbox.as_mut_ptr()
    );
    
    if init_result != 1 {
        eprintln!("❌ CEF initialization failed with code: {}", init_result);
        eprintln!("   This may be due to missing system dependencies or display issues");
        eprintln!("   Try: sudo apt install libx11-dev libgtk-3-dev libxcb1-dev");
        return std::process::ExitCode::FAILURE;
    }

    println!("🎨 CEF initialized - Running message loop...");
    println!("   A CEF window should appear now showing Fast2D graphics!");
    run_message_loop();

    println!("🧹 Cleaning up CEF...");
    if let Ok(window) = window.lock() {
        if let Some(window) = window.as_ref() {
            if !window.has_one_ref() {
                println!("⚠️  Window has multiple references: this is normal");
            }
        } else {
            println!("⚠️  Window was None during cleanup");
        }
    }

    shutdown();
    println!("✅ CEF shutdown complete");
    
    std::process::ExitCode::SUCCESS
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


