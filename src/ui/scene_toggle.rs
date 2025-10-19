use crate::view_model::{BAPDisplayMode, BAPViewModel};
use eframe::egui;
use egui::{Align, Layout, Vec2};

pub(crate) fn scene_toggle(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::TopBottomPanel::top("scenechanger")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].allocate_ui_with_layout(
                    Vec2::ZERO,
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.selectable_value(&mut model.display_mode, BAPDisplayMode::SVG, "Edit");
                    },
                );
                columns[1].allocate_ui_with_layout(
                    Vec2::ZERO,
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.selectable_value(&mut model.display_mode, BAPDisplayMode::Plot, "Plot");
                    },
                );
            });
        });
}
