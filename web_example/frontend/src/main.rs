use zoon::{eprintln, *};
// Remove the direct import of Family
// use fast2d::Family;

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::f32::consts::PI;

// --- Load font data directly ---
// No need for separate consts if only one font is used initially
// const FIRA_CODE_REGULAR_DATA: &[u8] = include_bytes!("../fonts/FiraCode-Regular.ttf");

pub fn main() {
    // --- Prepare font data for initialization ---
    let font_data_to_register: Vec<&'static [u8]> = vec![
        // Pass the raw font data bytes, casting to a slice reference
        include_bytes!("../fonts/FiraCode-Regular.ttf"),
        // Add more font data slices here if needed, casting them similarly
    ];

    // --- Initialize Fast2D with the font system ---
    if let Err(e) = fast2d::init_font_system(font_data_to_register) {
        eprintln!("Failed to initialize Fast2D Font System: {:?}", e);
        // Add specific error handling if needed
        match e {
            fast2d::FontSystemInitError::AlreadyInitialized => eprintln!("Warning: FontSystem already initialized."),
            fast2d::FontSystemInitError::NoFontsProvided => panic!("No font data provided to init_font_system."),
            fast2d::FontSystemInitError::DatabaseError(err) => panic!("Error loading font data: {}", err), // Adjust if error type changes
        }
    }

    start_app("app", root);
}

fn canvas_wrappers() -> [fast2d::CanvasWrapper; 3] {
    [
        { // Canvas 0: Simple Rectangle
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
                        // .family(fast2d::Family::Name("Fira Code"))
                        .into(),
                );
            });
            canvas_wrapper
        },
        { // Canvas 1: Sine Wave
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
                        .font_size(20.0)      // Already f32
                        // .family(fast2d::Family::Name("Fira Code"))
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
                        .size(300.0, 50.0)   // Use f32
                        .color(255, 255, 255, 0.8)
                        .font_size(20.0)      // Already f32
                        // .family(fast2d::Family::Name("Fira Code"))
                        .into(),
                );
            });
            canvas_wrapper
        }
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
                .items(canvas_wrappers().map(panel_with_canvas))
        )
}

fn panel_with_canvas(canvas_wrapper: fast2d::CanvasWrapper) -> impl Element {
    let canvas_wrapper = Rc::new(RefCell::new(canvas_wrapper));
    let pending_resized = Rc::new(Cell::new(None));
    El::new()
        .s(Align::center())
        .s(Clip::both())
        .s(Borders::all(Border::new().color(color!("Gray"))))
        // Give the panel a size constraint (e.g., fill available space up to a max)
        .s(Width::fill().max(650)) // Example max width
        .s(Height::exact(350)) // Example max height
        .on_viewport_size_change(clone!((canvas_wrapper, pending_resized) move |width, height| {
            if let Ok(mut canvas_wrapper) = canvas_wrapper.try_borrow_mut() {
                canvas_wrapper.resized(width, height);
            } else {
                pending_resized.set(Some((width, height)));
            }
        }))
        .child(
            Canvas::new()
                .width(0) // Integer
                .height(0) // Integer
                .s(Width::fill()) // Style will override layout width
                .s(Height::fill()) // Added fill height style
                .after_insert(move |canvas| {
                    Task::start(async move {
                        canvas_wrapper.borrow_mut().set_canvas(canvas).await;
                        if let Some((width, height)) = pending_resized.take() {
                            canvas_wrapper.borrow_mut().resized(width, height);
                        }
                    });
                })
        )
}
