mod register_fonts;
pub use register_fonts::register_fonts;

mod canvas_wrapper;
pub use canvas_wrapper::CanvasWrapper;

use web_sys::wasm_bindgen::UnwrapThrowExt;

pub(crate) fn draw_canvas(ctx: &web_sys::CanvasRenderingContext2d, objects: &[crate::Object2d]) {
    // Set default state (optional, but good practice)
    ctx.set_fill_style_str("black"); // Default fill
    ctx.set_stroke_style_str("black"); // Default stroke
    ctx.set_line_width(1.0);

    for obj in objects {
        match obj {
            crate::Object2d::Rectangle(rect) => {
                // Set fill color
                let fill_color = rect.color.to_canvas_rgba();
                ctx.set_fill_style_str(&fill_color);
                // TODO: Add rounded rectangle support if radii > 0 using path API
                ctx.fill_rect(
                    rect.position.x as f64,
                    rect.position.y as f64,
                    rect.size.width as f64,
                    rect.size.height as f64,
                );
                // Handle border
                if let (Some(border_width), Some(border_color_val)) = (rect.border_width, rect.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let stroke_color = border_color_val.to_canvas_rgba();
                        ctx.set_stroke_style_str(&stroke_color);
                        ctx.set_line_width(border_width as f64);
                        ctx.stroke_rect(
                            rect.position.x as f64,
                            rect.position.y as f64,
                            rect.size.width as f64,
                            rect.size.height as f64,
                        );
                    }
                }
            }
            crate::Object2d::Circle(circle) => {
                ctx.begin_path();
                ctx.arc(
                    circle.center.x as f64,
                    circle.center.y as f64,
                    circle.radius as f64,
                    0.0,
                    std::f64::consts::PI * 2.0,
                ).unwrap_throw();
                // Fill
                if circle.color.a > 0.0 {
                    let fill_color = circle.color.to_canvas_rgba();
                    ctx.set_fill_style_str(&fill_color);
                    ctx.fill();
                }
                // Border
                if let (Some(border_width), Some(border_color_val)) = (circle.border_width, circle.border_color) {
                    if border_width > 0.0 && border_color_val.a > 0.0 {
                        let stroke_color = border_color_val.to_canvas_rgba();
                        ctx.set_stroke_style_str(&stroke_color);
                        ctx.set_line_width(border_width as f64);
                        ctx.stroke();
                    }
                }
            }
            crate::Object2d::Line(line) => {
                if line.points.len() >= 2 && line.color.a > 0.0 {
                    let stroke_color = line.color.to_canvas_rgba();
                    ctx.set_stroke_style_str(&stroke_color);
                    ctx.set_line_width(line.width as f64);
                    ctx.begin_path();
                    ctx.move_to(line.points[0].x as f64, line.points[0].y as f64);
                    for i in 1..line.points.len() {
                        ctx.line_to(line.points[i].x as f64, line.points[i].y as f64);
                    }
                    ctx.stroke();
                }
            }
            crate::Object2d::Text(text) => {
                if text.color.a > 0.0 {
                    let fill_color = text.color.to_canvas_rgba();
                    ctx.set_fill_style_str(&fill_color);
                    let font_style = if text.italic { "italic" } else { "normal" };
                    let font_weight = font_weight_to_css(&text.weight);
                    let font_str = format!("{} {} {}px {}", font_style, font_weight, text.font_size, text.family);
                    ctx.set_font(&font_str);
                    let max_width = text.width;
                    let line_height = text.font_size * text.line_height_multiplier;
                    let words: Vec<&str> = text.text.split_whitespace().collect();
                    let mut lines: Vec<String> = Vec::new();
                    let mut current_line = String::new();
                    for word in words {
                        let test_line = if current_line.is_empty() {
                            word.to_string()
                        } else {
                            format!("{} {}", current_line, word)
                        };
                        let metrics = ctx.measure_text(&test_line).unwrap_throw();
                        if metrics.width() <= max_width as f64 || current_line.is_empty() {
                            current_line = test_line;
                        } else {
                            lines.push(current_line);
                            current_line = word.to_string();
                        }
                    }
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                    let mut y = text.top;
                    for line in lines {
                        let metrics = ctx.measure_text(&line).unwrap_throw();
                        let ascent = metrics.actual_bounding_box_ascent();
                        let font_box_ascent = metrics.font_bounding_box_ascent();
                        let gap = font_box_ascent - ascent;
                        let line_gap = if gap > 0.0 && gap < 1.0 { gap } else { 0.0 };
                        ctx.fill_text(&line, text.left as f64, y as f64 + ascent + line_gap).unwrap_throw();
                        y += line_height;
                        if y > text.top + text.height {
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn font_weight_to_css(weight: &crate::object_2d::text::FontWeight) -> &'static str {
    use crate::object_2d::text::FontWeight::*;
    match weight {
        Thin => "100",
        ExtraLight => "200",
        Light => "300",
        Regular => "400",
        Medium => "500",
        SemiBold => "600",
        Bold => "700",
        ExtraBold => "800",
        Black => "900",
    }
}
