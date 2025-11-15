use crate::view_model::{BAPDisplayMode, BAPViewModel};
use eframe::egui;
use egui::{Align, FontId, Layout, Vec2, vec2};

pub(crate) fn scene_toggle(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::TopBottomPanel::top("scenechanger")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(FontId {
                size: 18.,
                ..FontId::default()
            });
            ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
            let mut display_mode = model.display_mode().clone();
            ui.style_mut().spacing.item_spacing = vec2(0., 0.);
            ui.style_mut().visuals.button_frame = true;
            ui.style_mut().visuals.menu_corner_radius = egui::CornerRadius::same(16);
            ui.style_mut().visuals.window_corner_radius = egui::CornerRadius::same(16);
            ui.columns(2, |columns| {
                columns[0].allocate_ui_with_layout(
                    Vec2::ZERO,
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        // ui.style_mut().visuals
                        ui.selectable_value(&mut display_mode, BAPDisplayMode::SVG, "Edit");
                    },
                );
                columns[1].allocate_ui_with_layout(
                    Vec2::ZERO,
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.selectable_value(&mut display_mode, BAPDisplayMode::Plot, "Plot");
                        // let b = RadioButton::new(display_mode==BAPDisplayMode::Plot, "Plot");
                    },
                );
            });
            if display_mode != model.display_mode() {
                model.set_display_mode(display_mode);
            }
        });
}
