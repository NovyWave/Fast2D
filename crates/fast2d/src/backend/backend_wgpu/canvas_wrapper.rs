use crate::Object2d;
use web_sys::HtmlCanvasElement;
use super::Graphics;

pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas_element: Option<HtmlCanvasElement>,
    graphics: Option<Graphics>,
}

impl CanvasWrapper {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            canvas_element: None,
            graphics: None,
        }
    }

    pub async fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        self.canvas_element = Some(canvas.clone());
        self.graphics = Some(super::create_graphics(canvas, width, height).await);
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
        if let Some(graphics) = self.graphics.as_mut() {
            super::resize_graphics(graphics, width, height);
            self.draw();
        }
    }

    fn draw(&mut self) {
        if let Some(graphics) = self.graphics.as_mut() {
            super::draw(graphics, &self.objects);
        }
    }
}
