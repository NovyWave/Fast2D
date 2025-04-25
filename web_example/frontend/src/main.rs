use fast2d::zoon::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::f32::consts::PI; // Import PI for sine wave

pub fn main() {
    start_app("app", root);
}

async fn canvas_wrappers() -> [fast2d::CanvasWrapper; 3] { // Changed size to 3
    [
        { // Canvas 0: Simple Rectangle
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                objects.push(
                    fast2d::Rectangle::new()
                        .position(50, 50)
                        .size(200, 150) // Increased size to overlap text
                        .color(50, 0, 100, 1.0) // Blue
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("Simple Rectangle")
                        .font_size(60) // Made font size much larger
                        .line_height(60) // Adjusted line height
                        .color(255, 255, 255, 0.2)
                        .position(10, 50) // Use new position
                        .size(350, 120) // Use new size instead of bounds
                        .into(),
                );
            }).await;
            canvas_wrapper
        },
        { // Canvas 1: Sine Wave
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                // Generate points for a sine wave
                let mut sine_points = Vec::new();
                let amplitude = 50.0;
                let frequency = 0.02;
                let y_offset = 150.0;
                let steps = 100; // Keep as integer for loop
                for i in 0..=steps {
                    let x = (i as f32 / steps as f32) * 350.0; // Map i to x-coordinate (0-350)
                    let y = y_offset + amplitude * (x * frequency * 2.0 * PI).sin();
                    sine_points.push(x);
                    sine_points.push(y);
                }

                objects.push(
                    fast2d::Line::new()
                        .points(&sine_points)
                        .color(0, 255, 255, 1.0) // Cyan color
                        .width(3.0)
                        .into(),
                );
                objects.push(
                    fast2d::Text::new()
                        .text("Sine Wave Example")
                        .font_size(20) // Integer
                        .color(255, 255, 255, 0.8)
                        .position(10, 10) // Use new position
                        .size(300, 50) // Use new size instead of bounds
                        .into(),
                );
            }).await;
            canvas_wrapper
        },
        { // Canvas 2: Simple Face
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                // Removed y_offset variable

                // Head
                objects.push(
                    fast2d::Circle::new()
                        .center(175, 205) // 175 + 30
                        .radius(100)
                        .color(0, 128, 0, 1.0) // Green color
                        .into(),
                );
                // Left Eye
                objects.push(
                    fast2d::Circle::new()
                        .center(135, 175) // 145 + 30
                        .radius(15)
                        .color(255, 255, 255, 1.0) // White
                        .inner_border(2, 0, 0, 0, 1.0) // Increased border width to 2
                        .into(),
                );
                objects.push( // Pupil
                    fast2d::Circle::new()
                        .center(135, 175) // 145 + 30
                        .radius(7)
                        .color(0, 0, 0, 1.0) // Black
                        .into(),
                );
                 // Right Eye
                 objects.push(
                    fast2d::Circle::new()
                        .center(215, 175) // 145 + 30
                        .radius(15)
                        .color(255, 255, 255, 1.0) // White
                        .inner_border(2, 0, 0, 0, 1.0) // Increased border width to 2
                        .into(),
                );
                objects.push( // Pupil
                    fast2d::Circle::new()
                        .center(215, 175) // 145 + 30
                        .radius(7)
                        .color(0, 0, 0, 1.0) // Black
                        .into(),
                );

                // Hat
                objects.push( // Brim
                    fast2d::Rectangle::new()
                        .position(115, 100) // 70 + 30
                        .size(120, 20)
                        .color(0, 0, 0, 0.0) // Transparent fill (Reverted)
                        .rounded_corners(3, 3, 3, 3)
                        .inner_border(3, 139, 0, 0, 1.0) // Thick dark red border
                        .into(),
                );
                objects.push( // Top part
                    fast2d::Rectangle::new()
                        .position(135, 60) // 30 + 30
                        .size(80, 45)
                        .color(0, 0, 0, 0.0) // Transparent fill
                        .rounded_corners(5, 5, 0, 0)
                        .inner_border(3, 255, 165, 0, 1.0) // Thick orange border
                        .into(),
                );

                // Mouth (Smile) - Replace Rectangle with Line
                objects.push(
                    fast2d::Line::new()
                        .points(&[
                            140.0, 245.0, // 215.0 + 30.0
                            155.0, 260.0, // 230.0 + 30.0
                            175.0, 265.0, // 235.0 + 30.0
                            195.0, 260.0, // 230.0 + 30.0
                            210.0, 245.0, // 215.0 + 30.0
                        ])
                        .color(0, 0, 0, 1.0) // Black smile line
                        .width(5.0)
                        .into(),
                );
                 objects.push(
                    fast2d::Text::new()
                        .text("Face Example")
                        .font_size(20)
                        .color(255, 255, 255, 0.8)
                        .position(10, 10) // Use new position
                        .size(300, 50) // Use new size instead of bounds
                        .into(),
                );
            }).await;
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
                .items_signal_vec(
                    canvas_wrappers()
                        .map(Vec::from)
                        .into_signal_option()
                        .map(Option::unwrap_or_default)
                        .to_signal_vec()
                        .map(panel_with_canvas)
                )
        )
}

fn panel_with_canvas(canvas_wrapper: fast2d::CanvasWrapper) -> impl Element {
    let canvas_wrapper = Rc::new(RefCell::new(canvas_wrapper));
    El::new()
        .s(Align::center())
        .s(Clip::both())
        .s(Borders::all(Border::new().color(color!("Gray"))))
        // Give the panel a size constraint (e.g., fill available space up to a max)
        .s(Width::fill().max(650)) // Example max width
        .s(Height::exact(350)) // Example max height
        .on_viewport_size_change(clone!((canvas_wrapper) move |width, height| {
            let canvas_wrapper = canvas_wrapper.clone();
            Task::start(async move {
                canvas_wrapper.borrow_mut().resized(width, height).await;
            });
        }))
        .child(
            Canvas::new()
                .width(0) // Integer
                .height(0) // Integer
                .s(Width::fill()) // Style will override layout width
                .s(Height::fill()) // Added fill height style
                .after_insert(move |canvas| {
                    // Spawn the async function as a task
                    Task::start(async move {
                        canvas_wrapper.borrow_mut().set_canvas(canvas).await;
                    });
                })
        )
}
