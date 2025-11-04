use std::f64::consts::PI;

use csscolorparser::Color;
use egui::{Color32, Id, Layout, Rect, Slider, Stroke, StrokeKind, epaint::PathStroke, pos2, vec2};

use crate::{core::project::PenDetail, view_model::BAPViewModel};

pub fn pen_editor_window(model: &mut BAPViewModel, ctx: &egui::Context, pen_idx: usize) {
    egui::Modal::new(Id::new("Pen Editor")).show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.set_width(400.);
            ui.heading(format!(
                "Edit Pen #{} - {}",
                pen_idx,
                model
                    .pen_crib
                    .get(pen_idx)
                    .unwrap_or(&PenDetail::default())
                    .name
            ));
            let (painter_resp, painter) = ui.allocate_painter(vec2(390., 420.), egui::Sense::all());
            let prect = painter_resp.rect;
            let ofs = (prect.min.clone() + vec2(10., 10.)).to_vec2();
            let pen_crib_len = model.pen_crib.len();
            let pen = model.pen_crib.get_mut(pen_idx).expect(
                format!(
                    "Somehow pen indexes got mangled getting pen {} from pen crib of length {}",
                    pen_idx, pen_crib_len
                )
                .as_str(),
            );
            let color_code = csscolorparser::parse(pen.color.as_str()).unwrap_or_default();
            let [r, g, b, a] = color_code.to_linear_rgba_u8();
            let mut pen_color32 = Color32::from_rgba_premultiplied(r, g, b, a);

            // Editor for the TOOL ID (this is a tracking ID, not the machine ID)
            ui.allocate_ui_at_rect(
                Rect::from_min_max(pos2(0., 0.) + ofs, prect.min + vec2(250., 20.)),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut pen.name);
                    });
                },
            );

            // ui.allocate_ui_at_rect(
            //     Rect::from_min_max(pos2(0., 30.) + ofs, prect.min + vec2(250., 50.)),
            //     |ui| {
            //         ui.horizontal(|ui| {
            //             ui.label("Pen/Tool ID");
            //             ui.add(Slider::new(&mut pen.tool_id, 0..=100));
            //         })
            //     },
            // );

            ui.allocate_ui_at_rect(
                Rect::from_min_max(pos2(0., 80.) + ofs, prect.min + vec2(200., 120.)),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Feed(mm/s)");
                        let (mut enabled, mut tmp_vel) = match pen.feed_rate.clone() {
                            Some(vel) => (true, vel),
                            None => (false, 1200.),
                        };
                        ui.checkbox(&mut enabled, "Custom");
                        if enabled {
                            ui.add(Slider::new(&mut tmp_vel, 100.0..=10000.0));
                        } else {
                            ui.label(" - machine default -");
                        }

                        if enabled {
                            pen.feed_rate = Some(tmp_vel);
                        } else {
                            pen.feed_rate = None
                        }
                    })
                },
            );

            // Create the pen color picker
            let pen_color_response = ui.allocate_ui_at_rect(
                Rect::from_min_max(pos2(0., 150.) + ofs, pos2(300.0, 250.0) + ofs),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Pen Color:");
                        if ui
                            .color_edit_button_srgba(&mut pen_color32)
                            .on_hover_text("Change Pen Color")
                            .changed()
                        {
                            pen.color = Color::from_linear_rgba8(
                                pen_color32.r(),
                                pen_color32.g(),
                                pen_color32.b(),
                                pen_color32.a(),
                            )
                            .to_css_hex();
                        }
                    })
                },
            );

            // The input for the pen width
            let pen_width_slider_response = ui.allocate_ui_at_rect(
                Rect::from_min_max(pos2(0., 240.) + ofs, pos2(300.0, 270.0) + ofs),
                |ui| {
                    ui.add(
                        Slider::new(&mut pen.stroke_width, 0.1..=10.0)
                            .logarithmic(true)
                            .text("Width"),
                    )
                },
            );

            // The input for the pen density
            let pen_density_slider_response = ui.allocate_ui_at_rect(
                Rect::from_min_max(pos2(0., 350.) + ofs, pos2(300.0, 380.0) + ofs),
                |ui| {
                    ui.add(
                        Slider::new(&mut pen.stroke_density, 0.05..=1.0)
                            .logarithmic(true)
                            .text("Density"),
                    )
                },
            );

            let _ok_clicked_response = ui.allocate_ui_at_rect(
                Rect::from_min_max(pos2(0., 400.) + ofs, pos2(390.0, 420.0) + ofs),
                |ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Ok").clicked() {
                            // model.pen_crib_open = false
                            model.command_context = crate::view_model::CommandContext::PenCrib
                        }
                    });
                },
            );

            let pen_slope_right = vec2(
                -25. + 5.0 * pen.stroke_width as f32 / 2.,
                100.0 - (10.0 * pen.stroke_width as f32) * (15. * PI / 180.).cos() as f32,
            ); // -25. is sin(15)*100;
            let pen_slope_left = vec2(
                -25. - 5.0 * pen.stroke_width as f32 / 2.,
                -100.0 - (10.0 * pen.stroke_width as f32) * (15. * PI / 180.).cos() as f32,
            ); // -25. is sin(15)*100;

            let pen_right_tip = pos2(350., 200.) + ofs + pen_slope_right;
            let pen_left_tip =
                pos2(350., 200.) + ofs + pen_slope_right + vec2(-5.0 * pen.stroke_width as f32, 0.);

            painter.rect_filled(
                Rect::from_min_max(pos2(300., 0.) + ofs, pos2(350., 200.) + ofs),
                0.0,
                pen_color32.clone(),
            );

            painter.line(
                vec![
                    pos2(300., 200.) + ofs,
                    pos2(300., 0.) + ofs,
                    pos2(350., 0.) + ofs,
                    pos2(350., 200.) + ofs,
                    pen_right_tip.clone(),
                    pen_left_tip.clone(),
                    pos2(300., 200.) + ofs,
                ],
                PathStroke::new(3., ui.visuals().text_color()),
            );

            // Dimension line vertical right
            painter.line(
                vec![
                    pen_right_tip.clone() + vec2(0.0, 20.0),
                    pen_right_tip.clone() + vec2(0.0, 10.0),
                    // pen_right_tip.clone() + vec2(0.0, 20.0),
                    // pen_left_tip.clone() + vec2(0.0, 20.0),
                    // pen_left_tip.clone() + vec2(0.0, 30.0),
                    // pen_left_tip.clone() + vec2(0.0, 10.0),
                ],
                PathStroke::new(1., ui.visuals().text_color()),
            );
            // Dimension line vertical left
            painter.line(
                vec![
                    pen_left_tip.clone() + vec2(0.0, 20.0),
                    pen_left_tip.clone() + vec2(0.0, 10.0),
                ],
                PathStroke::new(1., ui.visuals().text_color()),
            );
            // Dimension line horizontal join
            painter.line(
                vec![
                    pen_right_tip.clone() + vec2(0.0, 15.0),
                    pen_left_tip.clone() + vec2(0.0, 15.0),
                ],
                PathStroke::new(1., ui.visuals().text_color()),
            );

            // Line attaching that to the slider.
            painter.line(
                vec![
                    pen_right_tip.clone() + vec2(-5.0 * (pen.stroke_width as f32) / 2., 15.0),
                    pen_right_tip.clone() + vec2(-5.0 * (pen.stroke_width as f32) / 2., 25.0),
                    pen_right_tip.clone()
                        + vec2(-5.0 * (pen.stroke_width as f32) / 2. - 50.0, 25.0),
                    pen_width_slider_response.response.rect.right_center() + vec2(70.0, 0.0),
                    pen_width_slider_response.response.rect.right_center() + vec2(10.0, 0.0),
                ],
                PathStroke::new(1., ui.visuals().text_color()),
            );

            let paper_top = pen_left_tip.y + 5.;

            // Draw the paper.
            painter.rect(
                Rect::from_min_max(
                    pen_left_tip + vec2(-10., 30.),
                    pen_right_tip + vec2(10., 80.),
                ),
                0.,
                model.paper_color.clone(),
                Stroke::NONE,
                StrokeKind::Outside,
            );

            // Simulate the density of the pen with a bunch of hatchlines.
            for i in 0..(1 + (8. * pen.stroke_width) as usize / 2) {
                painter.line(
                    vec![
                        pen_left_tip + vec2(0. + i as f32 * 10. / 8., 30.),
                        pen_left_tip + vec2(0. + i as f32 * 10. / 8., 80.),
                    ],
                    PathStroke::new((11. / 8. * pen.stroke_density as f32), pen_color32.clone()),
                );
            }

            // Line attaching that to the slider.
            painter.line(
                vec![
                    pen_right_tip.clone() + vec2(-5.0 * (pen.stroke_width as f32) / 2., 90.0),
                    pen_right_tip.clone() + vec2(-5.0 * (pen.stroke_width as f32) / 2., 100.0),
                    pen_right_tip.clone()
                        + vec2(-5.0 * (pen.stroke_width as f32) / 2. - 50., 100.0),
                    pen_density_slider_response.response.rect.right_center() + vec2(70.0, 0.0),
                    pen_density_slider_response.response.rect.right_center() + vec2(10.0, 0.0),
                ],
                PathStroke::new(1., ui.visuals().text_color()),
            );
            // ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // });
        });
    });
}
