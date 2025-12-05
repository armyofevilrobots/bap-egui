use crate::view_model::{BAPDisplayMode, BAPViewModel};
use eframe::egui;
use egui::{Align, Button, Color32, CornerRadius, FontId, Layout, Stroke, Ui, vec2};

pub(crate) fn scene_toggle_inner(model: &mut BAPViewModel, _ctx: &egui::Context, ui: &mut Ui) {
    ui.style_mut().override_font_id = Some(FontId {
        size: 18.,
        ..FontId::default()
    });
    ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
    let mut display_mode = model.display_mode().clone();
    ui.style_mut().spacing.item_spacing = vec2(0., 4.);
    ui.style_mut().spacing.button_padding = vec2(6., 2.);
    ui.style_mut().visuals.button_frame = true;
    ui.style_mut().visuals.menu_corner_radius = egui::CornerRadius::same(16);
    ui.style_mut().visuals.window_corner_radius = egui::CornerRadius::same(16);
    ui.vertical_centered(|ui| {
        ui.columns(2, |columns| {
            columns[0].allocate_ui_with_layout(
                // Vec2::ZERO,
                vec2(64., 32.),
                Layout::right_to_left(Align::Center),
                |ui| {
                    // ui.style_mut().visuals
                    // ui.selectable_value(&mut display_mode, BAPDisplayMode::SVG, "Edit");
                    // ui.style_mut().visuals.button_frame = true;
                    ui.style_mut().visuals.override_text_color = Some(match &display_mode {
                        BAPDisplayMode::SVG => ui.style().visuals.strong_text_color(),
                        BAPDisplayMode::Plot => ui.style().visuals.weak_text_color(),
                    });
                    let svg_button = Button::new("Edit")
                        .corner_radius(CornerRadius {
                            nw: 8,
                            ne: 0,
                            sw: 8,
                            se: 0,
                        })
                        .stroke(Stroke::new(
                            0.,
                            match display_mode {
                                BAPDisplayMode::SVG => ui.style().visuals.text_color(),
                                BAPDisplayMode::Plot => ui.style().visuals.weak_text_color(),
                            },
                        ))
                        .fill(match display_mode {
                            BAPDisplayMode::SVG => ui.style().visuals.extreme_bg_color,
                            BAPDisplayMode::Plot => ui.style().visuals.faint_bg_color,
                        })
                        .frame(true);
                    if ui.add(svg_button).clicked() {
                        display_mode = BAPDisplayMode::SVG;
                    };
                },
            );
            columns[1].allocate_ui_with_layout(
                // Vec2::ZERO,
                vec2(64., 32.),
                Layout::left_to_right(Align::Center),
                |ui| {
                    // ui.selectable_value(&mut display_mode, BAPDisplayMode::Plot, "Plot");
                    // let b = RadioButton::new(display_mode==BAPDisplayMode::Plot, "Plot");
                    ui.style_mut().visuals.override_text_color = Some(match &display_mode {
                        BAPDisplayMode::Plot => ui.style().visuals.strong_text_color(),
                        BAPDisplayMode::SVG => ui.style().visuals.weak_text_color(),
                    });
                    let plot_button = Button::new("Plot")
                        .corner_radius(CornerRadius {
                            nw: 0,
                            ne: 8,
                            sw: 0,
                            se: 8,
                        })
                        .stroke(Stroke::new(
                            0.,
                            match display_mode {
                                BAPDisplayMode::Plot => ui.style().visuals.text_color(),
                                BAPDisplayMode::SVG => ui.style().visuals.weak_text_color(),
                            },
                        ))
                        .fill(match display_mode {
                            BAPDisplayMode::Plot => ui.style().visuals.extreme_bg_color,
                            BAPDisplayMode::SVG => ui.style().visuals.faint_bg_color,
                        })
                        .frame(true);
                    if ui.add(plot_button).clicked() {
                        display_mode = BAPDisplayMode::Plot;
                    };
                },
            );
        });
    });
    if display_mode != model.display_mode() {
        model.set_display_mode(display_mode);
    }
}

pub(crate) fn scene_toggle(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::TopBottomPanel::top("scenechanger")
        .show_separator_line(false)
        // .frame(Frame::new().outer_margin(Margin {
        //     left: 0,
        //     right: 0,
        //     top: -16,
        //     bottom: 0,
        // }))
        .show(ctx, |ui| {
            scene_toggle_inner(model, ctx, ui);
        });
}
