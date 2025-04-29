use crate::Object2d;
use web_sys::HtmlCanvasElement;
use super::Graphics;

pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas: HtmlCanvasElement,
    graphics: Graphics,
}

impl CanvasWrapper {
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

    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        super::draw(&mut self.graphics, &self.objects);
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        super::resize_graphics(&mut self.graphics, width, height);
        super::draw(&mut self.graphics, &self.objects);
    }
}
