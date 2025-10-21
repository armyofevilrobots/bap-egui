use crate::core::project::Orientation;
use crate::view_model::{BAPDisplayMode, BAPViewModel, CommandContext};
use eframe::egui;
use egui::{Image, ImageButton, Vec2, vec2};

use super::tool_button::tool_button;

pub(crate) fn floating_tool_window(model: &mut BAPViewModel, ctx: &egui::Context, wtop: f32) {
    let win = egui::Window::new("")
        // .auto_sized()
        .default_pos((40., 40.))
        .collapsible(false)
        .resizable([false, false]);
    let win = if !model.docked {
        win.title_bar(false)
    } else {
        win.title_bar(false)
            .anchor(egui::Align2::LEFT_TOP, (0., wtop - 3.))
    };

    win.show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.toggle_value(&mut model.docked, "📌");
        });
        // ui.separator();
        if model.display_mode == BAPDisplayMode::SVG {
            egui::Grid::new("SVGTOOLZ")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    ui.end_row();
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/paper_stack.png"),
                        Some("Choose Paper Size".to_string()),
                    )
                    .clicked()
                    {
                        // println!("Showing paper chooser w indow.");
                        model.paper_modal_open = true;
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/origin_icon.png"),
                        Some("Set origin".into()),
                    )
                    .clicked()
                    {
                        println!("Switching to origin context.");
                        model.command_context = CommandContext::Origin;
                    };
                    let portrait_landscape_button = match model.paper_orientation {
                        Orientation::Landscape => tool_button(
                            ui,
                            egui::include_image!("../../resources/images/portrait.png"),
                            Some("Change to portrait orientation".into()),
                        ),
                        Orientation::Portrait => tool_button(
                            ui,
                            egui::include_image!("../../resources/images/landscape.png"),
                            Some("Change to landscape orientation".into()),
                        ),
                    };
                    if portrait_landscape_button.clicked() {
                        model.paper_orientation = match model.paper_orientation {
                            Orientation::Landscape => Orientation::Portrait,
                            Orientation::Portrait => Orientation::Landscape,
                        }
                    };
                    ui.end_row();
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/pen_crib.png"),
                        Some("Pen Management".into()),
                    )
                    .clicked()
                    {
                        model.pen_crib_open = true;
                    };
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/aoer_logo.png"),
                        Some("About...".into()),
                    );
                    ui.end_row();
                });
            ui.collapsing("Display Toggles", |ui| {
                ui.checkbox(&mut model.show_paper, "Show paper");
                ui.checkbox(&mut model.show_machine_limits, "Show limits");
                ui.checkbox(&mut model.show_extents, "Show extents");
                ui.checkbox(&mut model.show_rulers, "Show rulers");
            });
        } else {
            egui::Grid::new("PLOT-TOOLZ")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/aoer_logo.png"),
                        Some("Set origin".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/aoer_logo.png"),
                        Some("Set origin".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/origin_icon.png"),
                        Some("Set origin".into()),
                    );
                    ui.end_row();
                });
        };
    });
}
