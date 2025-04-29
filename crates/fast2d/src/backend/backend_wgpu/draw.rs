use super::*;

pub fn draw(gfx: &mut Graphics, objects: &[crate::Object2d]) {
    let output = match gfx.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(e) => {
            console::error_1(&JsValue::from_str(&format!("Error getting current texture: {:?}", e)));
            if e == wgpu::SurfaceError::Lost {
                gfx.surface.configure(&gfx.device, &gfx.surface_config);
                return;
            }
            return;
        }
    };

    let view = output.texture.create_view(&TextureViewDescriptor::default());
    let msaa_view = gfx.msaa_texture.create_view(&TextureViewDescriptor::default());

    let mut font_system = FONT_SYSTEM.get()
        .expect("FontSystem not initialized")
        .lock()
        .expect("Failed to lock FontSystem Mutex");

    let mut glyph_buffers: Vec<GlyphonBuffer> = Vec::new();

    for obj in objects {
        if let crate::Object2d::Text(text) = obj {
            let text_width_f32 = text.width;
            let text_height_f32 = text.height;
            let line_height_pixels = text.font_size * text.line_height_multiplier;
            let mut buffer = GlyphonBuffer::new(&mut font_system, Metrics::new(text.font_size, line_height_pixels));
            buffer.set_size(&mut font_system, Some(text_width_f32), Some(text_height_f32));

            let family_owned: FamilyOwned = (&text.family).into();
            let glyphon_family = match &family_owned {
                FamilyOwned::Name(name) => GlyphonFamily::Name(name.as_str()),
                FamilyOwned::SansSerif => GlyphonFamily::SansSerif,
                FamilyOwned::Serif => GlyphonFamily::Serif,
                FamilyOwned::Monospace => GlyphonFamily::Monospace,
                FamilyOwned::Cursive => GlyphonFamily::Cursive,
                FamilyOwned::Fantasy => GlyphonFamily::Fantasy,
            };

            let family_for_query = match &glyphon_family {
                GlyphonFamily::Name(name) => glyphon::fontdb::Family::Name(name),
                GlyphonFamily::SansSerif => glyphon::fontdb::Family::SansSerif,
                GlyphonFamily::Serif => glyphon::fontdb::Family::Serif,
                GlyphonFamily::Monospace => glyphon::fontdb::Family::Monospace,
                GlyphonFamily::Cursive => glyphon::fontdb::Family::Cursive,
                GlyphonFamily::Fantasy => glyphon::fontdb::Family::Fantasy,
            };

            let font_query = glyphon::fontdb::Query {
                families: &[family_for_query],
                ..Default::default()
            };

            let font_exists = font_system.db().query(&font_query).is_some();
            if !font_exists {
                let warning_message = format!("Warning: Font family '{:?}' not found. Falling back to default.", text.family);
                web_sys::console::warn_1(&JsValue::from_str(&warning_message));
            }

            let glyphon_color = text.color.to_glyphon_color();
            let attrs = Attrs::new()
                .family(glyphon_family)
                .color(glyphon_color)
                .weight(font_weight_to_glyphon(text.weight))
                .style(if text.italic { glyphon::fontdb::Style::Italic } else { glyphon::fontdb::Style::Normal });
            buffer.set_text(&mut font_system, &text.text, &attrs, Shaping::Advanced);
            glyph_buffers.push(buffer);
        }
    }

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

    match gfx.text_renderer.prepare(
        &gfx.device, &gfx.queue, &mut font_system, &mut gfx.atlas, &gfx.viewport,
        text_areas.into_iter(), &mut gfx.swash_cache,
    ) {
        Ok(_) => {}
        Err(e) => console::error_1(&JsValue::from_str(&format!("Error preparing text renderer: {:?}", e))),
    }

    let mut buffers: VertexBuffers<ColoredVertex, u32> = VertexBuffers::new();
    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    for obj in objects {
        match obj {
            crate::Object2d::Rectangle(rect) => {
                let linear_color = rect.color.to_linear();
                let border_width = rect.border_width.unwrap_or(0.0);
                let has_border = border_width > 0.0 && rect.border_color.map_or(false, |c| c.a > 0.0);
                // Draw fill (shrink if border is present)
                let fill_offset = if has_border { border_width } else { 0.0 };
                let fill_box = Box2D::new(
                    point(rect.position.x + fill_offset, rect.position.y + fill_offset),
                    point(rect.position.x + rect.size.width - fill_offset, rect.position.y + rect.size.height - fill_offset),
                );
                let mut builder = Path::builder();
                if rect.rounded_corners.top_left > 0.0 || rect.rounded_corners.top_right > 0.0 || rect.rounded_corners.bottom_left > 0.0 || rect.rounded_corners.bottom_right > 0.0 {
                    builder.add_rounded_rectangle(&fill_box, &LyonBorderRadii {
                        top_left: (rect.rounded_corners.top_left.max(0.0) - fill_offset).max(0.0),
                        top_right: (rect.rounded_corners.top_right.max(0.0) - fill_offset).max(0.0),
                        bottom_left: (rect.rounded_corners.bottom_left.max(0.0) - fill_offset).max(0.0),
                        bottom_right: (rect.rounded_corners.bottom_right.max(0.0) - fill_offset).max(0.0),
                    }, Winding::Positive);
                } else {
                    builder.add_rectangle(&fill_box, Winding::Positive);
                }
                let fill_path = builder.build();
                if rect.color.a > 0.0 && fill_box.size().width > 0.0 && fill_box.size().height > 0.0 {
                    fill_tessellator.tessellate_path(&fill_path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                }
                // Draw inner border
                if has_border && fill_box.size().width > 0.0 && fill_box.size().height > 0.0 {
                    let linear_border_color = rect.border_color.unwrap().to_linear();
                    let border_box = Box2D::new(
                        point(rect.position.x + border_width / 2.0, rect.position.y + border_width / 2.0),
                        point(rect.position.x + rect.size.width - border_width / 2.0, rect.position.y + rect.size.height - border_width / 2.0),
                    );
                    let mut border_builder = Path::builder();
                    if rect.rounded_corners.top_left > 0.0 || rect.rounded_corners.top_right > 0.0 || rect.rounded_corners.bottom_left > 0.0 || rect.rounded_corners.bottom_right > 0.0 {
                        border_builder.add_rounded_rectangle(&border_box, &LyonBorderRadii {
                            top_left: (rect.rounded_corners.top_left.max(0.0) - border_width / 2.0).max(0.0),
                            top_right: (rect.rounded_corners.top_right.max(0.0) - border_width / 2.0).max(0.0),
                            bottom_left: (rect.rounded_corners.bottom_left.max(0.0) - border_width / 2.0).max(0.0),
                            bottom_right: (rect.rounded_corners.bottom_right.max(0.0) - border_width / 2.0).max(0.0),
                        }, Winding::Positive);
                    } else {
                        border_builder.add_rectangle(&border_box, Winding::Positive);
                    }
                    let border_path = border_builder.build();
                    let options = StrokeOptions::default().with_line_width(border_width);
                    stroke_tessellator.tessellate_path(&border_path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_border_color })).unwrap();
                }
            }
            crate::Object2d::Circle(circle) => {
                let linear_color = circle.color.to_linear();
                let border_width = circle.border_width.unwrap_or(0.0);
                let has_border = border_width > 0.0 && circle.border_color.map_or(false, |c| c.a > 0.0);
                // Draw fill (shrink if border is present)
                let fill_radius = if has_border { circle.radius - border_width } else { circle.radius };
                let mut builder = Path::builder();
                builder.add_circle(point(circle.center.x, circle.center.y), fill_radius, Winding::Positive);
                let fill_path = builder.build();
                if circle.color.a > 0.0 && fill_radius > 0.0 {
                    fill_tessellator.tessellate_path(&fill_path, &FillOptions::default(), &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                }
                // Draw inner border as a ring
                if has_border && fill_radius > 0.0 {
                    let linear_border_color = circle.border_color.unwrap().to_linear();
                    let mut border_builder = Path::builder();
                    border_builder.add_circle(point(circle.center.x, circle.center.y), fill_radius + border_width / 2.0, Winding::Positive);
                    let border_path = border_builder.build();
                    let options = StrokeOptions::default().with_line_width(border_width);
                    stroke_tessellator.tessellate_path(&border_path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_border_color })).unwrap();
                }
            }
            crate::Object2d::Line(line) => {
                let linear_color = line.color.to_linear();
                let mut builder = Path::builder();
                if line.points.len() >= 2 {
                    builder.begin(point(line.points[0].x, line.points[0].y));
                    for i in 1..line.points.len() {
                        builder.line_to(point(line.points[i].x, line.points[i].y));
                    }
                    builder.end(false);
                }
                let path = builder.build();
                if line.points.len() >= 2 && line.color.a > 0.0 {
                    let options = StrokeOptions::default().with_line_width(line.width).with_line_cap(LineCap::Round).with_line_join(LineJoin::Round);
                    stroke_tessellator.tessellate_path(&path, &options, &mut BuffersBuilder::new(&mut buffers, |vertex: StrokeVertex| ColoredVertex { position: [vertex.position().x, vertex.position().y], color: linear_color })).unwrap();
                }
            }
            crate::Object2d::Text(_) => {}
        }
    }

    let vertex_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"), contents: bytemuck::cast_slice(&buffers.vertices), usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = gfx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"), contents: bytemuck::cast_slice(&buffers.indices), usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = buffers.indices.len() as u32;

    let mut encoder = gfx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(&view),
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
        });

        if num_indices > 0 {
            render_pass.set_pipeline(&gfx.rect_pipeline);
            render_pass.set_bind_group(0, &gfx.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        match gfx.text_renderer.render(&gfx.atlas, &gfx.viewport, &mut render_pass) {
            Ok(_) => {}
            Err(e) => console::error_1(&JsValue::from_str(&format!("Error rendering text: {:?}", e))),
        }
    }
    gfx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
