use crate::Object2d;
use web_sys::HtmlCanvasElement;
use super::Graphics;

/// A wrapper around a GPU-accelerated canvas (WebGPU or WebGL, depending on enabled features),
/// managing a list of 2D objects and rendering them.
pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas: HtmlCanvasElement,
    graphics: Graphics,
}

impl CanvasWrapper {
    /// Creates a new `CanvasWrapper` with the given HTML canvas element.
    ///
    /// Initializes the graphics context and prepares for rendering using the selected GPU backend
    /// (WebGPU or WebGL, depending on enabled features).
    ///
    /// # Arguments
    /// * `canvas` - The HTML canvas element to wrap and render to.
    ///
    /// # Returns
    /// An initialized `CanvasWrapper` instance.
    pub async fn new_with_canvas(canvas: HtmlCanvasElement) -> Self {
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let graphics = super::create_graphics(canvas.clone(), width, height).await;
        Self {
            objects: Vec::new(),
            canvas,
            graphics,
        }
    }

    /// Updates the list of 2D objects and redraws the canvas.
    ///
    /// # Arguments
    /// * `updater` - A closure that mutates the internal vector of `Object2d`.
    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        super::draw(&mut self.graphics, &self.objects);
    }

    /// Handles resizing of the canvas and graphics context, then redraws.
    ///
    /// # Arguments
    /// * `width` - The new width of the canvas in pixels.
    /// * `height` - The new height of the canvas in pixels.
    pub fn resized(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        super::resize_graphics(&mut self.graphics, width, height);
        super::draw(&mut self.graphics, &self.objects);
    }
}
