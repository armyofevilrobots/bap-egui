use std::collections::HashMap;

use crate::core::project::Orientation;
use crate::view_model::{BAPDisplayMode, BAPViewModel, CommandContext};
use eframe::egui;
use egui::{AtomExt, ComboBox, FontSelection, Image, ImageButton, Separator, TextEdit, Vec2, vec2};

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
            ui.add_space(8.);
            egui::Grid::new("SVGTOOLZ")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    // ui.end_row();
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
                        // println!("Switching to origin context.");
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
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/print.png"),
                        Some("Post to plot engine.".into()),
                    )
                    .clicked()
                    {};
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/zoom_fit.png"),
                        Some("Zoom to fit all on screen.".into()),
                    )
                    .clicked()
                    {
                        model.zoom_fit();
                    };
                    ui.end_row();
                });

            // ui.collapsing("Alignment", |ui| {
            ui.add_space(8.);
            ui.label("Alignment");
            egui::Grid::new("AlignmentToolz")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/align_center_paper.png"),
                        Some("Center to paper".into()),
                    )
                    .clicked()
                    {
                        println!("CENTER!!");
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/align_center_limits.png"),
                        Some("Center to machine limits".into()),
                    )
                    .clicked()
                    {
                        println!("CENTER!!");
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/smart_center.png"),
                        Some("Optimal center for paper size and machine limits".into()),
                    )
                    .clicked()
                    {};
                    ui.end_row();
                    ui.end_row();
                });
            // });
            ui.add_space(8.);
            ui.label("Display...");
            ui.checkbox(&mut model.show_paper, "Show paper");
            ui.checkbox(&mut model.show_machine_limits, "Show limits");
            ui.checkbox(&mut model.show_extents, "Show extents");
            ui.checkbox(&mut model.show_rulers, "Show rulers");
        } else
        /* if tool mode is plot mode */
        {
            ui.add_space(8.);
            // The 'serial' connection selector.
            let mut plotter = "/dev/acm0";
            let plotters = vec!["/dev/acm0", "magic-phaery-dust"];
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("Plotter Connection")
                    .selected_text(format!("{}", plotter))
                    .show_ui(ui, |ui| {
                        for plt in plotters.iter() {
                            ui.selectable_value(&mut plotter, plt.clone(), format!("{}", plt));
                        }
                    });
                ui.button(egui::include_image!(
                    "../../resources/images/plotter_connect.png"
                ));
            });
            ui.add_space(8.);

            egui::Grid::new("PLOT-TOOLZ")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/pen_up.png"),
                        Some("Pen Up (from paper)".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_up.png"),
                        Some("Move pen up (Y+)".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/cancel.png"),
                        Some("Cancel".into()),
                    );

                    ui.end_row();

                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_left.png"),
                        Some("Move pen left (X-)".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/home.png"),
                        Some("Go Home (G28 X0 Y0)".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_right.png"),
                        Some("Move pen right (X+)".into()),
                    );
                    ui.end_row();

                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/pen_down.png"),
                        Some("Pen down (on paper)".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_down.png"),
                        Some("Move pen down (Y-)".into()),
                    );
                    tool_button(
                        ui,
                        egui::include_image!("../../resources/images/play_circle.png"),
                        Some("Start/Resume plotting".into()),
                    );
                    ui.end_row();
                });
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut model.edit_cmd)
                        .min_size(vec2(72., 16.))
                        .desired_width(72.),
                );
                ui.button(egui::include_image!("../../resources/images/send_cmd.png"));
            });
        };
    });
}
