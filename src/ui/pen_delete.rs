use egui::Id;

use crate::view_model::{BAPViewModel, CommandContext};

pub fn pen_delete_window(model: &mut BAPViewModel, ctx: &egui::Context, pen_idx: usize) {
    egui::Modal::new(Id::new("DeletePen")).show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.set_width(400.);
            if let Some(pen) = model.pen_crib().get(pen_idx) {
                ui.heading(format!("Delete Pen #{} - {}", pen.tool_id, pen.name));
                ui.horizontal_centered(|ui| {
                    if ui.button("Cancel Delete?").clicked() {
                        model.set_command_context(CommandContext::PenCrib);
                    };
                    if ui.button("Confirm Delete!").clicked() {
                        model.pen_crib_mut().remove(pen_idx);
                        model.set_command_context(CommandContext::PenCrib);
                    }
                });
            } else {
                ui.label("Cannot find a pen with that ID. This is almost certainly a bug.");
                if ui.button("Cancel Delete?").clicked() {
                    model.set_command_context(CommandContext::PenCrib);
                };
            }
        });
    });
}
