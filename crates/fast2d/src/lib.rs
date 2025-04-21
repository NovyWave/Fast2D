pub use zoon;

use zoon::wasm_bindgen::throw_str;
use zoon::web_sys::HtmlCanvasElement;
use zoon::Task;
use zoon::UnwrapThrowExt;

use std::future::Future;
use std::sync::Arc;
use std::borrow::Cow;

use glyphon::{
    fontdb, Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};

use wgpu::{Device, MultisampleState, Queue, Surface, SurfaceConfiguration, SurfaceTarget};

const CANVAS_WIDTH: u32 = 350;
const CANVAS_HEIGHT: u32 = 350;

pub struct Text {
    text: Cow<'static, str>,
}

impl Text {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            text: text.into(),
        }
    }
}

impl Into<Object2d> for Text {
    fn into(self) -> Object2d {
        Object2d::Text(self)
    }
}

pub enum Object2d {
    Text(Text),
}

pub fn run(canvas: HtmlCanvasElement, objects: Vec<Object2d>) {
    Task::start(async move {
        let mut graphics = create_graphics(canvas).await;
        draw(&mut graphics)
    });
}

fn create_graphics(
    canvas: HtmlCanvasElement,
) -> impl Future<Output = Graphics> + 'static {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(SurfaceTarget::Canvas(canvas))
        .unwrap_or_else(|e| throw_str(&format!("{e:#?}")));

    async move {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap_throw();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WGPU Device"),
                    memory_hints: wgpu::MemoryHints::default(),
                    required_features: wgpu::Features::default(),
                    #[cfg(feature = "webgpu")]
                    required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
                    #[cfg(feature = "webgl")]
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .unwrap_throw();

        let surface_config = surface
            .get_default_config(&adapter, CANVAS_WIDTH, CANVAS_HEIGHT)
            .unwrap_throw();

        surface.configure(&device, &surface_config);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        // Set up text renderer
        let mut font_system = {
            // NOTE: Smaller and compressed font would be probably better
            let font_data = include_bytes!("../fonts/FiraCode-Regular.ttf");
            FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(font_data))])
        };
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        text_buffer.set_text(
            &mut font_system,
            "Hello world!",
            &Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        Graphics {
            device,
            queue,
            surface,
            surface_config,

            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
        }
    }
}

fn draw(gfx: &mut Graphics) {
    gfx.viewport.update(
        &gfx.queue,
        Resolution {
            width: gfx.surface_config.width,
            height: gfx.surface_config.height,
        },
    );

    gfx.text_renderer
        .prepare(
            &gfx.device,
            &gfx.queue,
            &mut gfx.font_system,
            &mut gfx.atlas,
            &gfx.viewport,
            [TextArea {
                buffer: &gfx.text_buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 600,
                    bottom: 160,
                },
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut gfx.swash_cache,
        )
        .unwrap();

    let frame = gfx.surface.get_current_texture().unwrap_throw();
    let view = frame.texture.create_view(&Default::default());
    let mut encoder = gfx.device.create_command_encoder(&Default::default());

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        gfx.text_renderer
            .render(&gfx.atlas, &gfx.viewport, &mut rpass)
            .unwrap();
    }

    let command_buffer = encoder.finish();
    gfx.queue.submit([command_buffer]);
    frame.present();

    gfx.atlas.trim();
}

// fn resized(&mut self, size: PhysicalSize<u32>) {
//     let MaybeGraphics::Graphics(gfx) = &mut self.graphics else {
//         return;
//     };
//     gfx.surface_config.width = size.width;
//     gfx.surface_config.height = size.height;
//     gfx.surface.configure(&gfx.device, &gfx.surface_config);
// }

#[allow(dead_code)]
struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,

    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,
}

// impl ApplicationHandler<Graphics> for Application {
//     fn window_event(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//         _window_id: WindowId,
//         event: WindowEvent,
//     ) {
//         match event {
//             WindowEvent::Resized(size) => self.resized(size),
//             WindowEvent::RedrawRequested => self.draw(),
//             WindowEvent::CloseRequested => event_loop.exit(),
//             _ => (),
//         }
//     }

//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         if let MaybeGraphics::Builder(builder) = &mut self.graphics {
//             builder.build_and_send(event_loop);
//         }
//     }

//     fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
//         self.graphics = MaybeGraphics::Graphics(graphics);
//     }
// }
