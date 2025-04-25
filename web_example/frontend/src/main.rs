use fast2d::zoon::*;
use std::rc::Rc;
use std::cell::RefCell;

pub fn main() {
    start_app("app", root);
}

fn canvas_wrappers() -> [fast2d::CanvasWrapper; 2] { 
    [
        {
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                objects.push(
                    fast2d::Text::new()
                        .text("Hello, world!")
                        .font_size(30)
                        .line_height(42)
                        .color(255, 255, 255, 1.0)
                        .family("FiraCode")
                        .bounds(0, 0, 600, 160)
                        .into(),
                );
                objects.push(
                    fast2d::Rectangle::new()
                        .position(100, 100)
                        .size(100, 50)
                        .color(0, 255, 0, 1.0)
                        .rounded_corners(10, 5, 20, 25)
                        .into(),
                );
                // Add a triangle wave line
                objects.push(
                    fast2d::Line::new()
                        .points(&[
                            20.0, 250.0,  // Start
                            70.0, 200.0,  // Peak 1
                            120.0, 250.0, // Trough 1
                            170.0, 200.0, // Peak 2
                            220.0, 250.0, // Trough 2
                            270.0, 200.0, // Peak 3
                            320.0, 250.0, // End
                        ])
                        .color(255, 255, 0, 1.0) // Yellow
                        .width(4.0) // Use width() instead of thickness()
                        .into(),
                );
            });
            canvas_wrapper
        },
        {    
            let mut canvas_wrapper = fast2d::CanvasWrapper::new();
            canvas_wrapper.update_objects(|objects| {
                objects.push(
                    fast2d::Text::new()
                        .text("Hello from Fast2D!")
                        .font_size(20)
                        .line_height(20)
                        .color(0, 0, 255, 1.0)
                        .family("FiraCode")
                        .position(20, 50)
                        .bounds(0, 0, 600, 160)
                        .into(),
                );
                objects.push(
                    fast2d::Rectangle::new()
                        .position(100, 200) // Adjusted Y position for visibility if needed
                        .size(150, 100)
                        .color(255, 0, 0, 1.0)
                        .rounded_corners(40, 10, 40, 10)
                        .inner_border(5, 255, 255, 255, 1.0) // Renamed method back
                        .into(),
                );
                // Add the Circle after the text
                objects.push(
                    fast2d::Circle::new()
                        .center(70, 60) // Position overlapping the text
                        .radius(40)
                        .color(0, 0, 0, 0.0) // Transparent fill
                        .inner_border(3, 255, 105, 180, 1.0) // Pink border (HotPink)
                        .into(),
                );
            });
            canvas_wrapper
        }
    ]
}

fn root() -> impl Element {
    Column::new()
        .s(Height::fill()) // Ensure column fills height
        .s(Width::fill()) // Ensure column fills width
        .s(Background::new().color(color!("Black")))
        .items(canvas_wrappers().map(panel_with_canvas))
}

fn panel_with_canvas(canvas_wrapper: fast2d::CanvasWrapper) -> impl Element {
    let canvas_wrapper = Rc::new(RefCell::new(canvas_wrapper));
    El::new()
        .s(Align::center())
        .s(Clip::both())
        .s(Borders::all(Border::new().color(color!("Gray"))))
        // Give the panel a size constraint (e.g., fill available space up to a max)
        .s(Width::fill().max(700)) // Example max width
        .s(Height::fill().max(350)) // Example max height
        .on_viewport_size_change(clone!((canvas_wrapper) move |width, height| {
            canvas_wrapper.borrow_mut().resized(width, height);
        }))
        .child(
            Canvas::new()
                .width(0) // Add initial width to satisfy type system
                .height(0) // Add initial height to satisfy type system
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
