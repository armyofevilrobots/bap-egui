// use crate::ui::bottom_panel::bottom_panel;
use crate::view_model::{BAPViewModel, CommandContext};
use eframe::egui;
use egui::{Id, Layout, Slider};

pub(crate) fn scale_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    // ui: &mut egui::Ui,
) /*-> ModalResponse<()>*/
{
    egui::Modal::new(Id::new("ScaleFactor")).show(ctx, |ui| {
        ui.set_width(250.);
        ui.heading("Scale by factor");

        if let CommandContext::Scale(factor) = &mut model.command_context {
            ui.add(
                Slider::new(factor, 0.01..=100.0)
                    .custom_formatter(|val, _range| format!("{:0.1}%", val * 100.0))
                    .logarithmic(true)
                    .text("Percent"),
            );
        }
        if let CommandContext::Scale(factor) = model.command_context {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Ok").clicked() {
                    // model.pen_crib_open = false
                    model.scale_by_factor(factor.clone());
                    model.command_context = crate::view_model::CommandContext::None
                }
                if ui.button("Cancel").clicked() {
                    // model.pen_crib_open = false
                    // println!("Not scaling. Quitting modal.");
                    model.command_context = crate::view_model::CommandContext::None
                }
            });
        }
    });
}
