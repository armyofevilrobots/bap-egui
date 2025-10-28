use std::sync::Arc;

use crate::{BAPViewModel, core::project::PenDetail};
use eframe::egui;
use egui::{
    Button, Color32, Frame, Grid, Id, Image, InnerResponse, Layout, Modal, Response, Rgba, Slider,
    Ui, Vec2, Widget, epaint::Hsva, style::HandleShape,
};

pub(crate) fn pen_crib_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    let mut drag_from: Option<Arc<usize>> = None;
    let mut drag_to: Option<Arc<usize>> = None;
    egui::Modal::new(Id::new("Pen Crib")).show(ctx, |ui| {
        ui.set_width(400.);

        // This is the frame that contains the grid.
        let frame = Frame::default().inner_margin(4.0);

        let (_, dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
            let mut pens = model.pen_crib.clone();
            Grid::new(format!("Pen-grid-draggable"))
                .striped(true)
                .show(ui, |ui| {
                    for (idx, pen) in pens.iter_mut().enumerate() {
                        let color_code =
                            csscolorparser::parse(pen.color.as_str()).unwrap_or_default();
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
                            model.command_context = crate::view_model::CommandContext::PenEdit(idx);
                        };
                        ui.label(format!("○ {:3.2}mm", pen.stroke_width));
                        ui.label(format!("◑ {:3.2}%", pen.stroke_density * 100.));
                        match pen.feed_rate {
                            Some(rate) => ui.label(format!("🔁 {:5.1}mm/min", rate)),
                            None => ui.label("Default feedrate"),
                        };
                        ui.label(format!("🆔 {}", pen.tool_id));
                        ui.end_row();
                        if let (Some(pointer), Some(hovered_payload)) = (
                            ui.input(|i| i.pointer.interact_pos()),
                            pen_drag_response.dnd_hover_payload::<usize>(),
                        ) {
                            let rect = pen_drag_response.rect;
                            let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                            let insert_idx = if *hovered_payload == idx {
                                // We are dragged onto ourselves
                                ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                idx
                            } else if pointer.y < rect.center().y {
                                // Above us
                                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                idx.max(1) - 1
                            } else {
                                // Below us
                                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                idx.min(model.pen_crib.len() - 2) + 1
                            };
                            if let Some(dragged_payload) = pen_drag_response.dnd_release_payload() {
                                // The user dropped onto this item.
                                drag_from = Some(dragged_payload);
                                drag_to = Some(Arc::new(insert_idx));
                            }
                        }
                    }
                });
        }); // End of DND dropzone frame wrapper
        if let Some(payload) = dropped_payload {
            // The user dropped onto the column, but not on any one item.
        } else if let Some(from) = drag_from
            && let Some(to) = drag_to
        {
            let tmp = model.pen_crib.remove(*from);
            model.pen_crib.insert(*to, tmp);
        }

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Ok").clicked() {
                // model.pen_crib_open = false
                model.command_context = crate::view_model::CommandContext::None
            }
        });
    });
}
