use std::f64::consts::PI;

use csscolorparser::Color;
use egui::{Color32, Id, Layout, Rect, Slider, Stroke, StrokeKind, epaint::PathStroke, pos2, vec2};

use crate::{
    core::project::PenDetail,
    view_model::{BAPViewModel, CommandContext},
};

pub fn pen_delete_window(model: &mut BAPViewModel, ctx: &egui::Context, pen_idx: usize) {
    egui::Modal::new(Id::new("DeletePen")).show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.set_width(400.);
            if let Some(pen) = model.pen_crib.get(pen_idx) {
                ui.heading(format!("Delete Pen #{} - {}", pen.tool_id, pen.name));
                ui.horizontal_centered(|ui| {
                    if ui.button("Cancel Delete?").clicked() {
                        model.command_context = CommandContext::PenCrib;
                    };
                    if ui.button("Confirm Delete!").clicked() {
                        model.pen_crib.remove(pen_idx);
                        model.command_context = CommandContext::PenCrib;
                    }
                });
            } else {
                ui.label("Cannot find a pen with that ID. This is almost certainly a bug.");
                if ui.button("Cancel Delete?").clicked() {
                    model.command_context = CommandContext::PenCrib;
                };
            }
        });
    });
}
