use zoon::{*, futures_util::future::try_join_all};
use std::f32::consts::PI;

pub fn main() {
    Task::start(async {
        load_and_register_fonts().await;
        start_app("app", root);
    });
}

async fn load_and_register_fonts() {
    let fonts = try_join_all([
        fast2d::fetch_file("/_api/public/fonts/FiraCode-Regular.ttf"),
        fast2d::fetch_file("/_api/public/fonts/Inter-Regular.ttf"),
        fast2d::fetch_file("/_api/public/fonts/Inter-Bold.ttf"),
        fast2d::fetch_file("/_api/public/fonts/Inter-BoldItalic.ttf"),
    ]).await.unwrap_throw();
    fast2d::register_fonts(&fonts).unwrap_throw();
}

fn examples() -> [fast2d::CanvasWrapper; 3] {
    [
        { // Canvas 1: Simple Rectangle
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                objects.push(
                    fast2d::Rectangle::new()
                        .position(50.0, 50.0) // Use f32
                        .size(200.0, 150.0)   // Use f32
                        .color(50, 0, 100, 1.0)
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("Simple Rectangle")
                        .position(10.0, 50.0) // Use f32
                        .size(350.0, 120.0)   // Use f32
                        .color(255, 255, 255, 0.2)
                        .font_size(60.0)
                        .family(fast2d::Family::name("Fira Code"))
                        .into(),
                );
            });
            canvas_wrapper
        },
        { // Canvas 2: Simple Face
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                // Head
                objects.push(
                    fast2d::Circle::new()
                        .center(175.0, 205.0) // Use f32
                        .radius(100.0)       // Use f32
                        .color(0, 128, 0, 1.0)
                        .into(),
                );
                // Left Eye
                objects.push(
                    fast2d::Circle::new()
                        .center(135.0, 175.0) // Use f32
                        .radius(15.0)       // Use f32
                        .color(255, 255, 255, 1.0)
                        .border(2.0, 0, 0, 0, 1.0) // Use f32 for width, renamed method
                        .into(),
                );
                objects.push( // Pupil
                    fast2d::Circle::new()
                        .center(135.0, 175.0) // Use f32
                        .radius(7.0)        // Use f32
                        .color(0, 0, 0, 1.0)
                        .into(),
                );
                 // Right Eye
                 objects.push(
                    fast2d::Circle::new()
                        .center(215.0, 175.0) // Use f32
                        .radius(15.0)       // Use f32
                        .color(255, 255, 255, 1.0)
                        .border(2.0, 0, 0, 0, 1.0) // Use f32 for width, renamed method
                        .into(),
                );
                objects.push( // Pupil
                    fast2d::Circle::new()
                        .center(215.0, 175.0) // Use f32
                        .radius(7.0)        // Use f32
                        .color(0, 0, 0, 1.0)
                        .into(),
                );
                // Hat
                objects.push( // Brim
                    fast2d::Rectangle::new()
                        .position(115.0, 100.0) // Use f32
                        .size(120.0, 20.0)    // Use f32
                        .color(0, 0, 0, 0.0)
                        .rounded_corners(3.0, 3.0, 3.0, 3.0) // Use f32
                        .border(3.0, 139, 0, 0, 1.0) // Use f32 for width, renamed method
                        .into(),
                );
                objects.push( // Top part
                    fast2d::Rectangle::new()
                        .position(135.0, 60.0) // Use f32
                        .size(80.0, 45.0)     // Use f32
                        .color(0, 0, 0, 0.0)
                        .rounded_corners(5.0, 5.0, 0.0, 0.0) // Use f32
                        .border(3.0, 255, 165, 0, 1.0) // Use f32 for width, renamed method
                        .into(),
                );
                // Mouth (Smile) - points are already f32
                let mouth_points = [
                    (140.0, 245.0),
                    (155.0, 260.0),
                    (175.0, 265.0),
                    (195.0, 260.0),
                    (210.0, 245.0),
                ];
                objects.push(
                    fast2d::Line::new()
                        .points(&mouth_points) // Pass slice of tuples
                        .color(0, 0, 0, 1.0)
                        .width(5.0) // Already f32
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("Face Example")
                        .position(10.0, 10.0) // Use f32
                        .size(150.0, 50.0)   // Use f32
                        .color(255, 255, 255, 1.0)
                        .font_size(20.0)      // Already f32
                        .family(fast2d::Family::name("Inter"))
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("With a ")
                        .position(180.0, 10.0) // Use f32
                        .size(70.0, 50.0)   // Use f32
                        .color(255, 255, 0, 1.0)
                        .font_size(20.0)      // Already f32
                        .family(fast2d::Family::name("Inter"))
                        .italic(false)
                        .weight(fast2d::FontWeight::Bold)
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("hat")
                        .position(250.0, 10.0) // Use f32
                        .size(50.0, 50.0)   // Use f32
                        .color(139, 0, 0, 1.0)
                        .font_size(20.0)      // Already f32
                        .family(fast2d::Family::name("Inter"))
                        .italic(true)
                        .weight(fast2d::FontWeight::Bold)
                        .into(),
                );
            });
            canvas_wrapper
        },
        { // Canvas 3: Sine Wave
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                // Generate points for a sine wave - already f32
                let mut sine_points_tuples = Vec::new(); // Use tuples for builder
                let amplitude = 50.0;
                let frequency = 0.02;
                let y_offset = 150.0;
                let steps = 100;
                for i in 0..=steps {
                    let x = (i as f32 / steps as f32) * 350.0;
                    let y = y_offset + amplitude * (x * frequency * 2.0 * PI).sin();
                    sine_points_tuples.push((x, y)); // Push tuple
                }
                objects.push(
                    fast2d::Line::new()
                        .points(&sine_points_tuples) // Pass slice of tuples
                        .color(0, 255, 255, 1.0)
                        .width(3.0) // Already f32
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("Sine Wave Example")
                        .position(10.0, 10.0) // Use f32
                        .size(300.0, 50.0)   // Use f32
                        .color(255, 255, 255, 0.8)
                        .font_size(20.0)  
                        .family(fast2d::Family::name("Fira Code"))    // Already f32
                        .into(),
                );
            });
            canvas_wrapper
        },
    ]
}

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

fn panel_with_canvas(example: fast2d::CanvasWrapper) -> impl Element {
    El::new()
        .s(Align::center())
        .s(Clip::both())
        .s(Borders::all(Border::new().color(color!("Gray"))))
        .s(Width::fill().max(650)) // Example max width
        .s(Height::exact(350)) // Example max height
        .child_signal(canvas_with_example(example).into_signal_option())
}

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
