use crate::Object2d;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use web_sys::wasm_bindgen::{UnwrapThrowExt, JsCast};

pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas_element: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl CanvasWrapper {
    pub async fn new_with_canvas(canvas: HtmlCanvasElement) -> Self {
        let context_object = canvas
            .get_context("2d")
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap_throw();
        Self {
            objects: Vec::new(),
            canvas_element: canvas,
            context: context_object,
        }
    }

    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        super::draw(&self.context, &self.objects);
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.canvas_element.set_width(width);
        self.canvas_element.set_height(height);
        super::draw(&self.context, &self.objects);
    }
}
