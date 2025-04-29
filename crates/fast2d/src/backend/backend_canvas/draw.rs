use web_sys::wasm_bindgen::UnwrapThrowExt;

pub fn draw(ctx: &web_sys::CanvasRenderingContext2d, objects: &[crate::Object2d]) {
    ctx.set_fill_style_str("black");
    ctx.set_stroke_style_str("black");
    ctx.set_line_width(1.0);
    for obj in objects {
        match obj {
            crate::Object2d::Rectangle(rect) => {
                let border_width = rect.border_width.unwrap_or(0.0);
                let has_border = border_width > 0.0 && rect.border_color.map_or(false, |c| c.a > 0.0);
                let fill_offset = if has_border { border_width } else { 0.0 };
                let fill_x = rect.position.x + fill_offset;
                let fill_y = rect.position.y + fill_offset;
                let fill_w = rect.size.width - 2.0 * fill_offset;
                let fill_h = rect.size.height - 2.0 * fill_offset;
                let fill_radii = crate::backend::RoundedCorners {
                    top_left: (rect.rounded_corners.top_left - fill_offset).max(0.0),
                    top_right: (rect.rounded_corners.top_right - fill_offset).max(0.0),
                    bottom_left: (rect.rounded_corners.bottom_left - fill_offset).max(0.0),
                    bottom_right: (rect.rounded_corners.bottom_right - fill_offset).max(0.0),
                };
                if rect.rounded_corners.top_left > 0.0 || rect.rounded_corners.top_right > 0.0 || rect.rounded_corners.bottom_left > 0.0 || rect.rounded_corners.bottom_right > 0.0 {
                    if rect.color.a > 0.0 && fill_w > 0.0 && fill_h > 0.0 {
                        let fill_color = rect.color.to_canvas_rgba();
                        ctx.set_fill_style_str(&fill_color);
                        draw_rounded_rect_path(ctx, fill_x, fill_y, fill_w, fill_h, &fill_radii);
                        ctx.fill();
                    }
                    if has_border && fill_w > 0.0 && fill_h > 0.0 {
                        let stroke_color = rect.border_color.unwrap().to_canvas_rgba();
                        ctx.set_stroke_style_str(&stroke_color);
                        ctx.set_line_width(border_width as f64);
                        let border_x = rect.position.x + border_width / 2.0;
                        let border_y = rect.position.y + border_width / 2.0;
                        let border_w = rect.size.width - border_width;
                        let border_h = rect.size.height - border_width;
                        let border_radii = crate::backend::RoundedCorners {
                            top_left: (rect.rounded_corners.top_left - border_width / 2.0).max(0.0),
                            top_right: (rect.rounded_corners.top_right - border_width / 2.0).max(0.0),
                            bottom_left: (rect.rounded_corners.bottom_left - border_width / 2.0).max(0.0),
                            bottom_right: (rect.rounded_corners.bottom_right - border_width / 2.0).max(0.0),
                        };
                        draw_rounded_rect_path(ctx, border_x, border_y, border_w, border_h, &border_radii);
                        ctx.stroke();
                    }
                } else {
                    if rect.color.a > 0.0 && fill_w > 0.0 && fill_h > 0.0 {
                        let fill_color = rect.color.to_canvas_rgba();
                        ctx.set_fill_style_str(&fill_color);
                        ctx.fill_rect(fill_x as f64, fill_y as f64, fill_w as f64, fill_h as f64);
                    }
                    if has_border && fill_w > 0.0 && fill_h > 0.0 {
                        let stroke_color = rect.border_color.unwrap().to_canvas_rgba();
                        ctx.set_stroke_style_str(&stroke_color);
                        ctx.set_line_width(border_width as f64);
                        ctx.stroke_rect(
                            (rect.position.x + border_width / 2.0) as f64,
                            (rect.position.y + border_width / 2.0) as f64,
                            (rect.size.width - border_width) as f64,
                            (rect.size.height - border_width) as f64,
                        );
                    }
                }
            }
            crate::Object2d::Circle(circle) => {
                let border_width = circle.border_width.unwrap_or(0.0);
                let has_border = border_width > 0.0 && circle.border_color.map_or(false, |c| c.a > 0.0);
                let fill_radius = if has_border { circle.radius - border_width } else { circle.radius };
                if circle.color.a > 0.0 && fill_radius > 0.0 {
                    ctx.begin_path();
                    ctx.arc(
                        circle.center.x as f64,
                        circle.center.y as f64,
                        fill_radius as f64,
                        0.0,
                        std::f64::consts::PI * 2.0,
                    ).unwrap_throw();
                    let fill_color = circle.color.to_canvas_rgba();
                    ctx.set_fill_style_str(&fill_color);
                    ctx.fill();
                }
                if has_border && fill_radius > 0.0 {
                    ctx.begin_path();
                    ctx.arc(
                        circle.center.x as f64,
                        circle.center.y as f64,
                        (fill_radius + border_width / 2.0) as f64,
                        0.0,
                        std::f64::consts::PI * 2.0,
                    ).unwrap_throw();
                    let stroke_color = circle.border_color.unwrap().to_canvas_rgba();
                    ctx.set_stroke_style_str(&stroke_color);
                    ctx.set_line_width(border_width as f64);
                    ctx.stroke();
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

fn draw_rounded_rect_path(
    ctx: &web_sys::CanvasRenderingContext2d,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radii: &crate::backend::RoundedCorners,
) {
    let tl = radii.top_left.min(w / 2.0).min(h / 2.0);
    let tr = radii.top_right.min(w / 2.0).min(h / 2.0);
    let br = radii.bottom_right.min(w / 2.0).min(h / 2.0);
    let bl = radii.bottom_left.min(w / 2.0).min(h / 2.0);
    ctx.begin_path();
    ctx.move_to((x + tl) as f64, y as f64);
    ctx.line_to((x + w - tr) as f64, y as f64);
    if tr > 0.0 {
        ctx.arc_to((x + w) as f64, y as f64, (x + w) as f64, (y + tr) as f64, tr as f64).unwrap_throw();
    }
    ctx.line_to((x + w) as f64, (y + h - br) as f64);
    if br > 0.0 {
        ctx.arc_to((x + w) as f64, (y + h) as f64, (x + w - br) as f64, (y + h) as f64, br as f64).unwrap_throw();
    }
    ctx.line_to((x + bl) as f64, (y + h) as f64);
    if bl > 0.0 {
        ctx.arc_to(x as f64, (y + h) as f64, x as f64, (y + h - bl) as f64, bl as f64).unwrap_throw();
    }
    ctx.line_to(x as f64, (y + tl) as f64);
    if tl > 0.0 {
        ctx.arc_to(x as f64, y as f64, (x + tl) as f64, y as f64, tl as f64).unwrap_throw();
    }
    ctx.close_path();
}

fn font_weight_to_css(weight: &crate::object2d::FontWeight) -> &'static str {
    use crate::object2d::FontWeight::*;
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
