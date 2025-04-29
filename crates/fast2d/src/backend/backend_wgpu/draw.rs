use super::*;
use crate::{Rectangle, Circle};

// The main draw function for rendering all 2D objects using wgpu
pub fn draw(gfx: &mut Graphics, objects: &[crate::Object2d]) {
    // Try to get the current frame's texture from the GPU surface
    let output = match gfx.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(e) => {
            // If we can't get the texture, log the error and try to recover if possible
            console::error_1(&JsValue::from_str(&format!("Error getting current texture: {:?}", e)));
            if e == wgpu::SurfaceError::Lost {
                // If the surface is lost, reconfigure it
                gfx.surface.configure(&gfx.device, &gfx.surface_config);
                return;
            }
            // For other errors, just return
            return;
        }
    };

    // Create views for the main output texture and the MSAA (anti-aliasing) texture
    let view = output.texture.create_view(&TextureViewDescriptor::default());
    let msaa_view = gfx.msaa_texture.create_view(&TextureViewDescriptor::default());

    // Lock the font system for text rendering
    let mut font_system = FONT_SYSTEM.get()
        .expect_throw("FontSystem not initialized")
        .lock()
        .expect_throw("Failed to lock FontSystem Mutex");

    // Prepare glyph buffers for all text objects
    let mut glyph_buffers: Vec<GlyphonBuffer> = Vec::new();

    // Loop through all objects and collect text buffers
    for obj in objects {
        if let crate::Object2d::Text(text) = obj {
            // Set up text metrics and buffer
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let line_height_pixels = text.font_size * text.line_height_multiplier;
            let mut buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(text.font_size, line_height_pixels));
            buffer.set_size(&mut font_system, Some(text_width_f32), Some(text_height_f32));

            // Convert font family to glyphon format
            let glyphon_family = match &text.family {
                crate::object2d::Family::Name(name) => GlyphonFamily::Name(name.as_ref()),
                crate::object2d::Family::SansSerif => GlyphonFamily::SansSerif,
                crate::object2d::Family::Serif => GlyphonFamily::Serif,
                crate::object2d::Family::Monospace => GlyphonFamily::Monospace,
                crate::object2d::Family::Cursive => GlyphonFamily::Cursive,
                crate::object2d::Family::Fantasy => GlyphonFamily::Fantasy,
            };

            let family_for_query = match &glyphon_family {
                GlyphonFamily::Name(name) => glyphon::fontdb::Family::Name(name),
                GlyphonFamily::SansSerif => glyphon::fontdb::Family::SansSerif,
                GlyphonFamily::Serif => glyphon::fontdb::Family::Serif,
                GlyphonFamily::Monospace => glyphon::fontdb::Family::Monospace,
                GlyphonFamily::Cursive => glyphon::fontdb::Family::Cursive,
                GlyphonFamily::Fantasy => glyphon::fontdb::Family::Fantasy,
            };

            // Check if the font exists, warn if not
            let font_query = glyphon::fontdb::Query {
                families: &[family_for_query],
                ..Default::default()
            };
            let font_exists = font_system.db().query(&font_query).is_some();
            if !font_exists {
                let warning_message = format!("Warning: Font family '{:?}' not found. Falling back to default.", text.family);
                web_sys::console::warn_1(&JsValue::from_str(&warning_message));
            }

            // Set up text attributes (color, weight, style)
            let glyphon_color = text.color.to_glyphon_color();
            let attrs = Attrs::new()
                .family(glyphon_family)
                .color(glyphon_color)
                .weight({
                    use crate::object2d::FontWeight::*;
                    match text.weight {
                        Thin => glyphon::fontdb::Weight::THIN,
                        ExtraLight => glyphon::fontdb::Weight::EXTRA_LIGHT,
                        Light => glyphon::fontdb::Weight::LIGHT,
                        Regular => glyphon::fontdb::Weight::NORMAL,
                        Medium => glyphon::fontdb::Weight::MEDIUM,
                        SemiBold => glyphon::fontdb::Weight::SEMIBOLD,
                        Bold => glyphon::fontdb::Weight::BOLD,
                        ExtraBold => glyphon::fontdb::Weight::EXTRA_BOLD,
                        Black => glyphon::fontdb::Weight::BLACK,
                    }
                })
                .style(if text.italic { glyphon::fontdb::Style::Italic } else { glyphon::fontdb::Style::Normal });
            buffer.set_text(&mut font_system, &text.text, &attrs, Shaping::Advanced);
            glyph_buffers.push(buffer);
        }
    }

    // Prepare text areas for rendering (position, bounds, etc.)
    let mut text_areas: Vec<TextArea> = Vec::new();
    let mut buffer_idx = 0;
    for obj in objects {
        if let crate::Object2d::Text(text) = obj {
            let glyphon_color = text.color.to_glyphon_color();
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let text_area = TextArea {
                buffer: &glyph_buffers[buffer_idx],
                left: text.left,
                top: text.top,
                bounds: TextBounds {
                    left: text.left as i32,
                    top: text.top as i32,
                    right: (text.left + text_width_f32) as i32,
                    bottom: (text.top + text_height_f32) as i32,
                },
                default_color: glyphon_color,
                scale: 1.0,
                custom_glyphs: &[],
            };
            text_areas.push(text_area);
            buffer_idx += 1;
        }
    }

    // Prepare the text renderer with all text areas
    match gfx.text_renderer.prepare(
        &gfx.device, &gfx.queue, &mut font_system, &mut gfx.atlas, &gfx.viewport,
        text_areas.into_iter(), &mut gfx.swash_cache,
    ) {
        Ok(_) => {}
        Err(e) => console::error_1(&JsValue::from_str(&format!("Error preparing text renderer: {:?}", e))),
    }

    // Create vertex and tessellator buffers for shape rendering
    let mut buffers: VertexBuffers<ColoredVertex, u32> = VertexBuffers::new();
    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    // Helper function to draw rectangles, including fill and optional border
    // This function takes a Rectangle object and draws it to the screen.
    // It handles both the filled area and the border (if any).
    fn draw_rectangle(
        rect: &Rectangle,
        buffers: &mut VertexBuffers<ColoredVertex, u32>,
        fill_tessellator: &mut FillTessellator,
        stroke_tessellator: &mut StrokeTessellator,
    ) {
        // Convert the rectangle's color to a linear color format for rendering
        let linear_color = rect.color.to_linear();
        // Get the border width, or 0 if not set
        let border_width = rect.border_width.unwrap_or(0.0);
        // Check if the rectangle has a visible border
        let has_border = border_width > 0.0 && rect.border_color.map_or(false, |c| c.a > 0.0);
        // If there is a border, shrink the fill area so the border fits inside the rectangle
        let fill_offset = if has_border { border_width } else { 0.0 };
        // Calculate the area to fill (the inside of the rectangle)
        let fill_box = Box2D::new(
            point(rect.position.x + fill_offset, rect.position.y + fill_offset),
            point(rect.position.x + rect.size.width - fill_offset, rect.position.y + rect.size.height - fill_offset),
        );
        let mut builder = Path::builder();
        // If any corner is rounded, add a rounded rectangle path
        if rect.rounded_corners.top_left > 0.0 || rect.rounded_corners.top_right > 0.0 || rect.rounded_corners.bottom_left > 0.0 || rect.rounded_corners.bottom_right > 0.0 {
            builder.add_rounded_rectangle(&fill_box, &LyonBorderRadii {
                top_left: (rect.rounded_corners.top_left.max(0.0) - fill_offset).max(0.0),
                top_right: (rect.rounded_corners.top_right.max(0.0) - fill_offset).max(0.0),
                bottom_left: (rect.rounded_corners.bottom_left.max(0.0) - fill_offset).max(0.0),
                bottom_right: (rect.rounded_corners.bottom_right.max(0.0) - fill_offset).max(0.0),
            }, Winding::Positive);
        } else {
            // Otherwise, add a simple rectangle path
            builder.add_rectangle(&fill_box, Winding::Positive);
        }
        let fill_path = builder.build();
        // Draw the filled part of the rectangle if it is visible
        if rect.color.a > 0.0 && fill_box.size().width > 0.0 && fill_box.size().height > 0.0 {
            fill_tessellator.tessellate_path(
                &fill_path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(buffers, |vertex: FillVertex| ColoredVertex {
                    position: [vertex.position().x, vertex.position().y],
                    color: linear_color,
                }),
            ).unwrap_throw();
        }
        // Draw the border if needed
        if has_border && fill_box.size().width > 0.0 && fill_box.size().height > 0.0 {
            // Convert the border color to linear format
            let linear_border_color = rect.border_color.unwrap_throw().to_linear();
            // Calculate the area for the border (centered on the rectangle's edge)
            let border_box = Box2D::new(
                point(rect.position.x + border_width / 2.0, rect.position.y + border_width / 2.0),
                point(rect.position.x + rect.size.width - border_width / 2.0, rect.position.y + rect.size.height - border_width / 2.0),
            );
            let mut border_builder = Path::builder();
            // Add a rounded rectangle path for the border if needed
            if rect.rounded_corners.top_left > 0.0 || rect.rounded_corners.top_right > 0.0 || rect.rounded_corners.bottom_left > 0.0 || rect.rounded_corners.bottom_right > 0.0 {
                border_builder.add_rounded_rectangle(&border_box, &LyonBorderRadii {
                    top_left: (rect.rounded_corners.top_left.max(0.0) - border_width / 2.0).max(0.0),
                    top_right: (rect.rounded_corners.top_right.max(0.0) - border_width / 2.0).max(0.0),
                    bottom_left: (rect.rounded_corners.bottom_left.max(0.0) - border_width / 2.0).max(0.0),
                    bottom_right: (rect.rounded_corners.bottom_right.max(0.0) - border_width / 2.0).max(0.0),
                }, Winding::Positive);
            } else {
                // Otherwise, add a simple rectangle path for the border
                border_builder.add_rectangle(&border_box, Winding::Positive);
            }
            let border_path = border_builder.build();
            // Set border options (width, etc.)
            let options = StrokeOptions::default().with_line_width(border_width);
            // Draw the border
            stroke_tessellator.tessellate_path(
                &border_path,
                &options,
                &mut BuffersBuilder::new(buffers, |vertex: StrokeVertex| ColoredVertex {
                    position: [vertex.position().x, vertex.position().y],
                    color: linear_border_color,
                }),
            ).unwrap_throw();
        }
    }

    // Helper function to draw circles, including fill and optional border
    // This function takes a Circle object and draws it to the screen.
    // It handles both the filled area and the border (if any).
    fn draw_circle(
        circle: &Circle,
        buffers: &mut VertexBuffers<ColoredVertex, u32>,
        fill_tessellator: &mut FillTessellator,
        stroke_tessellator: &mut StrokeTessellator,
    ) {
        // Convert the circle's color to a linear color format for rendering
        let linear_color = circle.color.to_linear();
        // Get the border width, or 0 if not set
        let border_width = circle.border_width.unwrap_or(0.0);
        // Check if the circle has a visible border
        let has_border = border_width > 0.0 && circle.border_color.map_or(false, |c| c.a > 0.0);
        // If there is a border, shrink the fill radius so the border fits inside the circle
        let fill_radius = if has_border { circle.radius - border_width } else { circle.radius };
        let mut builder = Path::builder();
        // Add a circle path for the filled area
        builder.add_circle(point(circle.center.x, circle.center.y), fill_radius, Winding::Positive);
        let fill_path = builder.build();
        // Draw the filled part of the circle if it is visible
        if circle.color.a > 0.0 && fill_radius > 0.0 {
            fill_tessellator.tessellate_path(
                &fill_path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(buffers, |vertex: FillVertex| ColoredVertex {
                    position: [vertex.position().x, vertex.position().y],
                    color: linear_color,
                }),
            ).unwrap_throw();
        }
        // Draw the border if needed
        if has_border && fill_radius > 0.0 {
            // Convert the border color to linear format
            let linear_border_color = circle.border_color.unwrap_throw().to_linear();
            let mut border_builder = Path::builder();
            // Add a circle path for the border (centered on the edge)
            border_builder.add_circle(
                point(circle.center.x, circle.center.y),
                fill_radius + border_width / 2.0,
                Winding::Positive,
            );
            let border_path = border_builder.build();
            // Set border options (width, etc.)
            let options = StrokeOptions::default().with_line_width(border_width);
            // Draw the border
            stroke_tessellator.tessellate_path(
                &border_path,
                &options,
                &mut BuffersBuilder::new(buffers, |vertex: StrokeVertex| ColoredVertex {
                    position: [vertex.position().x, vertex.position().y],
                    color: linear_border_color,
                }),
            ).unwrap_throw();
        }
    }

    // Loop through all objects and draw them
    for obj in objects {
        match obj {
            crate::Object2d::Rectangle(rect) => {
                // Draw a rectangle object
                draw_rectangle(rect, &mut buffers, &mut fill_tessellator, &mut stroke_tessellator);
            }
            crate::Object2d::Circle(circle) => {
                // Draw a circle object
                draw_circle(circle, &mut buffers, &mut fill_tessellator, &mut stroke_tessellator);
            }
            crate::Object2d::Line(line) => {
                // Draw a line object
                // Convert the line's color to a linear color format for rendering
                let linear_color = line.color.to_linear();
                let mut builder = Path::builder();
                // Only draw if there are at least two points (a line needs two points)
                if line.points.len() >= 2 {
                    // Start the line at the first point
                    builder.begin(point(line.points[0].x, line.points[0].y));
                    // Add each subsequent point to the path
                    for i in 1..line.points.len() {
                        builder.line_to(point(line.points[i].x, line.points[i].y));
                    }
                    // End the path (false = not closed)
                    builder.end(false);
                }
                let path = builder.build();
                // Draw the line if it is visible
                if line.points.len() >= 2 && line.color.a > 0.0 {
                    // Set line options: width, rounded ends and joins
                    let options = StrokeOptions::default()
                        .with_line_width(line.width)
                        .with_line_cap(LineCap::Round)
                        .with_line_join(LineJoin::Round);
                    // Draw the line
                    stroke_tessellator.tessellate_path(
                        &path,
                        &options,
                        &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex {
                            position: [vertex.position().x, vertex.position().y],
                            color: linear_color,
                        }),
                    ).unwrap_throw();
                }
            }
            crate::Object2d::Text(_) => {}
        }
    }

    // Create GPU buffers for vertices and indices
    let vertex_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"), contents: bytemuck::cast_slice(&buffers.vertices), usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"), contents: bytemuck::cast_slice(&buffers.indices), usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = buffers.indices.len() as u32;

    // Create a command encoder for the GPU commands
    let mut encoder = gfx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
    {
        // Begin a render pass (drawing session)
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(&view),
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });

        // Draw all shapes if there are any indices
        if num_indices > 0 {
            render_pass.set_pipeline(&gfx.rect_pipeline);
            render_pass.set_bind_group(0, &gfx.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        // Draw all text
        match gfx.text_renderer.render(&gfx.atlas, &gfx.viewport, &mut render_pass) {
            Ok(_) => {}
            Err(e) => console::error_1(&JsValue::from_str(&format!("Error rendering text: {:?}", e))),
        }
    }
    // Submit all drawing commands to the GPU
    gfx.queue.submit(std::iter::once(encoder.finish()));
    // Present the final image to the screen
    output.present();
}
