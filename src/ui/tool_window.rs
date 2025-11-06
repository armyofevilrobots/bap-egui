use crate::core::project::Orientation;
use crate::sender::PlotterState;
use crate::view_model::{BAPDisplayMode, BAPViewModel, CommandContext};
use eframe::egui;
use egui::{ComboBox, Slider, TextEdit, vec2};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use super::tool_button::tool_button;

pub(crate) fn floating_tool_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    wtop: f32,
    toasts: &mut Toasts,
) {
    let win = egui::Window::new("")
        // .auto_sized()
        .default_pos((40., 40.))
        .collapsible(false)
        .resizable([false, false]);
    let win = if !model.docked {
        win.title_bar(false)
    } else {
        win.title_bar(false)
            .anchor(egui::Align2::LEFT_TOP, (25.0, wtop + 49.))
    };

    win.show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.toggle_value(&mut model.docked, "📌");
        });
        // ui.separator();
        if model.display_mode() == BAPDisplayMode::SVG {
            ui.add_space(8.);
            egui::Grid::new("SVGTOOLZ")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    // ui.end_row();
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/paper_stack.png"),
                        Some("Choose Paper Size".to_string()),
                        true,
                    )
                    .clicked()
                    {
                        // println!("Showing paper chooser w indow.");
                        // model.paper_modal_open = true;
                        model.command_context = CommandContext::PaperChooser;
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/origin_icon.png"),
                        Some("Set origin".into()),
                        true,
                    )
                    .clicked()
                    {
                        // println!("Switching to origin context.");
                        model.command_context = CommandContext::Origin;
                    };
                    let portrait_landscape_button = match model.paper_orientation() {
                        Orientation::Landscape => tool_button(
                            ui,
                            egui::include_image!("../../resources/images/portrait.png"),
                            Some("Change to portrait orientation".into()),
                            true,
                        ),
                        Orientation::Portrait => tool_button(
                            ui,
                            egui::include_image!("../../resources/images/landscape.png"),
                            Some("Change to landscape orientation".into()),
                            true,
                        ),
                    };
                    if portrait_landscape_button.clicked() {
                        model.set_paper_orientation(
                            &match model.paper_orientation() {
                                Orientation::Landscape => Orientation::Portrait,
                                Orientation::Portrait => Orientation::Landscape,
                            },
                            true,
                        );
                    };
                    ui.end_row();
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/pen_crib.png"),
                        Some("Pen Management".into()),
                        true,
                    )
                    .clicked()
                    {
                        // model.pen_crib_open = true;
                        model.command_context = CommandContext::PenCrib;
                    };
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/print.png"),
                        Some("Post to plot engine.".into()),
                        match model.plotter_state {
                            PlotterState::Running(_, _, _) => false,
                            _ => true,
                        } && model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.request_post();
                        toasts.add(Toast {
                            kind: ToastKind::Info,
                            text: format!("Posting program...").into(),
                            options: ToastOptions::default()
                                .duration_in_seconds(10.)
                                .show_progress(true),
                            ..Default::default()
                        });
                    };
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/zoom_fit.png"),
                        Some("Zoom to fit all on screen.".into()),
                        true,
                    )
                    .clicked()
                    {
                        model.zoom_fit();
                    };
                    ui.end_row();
                });

            // ui.collapsing("Alignment", |ui| {
            ui.add_space(16.);
            // Label::new("Alignment");
            egui::Grid::new("AlignmentToolz")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/align_center_paper.png"),
                        Some("Center to paper".into()),
                        model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.center_paper();
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/align_center_limits.png"),
                        Some("Center to machine limits".into()),
                        model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.center_machine();
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/smart_center.png"),
                        Some("Optimal center for paper size and machine limits".into()),
                        model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.center_smart();
                    };
                    ui.end_row();
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/scale.png"),
                        Some("Scale by a factor".into()),
                        model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.command_context = CommandContext::Scale(1.);
                    }

                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/rotate_right.png"),
                        Some("Free Rotate".into()),
                        model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.command_context = CommandContext::Rotate(None, None, None);
                    }

                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/expand.png"),
                        Some("Scale to fit paper/machine with matting".into()),
                        model.source_image_extents.is_some(),
                    )
                    .clicked()
                    {
                        model.command_context = CommandContext::Scale(1.);
                    }

                    ui.end_row();
                });
            // });
            ui.add_space(16.);
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
            // let mut plotter = "/dev/acm0";
            // let plotters = vec!["/dev/acm0", "magic-phaery-dust"];
            let last_port = model.current_port.clone();
            ui.horizontal(|ui| {
                let cb_resp = ComboBox::from_id_salt("Plotter Connection")
                    .selected_text(format!(
                        "{}",
                        model.current_port.replace("serial:///dev/", "")
                    ))
                    .width(72.)
                    .truncate()
                    .show_ui(ui, |ui| {
                        for plt in model.serial_ports.iter() {
                            if ui
                                .selectable_value(
                                    &mut model.current_port,
                                    plt.clone(),
                                    format!("{}", plt.replace("serial:///dev/", "")),
                                )
                                .clicked()
                            {
                                match model.plotter_state {
                                    PlotterState::Disconnected => (),
                                    _ => {
                                        model.current_port = last_port.clone();
                                        toasts.add(Toast {
                                            kind: ToastKind::Error,
                                            text: format!(
                                                "Cannot change port while plotter is connected."
                                            )
                                            .into(),
                                            options: ToastOptions::default(),
                                            ..Default::default()
                                        });
                                    }
                                }
                            };
                        }
                    });
                if cb_resp.response.changed() {
                    println!("Got a change on serial selector.");
                    match model.plotter_state {
                        PlotterState::Disconnected => model.current_port = last_port.clone(),
                        _ => (),
                    }
                }
                if ui
                    .button(match model.plotter_state {
                        PlotterState::Disconnected => {
                            egui::include_image!("../../resources/images/plotter_connect.png")
                        }
                        PlotterState::Failed(_) => {
                            egui::include_image!("../../resources/images/plotter_connect.png")
                        }
                        _ => egui::include_image!("../../resources/images/plotter_disconnect.png"),
                    })
                    .clicked()
                {
                    match model.plotter_state {
                        PlotterState::Disconnected => model.set_serial(&model.current_port),
                        PlotterState::Dead => model.set_serial(&model.current_port),
                        PlotterState::Running(_, _, _) => {
                            toasts.add(Toast {
                                kind: ToastKind::Error,
                                text: format!("Cannot close connection while plotter is running.")
                                    .into(),
                                options: ToastOptions::default(),
                                ..Default::default()
                            });
                        }
                        _ => {
                            model.close_serial();
                            // model.current_port = last_port.clone();
                        }
                    }
                }
            });
            ui.add_space(8.);

            egui::Grid::new("PLOT-TOOLZ")
                .spacing(vec2(0., 5.))
                .show(ui, |ui| {
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/pen_up.png"),
                        Some("Pen Up (from paper)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.pen_up();
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_up.png"),
                        Some("Move pen up (Y+)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.request_relative_move(vec2(0., model.move_increment));
                    };
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/cancel.png"),
                        Some("Cancel".into()),
                        match model.plotter_state {
                            PlotterState::Running(_, _, _) => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.plot_cancel();
                    };

                    ui.end_row();

                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_left.png"),
                        Some("Move pen left (X-)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.request_relative_move(vec2(-model.move_increment, 0.));
                    };

                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/home.png"),
                        Some("Go Home (G28 X0 Y0)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.send_command(&"G28 X0 Y0".to_string());
                    }
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_right.png"),
                        Some("Move pen right (X+)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.request_relative_move(vec2(model.move_increment, 0.));
                    }
                    ui.end_row();

                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/pen_down.png"),
                        Some("Pen down (on paper)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.pen_down();
                    };
                    if tool_button(
                        ui,
                        egui::include_image!("../../resources/images/move_arrow_down.png"),
                        Some("Move pen down (Y-)".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        model.request_relative_move(vec2(0., -model.move_increment));
                    }

                    if tool_button(
                        ui,
                        if let PlotterState::Running(_, _, _) = model.plotter_state {
                            egui::include_image!("../../resources/images/pause_circle.png")
                        } else {
                            egui::include_image!("../../resources/images/play_circle.png")
                        },
                        Some("Start/Resume plotting".into()),
                        match model.plotter_state {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            PlotterState::Running(_, _, _) => true,
                            PlotterState::Failed(_) => true,
                            _ => false,
                        },
                    )
                    .clicked()
                    {
                        match model.plotter_state {
                            PlotterState::Running(_, _, _) => model.plot_pause(),
                            PlotterState::Paused(_, _, _) => model.plot_start(),
                            PlotterState::Ready => model.plot_start(),
                            _ => (),
                        }
                    };
                    ui.end_row();
                });
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.style_mut().spacing.slider_width = 48.;
                ui.add(
                    Slider::new(&mut model.move_increment, 0.1..=100.0)
                        .suffix("mm")
                        .logarithmic(true)
                        .fixed_decimals(1),
                );
            });
            ui.add_space(8.);
            ui.horizontal(|ui| {
                let cmd_response = ui.add_enabled(
                    match model.plotter_state {
                        PlotterState::Ready => true,
                        PlotterState::Paused(_, _, _) => true,
                        _ => false,
                    },
                    TextEdit::singleline(&mut model.edit_cmd)
                        .min_size(vec2(72., 16.))
                        .desired_width(72.),
                );
                let mut viz_cue = false;
                if cmd_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    model.send_command(&model.edit_cmd);
                    viz_cue = true;
                }
                let mut but_resp =
                    ui.button(egui::include_image!("../../resources/images/send_cmd.png"));
                if viz_cue {
                    but_resp = but_resp.highlight();
                }

                if but_resp.clicked() {
                    match model.plotter_state {
                        PlotterState::Ready => model.send_command(&model.edit_cmd),
                        PlotterState::Paused(_, _, _) => model.send_command(&model.edit_cmd),
                        _ => {
                            toasts.add(Toast {
                                kind: ToastKind::Warning,
                                text: "Cannot send plotter command right now.".into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(5.0)
                                    .show_progress(true),
                                ..Default::default()
                            });
                        }
                    };
                };
            });
        };
    });
}
