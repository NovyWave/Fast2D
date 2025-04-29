use zoon::{*, futures_util::future::try_join_all};
use std::f32::consts::PI;

/// Entry point: loads fonts and starts the app.
pub fn main() {
    Task::start(async {
        load_and_register_fonts().await;
        start_app("app", root);
    });
}

/// Loads and registers required fonts asynchronously.
async fn load_and_register_fonts() {
    let fonts = try_join_all([
        fast2d::fetch_file("/_api/public/fonts/FiraCode-Regular.ttf"),
        fast2d::fetch_file("/_api/public/fonts/Inter-Regular.ttf"),
        fast2d::fetch_file("/_api/public/fonts/Inter-Bold.ttf"),
        fast2d::fetch_file("/_api/public/fonts/Inter-BoldItalic.ttf"),
    ]).await.unwrap_throw();
    fast2d::register_fonts(&fonts).unwrap_throw();
}

/// Returns an array of example canvases.
fn examples() -> [fast2d::CanvasWrapper; 3] {
    [
        example_rectangle(),
        example_face(),
        example_sine_wave(),
    ]
}

/// Example 1: Simple Rectangle
fn example_rectangle() -> fast2d::CanvasWrapper {
    let mut canvas_wrapper = fast2d::CanvasWrapper::new();
    canvas_wrapper.update_objects(|objects| {
        objects.push(
            fast2d::Rectangle::new()
                .position(50., 50.)
                .inner_border(30., 255, 255, 0, 1.0)
                .size(200., 150.)
                .color(50, 0, 100, 1.0)
                .into(),
        );
        objects.push(
            fast2d::Text::new()
                .text("Simple Rectangle")
                .position(10., 50.)
                .size(350., 120.)
                .color(255, 255, 255, 0.2)
                .font_size(60.)
                .family(fast2d::Family::name("Fira Code"))
                .into(),
        );
    });
    canvas_wrapper
}

/// Example 2: Simple Face
fn example_face() -> fast2d::CanvasWrapper {
    let mut canvas_wrapper = fast2d::CanvasWrapper::new();
    canvas_wrapper.update_objects(|objects| {
        // Head
        objects.push(
            fast2d::Circle::new()
                .center(175., 205.)
                .inner_border(30., 255, 255, 0, 1.0)
                .radius(100.)
                .color(0, 128, 0, 1.0)
                .into(),
        );
        // Left Eye
        objects.push(
            fast2d::Circle::new()
                .center(135., 175.)
                .radius(15.)
                .color(255, 255, 255, 1.0)
                .inner_border(2., 0, 0, 0, 1.0)
                .into(),
        );
        objects.push( // Pupil
            fast2d::Circle::new()
                .center(135., 175.)
                .radius(7.)
                .color(0, 0, 0, 1.0)
                .into(),
        );
        // Right Eye
        objects.push(
            fast2d::Circle::new()
                .center(215., 175.)
                .radius(15.)
                .color(255, 255, 255, 1.0)
                .inner_border(2., 0, 0, 0, 1.0)
                .into(),
        );
        objects.push( // Pupil
            fast2d::Circle::new()
                .center(215., 175.)
                .radius(7.)
                .color(0, 0, 0, 1.0)
                .into(),
        );
        // Hat
        objects.push( // Brim
            fast2d::Rectangle::new()
                .position(115., 100.)
                .size(120., 20.)
                .color(0, 0, 0, 0.0)
                .rounded_corners(3., 3., 3., 3.)
                .inner_border(3., 139, 0, 0, 1.0)
                .into(),
        );
        objects.push( // Top part
            fast2d::Rectangle::new()
                .position(135., 60.)
                .size(80., 45.)
                .color(0, 0, 0, 0.0)
                .rounded_corners(5., 5., 0., 0.)
                .inner_border(3., 255, 165, 0, 1.0)
                .into(),
        );
        // Mouth (Smile)
        let mouth_points = [
            (140., 245.),
            (155., 260.),
            (175., 265.),
            (195., 260.),
            (210., 245.),
        ];
        objects.push(
            fast2d::Line::new()
                .points(&mouth_points)
                .color(0, 0, 0, 1.0)
                .width(5.)
                .into(),
        );
        // Texts
        objects.push(
            fast2d::Text::new()
                .text("Face Example")
                .position(10., 10.)
                .size(150., 50.)
                .color(255, 255, 255, 1.0)
                .font_size(20.)
                .family(fast2d::Family::name("Inter"))
                .into(),
        );
        objects.push(
            fast2d::Text::new()
                .text("With a ")
                .position(180., 10.)
                .size(70., 50.)
                .color(255, 255, 0, 1.0)
                .font_size(20.)
                .family(fast2d::Family::name("Inter"))
                .italic(false)
                .weight(fast2d::FontWeight::Bold)
                .into(),
        );
        objects.push(
            fast2d::Text::new()
                .text("hat")
                .position(250., 10.)
                .size(50., 50.)
                .color(139, 0, 0, 1.0)
                .font_size(20.)
                .family(fast2d::Family::name("Inter"))
                .italic(true)
                .weight(fast2d::FontWeight::Bold)
                .into(),
        );
    });
    canvas_wrapper
}

/// Example 3: Sine Wave
fn example_sine_wave() -> fast2d::CanvasWrapper {
    let mut canvas_wrapper = fast2d::CanvasWrapper::new();
    canvas_wrapper.update_objects(|objects| {
        // Generate points for a sine wave
        let mut sine_points_tuples = Vec::new();
        let amplitude = 50.;
        let frequency = 0.02;
        let y_offset = 150.;
        let steps = 100;
        for i in 0..=steps {
            let x = (i as f32 / steps as f32) * 350.;
            let y = y_offset + amplitude * (x * frequency * 2. * PI).sin();
            sine_points_tuples.push((x, y));
        }
        objects.push(
            fast2d::Line::new()
                .points(&sine_points_tuples)
                .color(0, 255, 255, 1.0)
                .width(3.)
                .into(),
        );
        objects.push(
            fast2d::Text::new()
                .text("Sine Wave Example")
                .position(10., 10.)
                .size(300., 50.)
                .color(255, 255, 255, 0.8)
                .font_size(20.)
                .family(fast2d::Family::name("Fira Code"))
                .into(),
        );
    });
    canvas_wrapper
}

/// Root UI layout: creates a scrollable column of example panels.
fn root() -> impl Element {
    El::new()
        .s(Height::fill()) // Ensure column fills height
        .s(Width::fill())
        .s(Scrollbars::both())
        .s(Background::new().color(color!("Black")))
        .child(
            Column::new()
                .s(Gap::both(10)) // Add gap between panels
                .s(Scrollbars::both())
                .s(Padding::all(10))
                .items(examples().map(panel_with_canvas))
        )
}

/// Wraps a canvas example in a styled panel.
fn panel_with_canvas(example: fast2d::CanvasWrapper) -> impl Element {
    El::new()
        .s(Align::center())
        .s(Clip::both())
        .s(Borders::all(Border::new().color(color!("Gray"))))
        .s(Width::fill().max(650)) // Example max width
        .s(Height::exact(350)) // Example max height
        .child_signal(canvas_with_example(example).into_signal_option())
}

/// Asynchronously attaches a fast2d example to a Zoon canvas element.
async fn canvas_with_example(mut example: fast2d::CanvasWrapper) -> impl Element {
    let mut zoon_canvas = Canvas::new()
        .width(0)
        .height(0)
        .s(Width::fill())
        .s(Height::fill());
    example.set_canvas(zoon_canvas.raw_el_mut().dom_element()).await; 
    zoon_canvas.update_raw_el(move |raw_el| {
        raw_el.on_resize(move |width, height| {
            example.resized(width, height);
        })
    })
}
