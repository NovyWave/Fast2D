use crate::Object2d;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use web_sys::wasm_bindgen::{UnwrapThrowExt, JsCast};

/// A wrapper around an HTML Canvas 2D context, managing a list of 2D objects and rendering them.
pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl CanvasWrapper {
    /// Creates a new `CanvasWrapper` with the given HTML canvas element.
    ///
    /// Initializes the 2D rendering context and prepares for drawing.
    ///
    /// # Arguments
    /// * `canvas` - The HTML canvas element to wrap and render to.
    ///
    /// # Returns
    /// An initialized `CanvasWrapper` instance.
    pub async fn new_with_canvas(canvas: HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap_throw();
        Self {
            objects: Vec::new(),
            canvas,
            context,
        }
    }

    /// Updates the list of 2D objects and redraws the canvas.
    ///
    /// # Arguments
    /// * `updater` - A closure that mutates the internal vector of `Object2d`.
    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        super::draw(&self.context, &self.objects);
    }

    /// Handles resizing of the canvas and redraws the contents.
    ///
    /// # Arguments
    /// * `width` - The new width of the canvas in pixels.
    /// * `height` - The new height of the canvas in pixels.
    pub fn resized(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        super::draw(&self.context, &self.objects);
    }
}
