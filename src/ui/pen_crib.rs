use std::sync::Arc;

use crate::{BAPViewModel, core::project::PenDetail, view_model::CommandContext};
use eframe::egui;
use egui::{Button, Color32, Frame, Grid, Id, Layout};

pub(crate) fn pen_crib_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    let mut drag_from: Option<Arc<usize>> = None;
    let mut drag_to: Option<Arc<usize>> = None;
    let _pen_delete: Option<usize> = None;
    egui::Modal::new(Id::new("Pen Crib")).show(ctx, |ui| {
        ui.set_width(400.);

        // This is the frame that contains the grid.
        let frame = Frame::default().inner_margin(4.0);

        let (_, dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
            let mut pens = model.pen_crib.clone();
            for (idx, pen) in pens.iter_mut().enumerate() {
                pen.tool_id = idx + 1;
            }
            Grid::new(format!("Pen-grid-draggable"))
                .striped(true)
                .show(ui, |ui| {
                    for (idx, pen) in pens.iter_mut().enumerate() {
                        let color_code = pen.color.clone();
                        // csscolorparser::parse(pen.color.as_str()).unwrap_or_default();
                        let [r, g, b, a] = color_code.to_rgba8();
                        let pen_drag_response = ui
                            .dnd_drag_source(
                                Id::new(format!("pen-drag-source-{}", idx)),
                                idx,
                                |ui| {
                                    ui.label(format!("✋{}✋", pen.name.clone()).to_string());
                                },
                            )
                            .response;
                        if ui
                            .add(
                                Button::new("Edit")
                                    .fill(Color32::from_rgba_premultiplied(r, g, b, a)),
                            )
                            .clicked()
                        {
                            model.command_context =
                                crate::view_model::CommandContext::PenEdit(idx, pen.clone());
                        };
                        if ui
                            .add_enabled(model.picked.is_some(), Button::new("⤵Apply"))
                            .clicked()
                        {
                            model.yolo_view_command(
                                crate::core::commands::ViewCommand::ApplyPenToSelection(
                                    pen.tool_id.clone(),
                                ),
                            );
                        }
                        ui.label(format!("○ {:3.2}mm", pen.stroke_width));
                        ui.label(format!("◑ {:3.2}%", pen.stroke_density * 100.));
                        match pen.feed_rate {
                            Some(rate) => ui.label(format!("🔁 {:5.1}mm/min", rate)),
                            None => ui.label("Default feedrate"),
                        };
                        ui.label(format!("🆔 {}", pen.tool_id));
                        if ui.button("♻Delete").clicked() {
                            model.command_context = CommandContext::PenDelete(idx);
                        }
                        ui.end_row();
                        if let (Some(pointer), Some(hovered_payload)) = (
                            ui.input(|i| i.pointer.interact_pos()),
                            pen_drag_response.dnd_hover_payload::<usize>(),
                        ) {
                            let rect = pen_drag_response.rect;
                            let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                            // println!("Hovered payload {} idx {}", hovered_payload, idx);
                            let insert_idx = if *hovered_payload == idx {
                                // We are dragged onto ourselves
                                // println!("SELF");
                                ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                idx
                            } else if pointer.y < rect.center().y {
                                // Above us
                                // println!("ABOVE");
                                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                // Because we move the destination UP when we remove the source
                                idx.max(0) - if *hovered_payload < idx { 1 } else { 0 }
                            } else {
                                // Below us
                                // println!("BELOW");
                                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                // Because we move the destination UP when we remove the source
                                idx.min(model.pen_crib.len() - 1)
                                    + if *hovered_payload < idx { 0 } else { 1 }
                            };
                            // println!("Dragging from {} to {}", hovered_payload, insert_idx);
                            if let Some(dragged_payload) = pen_drag_response.dnd_release_payload() {
                                // The user dropped onto this item.
                                drag_from = Some(dragged_payload);
                                drag_to = Some(Arc::new(insert_idx));
                            }
                        }
                    }
                });
        }); // End of DND dropzone frame wrapper
        if let Some(_payload) = dropped_payload {
            // The user dropped onto the column, but not on any one item.
        } else if let Some(from) = drag_from
            && let Some(to) = drag_to
        {
            let tmp = model.pen_crib.remove(*from);
            model.pen_crib.insert(*to, tmp);
        }
        for (idx, pen) in model.pen_crib.iter_mut().enumerate() {
            pen.tool_id = idx + 1;
        }

        if ui.button("⊞ADD").clicked() {
            let pen_id = model.pen_crib.len() + 1;
            model.pen_crib.push(PenDetail {
                tool_id: pen_id,
                ..PenDetail::default()
            });
            model.command_context = crate::view_model::CommandContext::PenEdit(
                pen_id - 1,
                model.pen_crib.last().unwrap().clone(),
            );
        };

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Ok").clicked() {
                model.update_pen_details();
                model.command_context = crate::view_model::CommandContext::None
            }
        });
    });
}
