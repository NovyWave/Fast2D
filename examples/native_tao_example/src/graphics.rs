use anyhow::Result;
use std::sync::Arc;
use tao::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

/// Graphics context managing WGPU setup
pub struct GraphicsContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Surface,
    pub surface_config: SurfaceConfiguration,
}

impl GraphicsContext {
    /// Create a new graphics context
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        // Create WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // Create surface
        let surface = unsafe { instance.create_surface(&*window)? };
        
        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate adapter"))?;
        
        // Request device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;
        
        // Configure surface
        let window_size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        
        surface.configure(&device, &surface_config);
        
        println!("Graphics context created successfully!");
        println!("  Surface format: {:?}", surface_format);
        println!("  Surface size: {}x{}", window_size.width, window_size.height);
        
        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface,
            surface_config,
        })
    }
    
    /// Resize the graphics context
    pub async fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            println!("Graphics resized to: {}x{}", width, height);
        }
        Ok(())
    }
}