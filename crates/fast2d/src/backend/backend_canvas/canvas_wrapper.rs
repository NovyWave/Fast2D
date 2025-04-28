use crate::Object2d;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use web_sys::wasm_bindgen::{UnwrapThrowExt, JsCast};

pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas_element: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
}

impl CanvasWrapper {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            canvas_element: None,
            context: None,
        }
    }

    pub async fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        self.canvas_element = Some(canvas.clone());
        let context_object = canvas
            .get_context("2d")
            .unwrap_throw()
            .unwrap_throw()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap_throw();
        self.context = Some(context_object);
        self.draw();
    }

    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        self.draw();
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        if let Some(canvas) = &self.canvas_element {
            canvas.set_width(width);
            canvas.set_height(height);
        }
        self.draw();
    }

    fn draw(&mut self) {
        if let Some(context) = &self.context {
            if let Some(canvas) = &self.canvas_element {
                context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
                super::draw_canvas(context, &self.objects);
            }
        }
    }
}
