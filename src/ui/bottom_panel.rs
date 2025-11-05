use crate::BAPViewModel;
use eframe::egui;
use egui::{Layout, ProgressBar};

pub fn bottom_panel(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("BottomPanel")
        .show_separator_line(true)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if let Some((msg, progress)) = &model.progress {
                    ui.add(
                        ProgressBar::new(*progress as f32 / 100.)
                            .desired_width(320.)
                            .text(msg),
                    );
                } else {
                    ui.add(
                        ProgressBar::new(1.)
                            .text(model.status_msg.clone().unwrap_or("Ready.".to_string()))
                            .desired_width(320.),
                    );
                }
                if let Some(pos) = ctx.pointer_latest_pos() {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let pos = model.frame_coords_to_mm(pos);
                        ui.label(format!("❌X{:.2},Y{:.2}", model.origin.x, model.origin.y));
                        ui.label(format!("↖X{:.2},Y{:.2}", pos.x, pos.y));
                        ui.label(format!("🔍{:.2}%", 100.0 * model.zoom() / 11.50)); // This is just a weird zoom factor correction.
                    });
                };
            });
        });
}
