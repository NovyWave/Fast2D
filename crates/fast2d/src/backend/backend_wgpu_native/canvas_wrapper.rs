use crate::Object2d;
use super::Graphics;
use wgpu::{Adapter, Device, Queue, Surface};

/// A wrapper around a GPU-accelerated native surface, managing a list of 2D objects and rendering them.
/// 
/// This is the native equivalent of the web CanvasWrapper, providing the same API but working with 
/// native WGPU surfaces instead of HTML canvas elements.
pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    graphics: Graphics,
    width: u32,
    height: u32,
}

impl CanvasWrapper {
    /// Creates a new `CanvasWrapper` with a native WGPU surface.
    ///
    /// This is the native alternative to `new_with_canvas()`. It initializes the graphics context 
    /// and prepares for rendering using native WGPU on the desktop.
    ///
    /// # Arguments
    /// * `surface` - The WGPU surface to render to (created from a native window)
    /// * `device` - The WGPU device (graphics card handle)  
    /// * `queue` - The WGPU queue (command submission)
    /// * `adapter` - The WGPU adapter (needed for surface capabilities)
    /// * `width` - Initial width of the surface in pixels
    /// * `height` - Initial height of the surface in pixels
    ///
    /// # Returns
    /// An initialized `CanvasWrapper` instance ready for native rendering.
    pub async fn new_with_surface(
        surface: Surface<'static>,
        device: Device,
        queue: Queue,
        adapter: Adapter, 
        width: u32,
        height: u32,
    ) -> Self {
        let graphics = super::create_graphics_with_adapter(surface, device, queue, adapter, width, height).await;
        Self {
            objects: Vec::new(),
            graphics,
            width,
            height,
        }
    }

    // TODO: Add convenience method for creating from window handle once the core API is stabilized

    /// Creates a new `CanvasWrapper` with a native WGPU surface and adapter.
    ///
    /// This is an internal method that includes the adapter for getting surface capabilities.
    ///
    /// # Arguments
    /// * `surface` - The WGPU surface to render to (created from a native window)
    /// * `device` - The WGPU device (graphics card handle)  
    /// * `queue` - The WGPU queue (command submission)
    /// * `adapter` - The WGPU adapter (needed for surface capabilities)
    /// * `width` - Initial width of the surface in pixels
    /// * `height` - Initial height of the surface in pixels
    ///
    /// # Returns
    /// An initialized `CanvasWrapper` instance ready for native rendering.
    async fn new_with_surface_and_adapter(
        surface: Surface<'static>,
        device: Device,
        queue: Queue,
        adapter: wgpu::Adapter,
        width: u32,
        height: u32,
    ) -> Self {
        let graphics = super::create_graphics_with_adapter(surface, device, queue, adapter, width, height).await;
        Self {
            objects: Vec::new(),
            graphics,
            width,
            height,
        }
    }

    /// Updates the list of 2D objects and redraws the surface.
    ///
    /// This method provides the same API as the web version, ensuring compatibility.
    ///
    /// # Arguments
    /// * `updater` - A closure that mutates the internal vector of `Object2d`.
    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) -> Result<(), wgpu::SurfaceError> {
        updater(&mut self.objects);
        super::draw(&mut self.graphics, &self.objects)
    }

    /// Handles resizing of the surface and graphics context, then redraws.
    ///
    /// This method provides the same API as the web version, ensuring compatibility.
    ///
    /// # Arguments
    /// * `width` - The new width of the surface in pixels.
    /// * `height` - The new height of the surface in pixels.
    pub fn resized(&mut self, width: u32, height: u32) -> Result<(), wgpu::SurfaceError> {
        self.width = width;
        self.height = height;
        super::resize_graphics(&mut self.graphics, width, height);
        super::draw(&mut self.graphics, &self.objects)
    }

    /// Handles resizing of the surface and graphics context WITHOUT redrawing.
    ///
    /// This is useful when you want to batch resize + layout updates into a single draw call.
    ///
    /// # Arguments
    /// * `width` - The new width of the surface in pixels.
    /// * `height` - The new height of the surface in pixels.
    pub fn resize_only(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        super::resize_graphics(&mut self.graphics, width, height);
    }

    /// Gets the current width of the surface.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Gets the current height of the surface.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Gets a reference to the underlying graphics context.
    /// 
    /// This can be useful for advanced use cases that need direct access to WGPU resources.
    pub fn graphics(&self) -> &Graphics {
        &self.graphics
    }

    /// Gets a mutable reference to the underlying graphics context.
    /// 
    /// This can be useful for advanced use cases that need direct access to WGPU resources.
    pub fn graphics_mut(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    /// Gets a reference to the current objects being rendered.
    pub fn objects(&self) -> &Vec<Object2d> {
        &self.objects
    }

    /// Renders the current objects without updating them.
    /// 
    /// This method is useful for continuous rendering without modifying the object list.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        super::draw(&mut self.graphics, &self.objects)
    }
}