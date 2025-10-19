use crate::BAPViewModel;
use eframe::egui;
use egui::{Id, Layout};

pub(crate) fn pen_crib_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::Modal::new(Id::new("Pen Crib")).show(ctx, |ui| {
        ui.set_width(400.);
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                model.pen_crib_open = false
            }
            if ui.button("Ok").clicked() {
                model.pen_crib_open = false
            }
        });
    });
}
