use fast2d::zoon::*;

// @TODO remove
// #[allow(dead_code)]
// mod hello_triangle;

// #[allow(dead_code)]
// mod hello_world;

// #[allow(dead_code)]
// mod rust_logo;

// @TODO remove this together with the same code in the `lib.rs` file
const CANVAS_WIDTH: u32 = 350;
const CANVAS_HEIGHT: u32 = 350;

pub fn main() {
    start_app("app", root);
}

fn root() -> impl Element {
    Column::new()
        .s(Height::fill())
        .s(Background::new().color(color!("Black")))
        .item(panel_with_canvas(|canvas| { 
            fast2d::run(canvas, vec![
                fast2d::Text::new()
                    .position(10, 10)
                    .text("Hello, world!")
                    .font_size(30)
                    .line_height(42)
                    .color(255, 255, 255, 1)
                    .family("FiraCode")
                    .bounds(0, 0, 600, 160)
                    .into(),
                fast2d::Rectangle::new()
                    .position(100, 100)
                    .size(100, 50)
                    .color(0, 255, 0, 1)
                    .rounded_corners(10, 5, 20, 25)
                    .into(),
            ])
        }))
        .item(panel_with_canvas(|canvas| { 
            fast2d::run(canvas, vec![
                fast2d::Text::new()
                    .text("Hello from Fast2D!")
                    .size(30)
                    .line_height(42)
                    .color(255, 255, 255)
                    .family("FiraCode")
                    .position(10, 10)
                    .bounds(0, 0, 600, 160)
                    .into(),
                fast2d::Rectangle::new()
                    .position(100, 100)
                    .size(100, 50)
                    .color(255, 0, 0, 1)
                    .rounded_corners(10, 5, 20, 25)
                    .into(),
            ])
        }))
}

fn panel_with_canvas(
    example_runner: impl FnOnce(web_sys::HtmlCanvasElement) + 'static,
) -> impl Element {
    El::new()
        .s(Align::center())
        .s(Clip::both())
        .s(Borders::all(Border::new().color(color!("Gray"))))
        .child(
            Canvas::new()
                .width(CANVAS_WIDTH)
                .height(CANVAS_HEIGHT)
                .after_insert(example_runner),
        )
}
