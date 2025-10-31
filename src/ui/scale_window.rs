// use crate::ui::bottom_panel::bottom_panel;
use crate::ui::menu::main_menu;
use crate::ui::paper_chooser::paper_chooser_window;
use crate::ui::pen_crib::pen_crib_window;
use crate::view_model::{BAPViewModel, CommandContext};
use eframe::egui;
use egui::Direction::BottomUp;
use egui::{Align2, Color32, Id, Key, Layout, Rect, Slider, Stroke, StrokeKind, pos2};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

pub(crate) fn scale_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    // ui: &mut egui::Ui,
) /*-> ModalResponse<()>*/
{
    egui::Modal::new(Id::new("ScaleFactor")).show(ctx, |ui| {
        ui.set_width(400.);
        ui.heading("Scale by factor");

        if let CommandContext::Scale(factor) = &mut model.command_context {
            ui.add(
                Slider::new(factor, 0.01..=100.0)
                    .logarithmic(true)
                    .text("Width"),
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
                    println!("Not scaling. Quitting modal.");
                    model.command_context = crate::view_model::CommandContext::None
                }
            });
        }
    });
}
