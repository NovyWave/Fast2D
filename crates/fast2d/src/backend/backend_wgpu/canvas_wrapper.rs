use crate::Object2d;
use web_sys::HtmlCanvasElement;
use crate::backend::Graphics;

/// CanvasWrapper manages a collection of 2D objects and the associated canvas element/graphics for WGPU/WebGL.
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
        self.graphics = Some(crate::backend::create_graphics(canvas, width, height).await);
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
            let new_width = width.max(1);
            let new_height = height.max(1);
            graphics.surface_config.width = new_width;
            graphics.surface_config.height = new_height;
            graphics.surface.configure(&graphics.device, &graphics.surface_config);
            graphics.msaa_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Texture"),
                size: wgpu::Extent3d { width: new_width, height: new_height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: crate::backend::MSAA_SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: graphics.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            graphics.viewport.update(&graphics.queue, glyphon::Resolution { width: new_width, height: new_height });
            let uniforms = crate::backend::CanvasUniforms { width: new_width as f32, height: new_height as f32, _padding1: 0.0, _padding2: 0.0 };
            graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
            self.draw();
        }
    }

    fn draw(&mut self) {
        if let Some(graphics) = self.graphics.as_mut() {
            crate::backend::draw_wgpu(graphics, &self.objects);
        }
    }
}
