use crate::Object2d;
use web_sys::HtmlCanvasElement;
#[cfg(feature = "canvas")]
use web_sys::CanvasRenderingContext2d;
#[cfg(feature = "canvas")]
use web_sys::wasm_bindgen::UnwrapThrowExt;
#[cfg(feature = "canvas")]
use web_sys::wasm_bindgen::JsCast;
#[cfg(not(feature = "canvas"))]
use super::Graphics;

/// CanvasWrapper manages a collection of 2D objects and the associated canvas element/context/graphics.
pub struct CanvasWrapper {
    objects: Vec<Object2d>,
    canvas_element: Option<HtmlCanvasElement>,
    #[cfg(feature = "canvas")]
    context: Option<CanvasRenderingContext2d>,
    #[cfg(not(feature = "canvas"))]
    graphics: Option<Graphics>,
}

impl CanvasWrapper {
    pub fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "canvas")] {
                Self {
                    objects: Vec::new(),
                    canvas_element: None,
                    context: None,
                }
            } else {
                Self {
                    objects: Vec::new(),
                    canvas_element: None,
                    graphics: None,
                }
            }
        }
    }

    #[cfg_attr(feature = "canvas", allow(unused_variables))]
    pub async fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        self.canvas_element = Some(canvas.clone());
        cfg_if::cfg_if! {
            if #[cfg(feature = "canvas")] {
                // Get 2D rendering context
                let context_object = canvas
                    .get_context("2d")
                    .unwrap_throw()
                    .unwrap_throw()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap_throw();
                self.context = Some(context_object);
            } else {
                self.graphics = Some(super::create_graphics(canvas, width, height).await);
            }
        }
        self.draw();
    }

    pub fn update_objects(&mut self, updater: impl FnOnce(&mut Vec<Object2d>)) {
        updater(&mut self.objects);
        self.draw();
    }

    #[cfg_attr(feature = "canvas", allow(unused_variables))]
    pub fn resized(&mut self, width: u32, height: u32) {
        if let Some(canvas) = &self.canvas_element {
            canvas.set_width(width);
            canvas.set_height(height);
        }
        cfg_if::cfg_if! {
            if #[cfg(feature = "canvas")] {
                self.draw();
            } else {
                if let Some(graphics) = &mut self.graphics {
                    let new_width = width.max(1);
                    let new_height = height.max(1);
                    graphics.surface_config.width = new_width;
                    graphics.surface_config.height = new_height;
                    graphics.surface.configure(&graphics.device, &graphics.surface_config);
                    graphics.msaa_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("MSAA Texture"),
                        size: wgpu::Extent3d { width: new_width, height: new_height, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: super::MSAA_SAMPLE_COUNT,
                        dimension: wgpu::TextureDimension::D2,
                        format: graphics.surface_config.format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });
                    graphics.viewport.update(&graphics.queue, glyphon::Resolution { width: new_width, height: new_height });
                    let uniforms = super::CanvasUniforms { width: new_width as f32, height: new_height as f32, _padding1: 0.0, _padding2: 0.0 };
                    graphics.queue.write_buffer(&graphics.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
                    self.draw();
                }
            }
        }
    }

    fn draw(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "canvas")] {
                if let Some(context) = &self.context {
                    if let Some(canvas) = &self.canvas_element {
                        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
                        super::draw_canvas(context, &self.objects);
                    }
                }
            } else {
                if let Some(graphics) = &mut self.graphics {
                    super::draw_wgpu(graphics, &self.objects);
                }
            }
        }
    }
}
