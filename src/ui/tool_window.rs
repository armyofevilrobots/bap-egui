use crate::core::config::DockPosition;
use crate::core::config::RulerOrigin;
use crate::core::project::Orientation;
use crate::core::sender::PlotterState;
use crate::ui::tool_button::toggle_button;
use crate::view_model::{BAPDisplayMode, BAPViewModel, CommandContext};
use eframe::egui;
use egui::Button;
use egui::Color32;
use egui::CornerRadius;
use egui::Frame;
use egui::RichText;
use egui::Stroke;
use egui::Vec2b;
use egui::{ComboBox, Pos2, Sense, Slider, TextEdit, vec2};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

use super::tool_button::tool_button;

pub(crate) fn floating_tool_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    wtop: f32,
    toasts: &mut Toasts,
) {
    let default_height = ctx.content_rect().height() - wtop - 22.; //23.;
    let win = egui::Window::new("")
        // .auto_sized()
        .frame(
            Frame::new()
                .fill(
                    ctx.style()
                        .visuals
                        .window_fill
                        .to_opaque()
                        .blend(ctx.style().visuals.faint_bg_color),
                )
                .inner_margin(8.)
                .corner_radius(0)
                .stroke(Stroke::NONE),
        )
        .default_pos((32., 32.))
        .collapsible(false)
        .resizable([false, false]);
    let win = match model.toolbar_position() {
        DockPosition::Floating(_x, _y) => win.title_bar(false), //.current_pos(Pos2 { x, y }),
        DockPosition::Left => {
            let ofs = if model.show_rulers() {
                (24.0, wtop + 74.)
            } else {
                (2., wtop + 49.)
            };
            win.title_bar(false)
                .anchor(egui::Align2::LEFT_TOP, ofs)
                .default_height(default_height)
                .min_height(default_height)
                .max_height(default_height)
        }
        DockPosition::Right => {
            let ofs = if model.show_rulers() {
                (25.0, wtop + 74.)
            } else {
                (2., wtop + 49.)
            };

            win.title_bar(false)
                .anchor(egui::Align2::RIGHT_TOP, ofs)
                .default_height(default_height)
                .min_height(default_height)
                .max_height(default_height)
        }
    };

    let win_response = win.show(ctx, |ui| {
        ui.horizontal(|ui| {
            let mut docked = if let DockPosition::Floating(_x, _y) = model.toolbar_position() {
                false
            } else {
                true
            };
            let dock_response = ui.toggle_value(&mut docked, "📌");
            model.set_toolbar_position(&if docked {
                match model.toolbar_position() {
                    DockPosition::Left => DockPosition::Left,
                    DockPosition::Right => DockPosition::Right,
                    DockPosition::Floating(x, _y) => {
                        if x > ctx.viewport_rect().width() / 2. {
                            DockPosition::Right
                        } else {
                            DockPosition::Left
                        }
                    }
                }
            } else {
                let Pos2 { x, y } = ui.min_rect().min;
                DockPosition::Floating(x, y)
            });
            if dock_response.clicked() {
                model.update_core_config_from_changes();
            };
        });
        // ui.separator();
        // ui.shrink_width_to_current();
        // super::scene_toggle::scene_toggle_toolbox(model, ctx, ui);
        egui::ScrollArea::vertical().show(ui, |ui| {
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
                            model.set_command_context(CommandContext::PaperChooser);
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
                            model.set_command_context(CommandContext::Origin);
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
                            let new_orientation = &match model.paper_orientation() {
                                Orientation::Landscape => Orientation::Portrait,
                                Orientation::Portrait => Orientation::Landscape,
                            };
                            model.set_paper_orientation(new_orientation, true);
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
                            model.set_command_context(CommandContext::PenCrib);
                        };

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/machine_icon.png"),
                            Some("Edit Machine/Post".into()),
                            true,
                        )
                        .clicked()
                        {
                            // model.pen_crib_open = true;
                            model.set_command_context(CommandContext::MachineEdit(Some(
                                model.machine_config().clone(),
                            )));
                        };

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/print.png"),
                            Some("Post to plot engine.".into()),
                            match model.plotter_state() {
                                PlotterState::Running(_, _, _) => false,
                                _ => true,
                            } && model.source_image_extents().is_some(),
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
                        ui.end_row();
                        // ui.add_space(0.);

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
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.center_paper();
                        }
                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/align_center_limits.png"),
                            Some("Center to machine limits".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.center_machine();
                        }
                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/smart_center.png"),
                            Some("Optimal center for paper size and machine limits".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.center_smart();
                        };
                        ui.end_row();

                        // Add free scale here
                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/free_scale.png"),
                            Some("Free scale around a point".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.set_command_context(CommandContext::ScaleAround(None, None));
                        }

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/rotate_right.png"),
                            Some("Free Rotate".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.set_command_context(CommandContext::Rotate(None, None, None));
                        }

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/move_geo.png"),
                            Some("Free Translate".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.set_command_context(CommandContext::Translate(None));
                        }

                        ui.end_row();
                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/scale.png"),
                            Some("Scale by a factor".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.set_command_context(CommandContext::Scale(1.));
                        }

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/expand.png"),
                            Some("Scale to fit paper/machine with matting".into()),
                            model.source_image_extents().is_some(),
                        )
                        .clicked()
                        {
                            model.set_command_context(CommandContext::Scale(1.));
                        }
                        ui.end_row();
                    });
                ui.add_space(16.);
                ui.label("Ruler Origin");
                let mut ro = model.ruler_origin();
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = vec2(0., 4.);
                    ui.style_mut().spacing.button_padding = vec2(6., 2.);
                    ui.style_mut().visuals.button_frame = true;
                    ui.style_mut().visuals.menu_corner_radius = egui::CornerRadius::same(16);
                    ui.style_mut().visuals.window_corner_radius = egui::CornerRadius::same(16);
                    ui.style_mut().visuals.override_text_color = Some(match &ro {
                        RulerOrigin::Origin => ui.style().visuals.strong_text_color(),
                        RulerOrigin::Source => ui.style().visuals.weak_text_color(),
                    });
                    let origin_rulers_button = Button::new("Origin")
                        .corner_radius(CornerRadius {
                            nw: 8,
                            ne: 0,
                            sw: 8,
                            se: 0,
                        })
                        .stroke(Stroke::new(
                            0.,
                            match &ro {
                                RulerOrigin::Origin => ui.style().visuals.text_color(),
                                RulerOrigin::Source => ui.style().visuals.weak_text_color(),
                            },
                        ))
                        .fill(match &ro {
                            RulerOrigin::Origin => ui.style().visuals.extreme_bg_color,
                            RulerOrigin::Source => ui.style().visuals.faint_bg_color,
                        })
                        .frame(true);
                    if ui.add(origin_rulers_button).clicked() {
                        ro = RulerOrigin::Origin;
                        model.set_ruler_origin(&ro);
                    };

                    ui.style_mut().visuals.override_text_color = Some(match &ro {
                        RulerOrigin::Source => ui.style().visuals.strong_text_color(),
                        RulerOrigin::Origin => ui.style().visuals.weak_text_color(),
                    });
                    let source_rulers_button = Button::new("Geometry")
                        .corner_radius(CornerRadius {
                            nw: 0,
                            ne: 8,
                            sw: 0,
                            se: 8,
                        })
                        .stroke(Stroke::new(
                            0.,
                            match &ro {
                                RulerOrigin::Source => ui.style().visuals.text_color(),
                                RulerOrigin::Origin => ui.style().visuals.weak_text_color(),
                            },
                        ))
                        .fill(match &ro {
                            RulerOrigin::Source => ui.style().visuals.extreme_bg_color,
                            RulerOrigin::Origin => ui.style().visuals.faint_bg_color,
                        })
                        .frame(true);
                    if ui.add(source_rulers_button).clicked() {
                        ro = RulerOrigin::Source;
                        model.set_ruler_origin(&ro);
                    };
                });
                /*
                let mut ro = model.ruler_origin();
                if ui
                    .radio_value(&mut ro, RulerOrigin::Origin, "Origin")
                    .clicked()
                {
                    model.set_ruler_origin(&ro);
                    model.update_core_config_from_changes();
                };
                if ui
                    .radio_value(&mut ro, RulerOrigin::Source, "Geometry")
                    .clicked()
                {
                    model.set_ruler_origin(&ro);
                    model.update_core_config_from_changes();
                };
                */
                ui.add_space(16.);
                ui.label("Display...");
                // egui::Grid::new("ShowHideToolz")
                //     .spacing(vec2(0., 5.))
                //     .max_col_width(24.)
                //     .num_columns(4)
                ui.horizontal(|ui| {
                    // });
                    // .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(4., 4.);
                    let mut show_paper = model.show_paper();
                    if toggle_button(
                        ui,
                        &mut show_paper,
                        egui::include_image!("../../resources/images/paper_sheets.png"),
                        Some("Show paper".to_string()),
                        true,
                    )
                    .clicked()
                    {
                        model.set_show_paper(show_paper);
                        model.update_core_config_from_changes();
                    };

                    // Limits
                    let mut show_machine_limits = model.show_machine_limits();
                    if toggle_button(
                        ui,
                        &mut show_machine_limits,
                        egui::include_image!("../../resources/images/machine_outline.png"),
                        Some("Show machine limits".to_string()),
                        true,
                    )
                    .clicked()
                    {
                        model.set_show_machine_limits(show_machine_limits);
                        model.update_core_config_from_changes();
                    };

                    // Extents
                    let mut show_extents = model.show_extents();
                    if toggle_button(
                        ui,
                        &mut show_extents,
                        egui::include_image!("../../resources/images/extents_outline.png"),
                        Some("Show extents".to_string()),
                        true,
                    )
                    .clicked()
                    {
                        model.set_show_extents(show_extents);
                        model.update_core_config_from_changes();
                    };

                    let mut show_rulers = model.show_rulers();
                    if toggle_button(
                        ui,
                        &mut show_rulers,
                        egui::include_image!("../../resources/images/ruler.png"),
                        Some("Show rulers".to_string()),
                        true,
                    )
                    .clicked()
                    {
                        model.set_show_rulers(show_rulers);
                        model.update_core_config_from_changes();
                    };
                });

                ui.horizontal(|ui| {
                    let mut show_layers = model.show_layers();
                    if toggle_button(
                        ui,
                        &mut show_layers,
                        egui::include_image!("../../resources/images/layers.png"),
                        Some("Show layers".to_string()),
                        true,
                    )
                    .clicked()
                    {
                        model.set_show_layers(show_layers);
                        model.set_inhibit_space_command(false); // Weird this isn't triggered magically, but... /shrug?
                        model.update_core_config_from_changes();
                    };
                });

                ui.add_space(16.);
                ui.label("Pen Palette");
                /* */
                // egui::ScrollArea::vertical()
                //     .auto_shrink(Vec2b::new(false, true))
                //     .max_width(150.)
                //     .show(ui, |ui| {
                for chunk in model.pen_crib().chunks(4) {
                    ui.horizontal(|ui| {
                        for pen in chunk {
                            let color_code = pen.color.clone();
                            let [r, g, b, a] = color_code.to_rgba8();
                            let color = Color32::from_rgba_premultiplied(r, g, b, a);
                            let color_selection_n = egui::Button::new(
                                RichText::new(format!("{}", pen.tool_id)).size(8.),
                            )
                            .fill(color)
                            .min_size(vec2(20., 16.));
                            if ui.add(color_selection_n).clicked() {
                                // model.c
                                model.apply_color_to_selection(pen.identity);
                            }
                        }
                    });
                }
                // });
                //
                // });
            } else
            /* if tool mode is plot mode */
            {
                ui.add_space(8.);
                // The 'serial' connection selector.
                // let mut plotter = "/dev/acm0";
                // let plotters = vec!["/dev/acm0", "magic-phaery-dust"];
                let last_port = model.current_port();
                let mut tmp_port = last_port.clone();
                ui.horizontal(|ui| {
                    let cb_resp = ComboBox::from_id_salt("Plotter Connection")
                        .selected_text(format!(
                            "{}",
                            model.current_port().replace("serial:///dev/", "")
                        ))
                        .width(72.)
                        .truncate()
                        .show_ui(ui, |ui| {
                            for plt in model.serial_ports().iter() {
                                if ui
                                    .selectable_value(
                                        &mut tmp_port,
                                        plt.clone(),
                                        format!("{}", plt.replace("serial:///dev/", "")),
                                    )
                                    .clicked()
                                {
                                    match model.plotter_state() {
                                        PlotterState::Disconnected => {
                                            model.set_current_port(tmp_port.clone());
                                        }
                                        _ => {
                                            model.set_current_port(last_port.clone());
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
                        //println!("Got a change on serial selector.");
                        match model.plotter_state() {
                            PlotterState::Disconnected => model.set_current_port(last_port.clone()),
                            _ => model.set_current_port(tmp_port),
                        }
                    }
                    if ui
                        .button(match model.plotter_state() {
                            PlotterState::Disconnected => {
                                egui::include_image!("../../resources/images/plotter_connect.png")
                            }
                            PlotterState::Failed(_) => {
                                egui::include_image!("../../resources/images/plotter_connect.png")
                            }
                            _ => egui::include_image!(
                                "../../resources/images/plotter_disconnect.png"
                            ),
                        })
                        .clicked()
                    {
                        match model.plotter_state() {
                            PlotterState::Disconnected => model.set_serial(&model.current_port()),
                            PlotterState::Dead => model.set_serial(&model.current_port()),
                            PlotterState::Running(_, _, _) => {
                                toasts.add(Toast {
                                    kind: ToastKind::Error,
                                    text: format!(
                                        "Cannot close connection while plotter is running."
                                    )
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
                            match model.plotter_state() {
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
                            match model.plotter_state() {
                                PlotterState::Ready => true,
                                PlotterState::Paused(_, _, _) => true,
                                _ => false,
                            },
                        )
                        .clicked()
                        {
                            model.request_relative_move(vec2(0., model.move_increment()));
                        };
                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/cancel.png"),
                            Some("Cancel".into()),
                            match model.plotter_state() {
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
                            match model.plotter_state() {
                                PlotterState::Ready => true,
                                PlotterState::Paused(_, _, _) => true,
                                _ => false,
                            },
                        )
                        .clicked()
                        {
                            model.request_relative_move(vec2(-model.move_increment(), 0.));
                        };

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/home.png"),
                            Some("Go Home (G28 X0 Y0)".into()),
                            match model.plotter_state() {
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
                            match model.plotter_state() {
                                PlotterState::Ready => true,
                                PlotterState::Paused(_, _, _) => true,
                                _ => false,
                            },
                        )
                        .clicked()
                        {
                            model.request_relative_move(vec2(model.move_increment(), 0.));
                        }
                        ui.end_row();

                        if tool_button(
                            ui,
                            egui::include_image!("../../resources/images/pen_down.png"),
                            Some("Pen down (on paper)".into()),
                            match model.plotter_state() {
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
                            match model.plotter_state() {
                                PlotterState::Ready => true,
                                PlotterState::Paused(_, _, _) => true,
                                _ => false,
                            },
                        )
                        .clicked()
                        {
                            model.request_relative_move(vec2(0., -model.move_increment()));
                        }

                        if tool_button(
                            ui,
                            if let PlotterState::Running(_, _, _) = model.plotter_state() {
                                egui::include_image!("../../resources/images/pause_circle.png")
                            } else {
                                egui::include_image!("../../resources/images/play_circle.png")
                            },
                            Some("Start/Resume plotting".into()),
                            match model.plotter_state() {
                                PlotterState::Ready => true,
                                PlotterState::Paused(_, _, _) => true,
                                PlotterState::Running(_, _, _) => true,
                                PlotterState::Failed(_) => true,
                                _ => false,
                            },
                        )
                        .clicked()
                        {
                            match model.plotter_state() {
                                PlotterState::Running(_, _, _) => model.plot_pause(),
                                PlotterState::Paused(_, _, _) => model.plot_start(),
                                PlotterState::Ready => model.plot_start(),
                                _ => (),
                            }
                        };
                        ui.end_row();
                        egui::Grid::new("Misc GCODE toolz")
                            .spacing(vec2(0., 5.))
                            .show(ui, |ui| {
                                if tool_button(
                                    ui,
                                    egui::include_image!("../../resources/images/gcode.png"),
                                    Some("Edit GCode".into()),
                                    model.gcode().len() > 0,
                                )
                                .clicked()
                                {
                                    model.set_command_context(CommandContext::EditGcode(Some(
                                        model.gcode().clone(),
                                    )));
                                }
                            });
                    });
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    let mut move_increment = model.move_increment();
                    ui.style_mut().spacing.slider_width = 48.;
                    if ui
                        .add(
                            Slider::new(&mut move_increment, 0.1..=100.0)
                                .suffix("mm")
                                .logarithmic(true)
                                .fixed_decimals(1),
                        )
                        .changed()
                    {
                        model.set_move_increment(move_increment);
                    };
                });
                ui.add_space(8.);
                let mut edit_cmd = model.edit_cmd();
                ui.horizontal(|ui| {
                    let cmd_response = ui.add_enabled(
                        match model.plotter_state() {
                            PlotterState::Ready => true,
                            PlotterState::Paused(_, _, _) => true,
                            _ => false,
                        },
                        TextEdit::singleline(&mut edit_cmd)
                            .min_size(vec2(72., 16.))
                            .desired_width(72.),
                    );
                    let mut viz_cue = false;
                    if cmd_response.changed() {
                        model.set_edit_cmd(edit_cmd.clone());
                    }
                    if cmd_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        model.send_command(&edit_cmd);
                        viz_cue = true;
                    }
                    let mut but_resp =
                        ui.button(egui::include_image!("../../resources/images/send_cmd.png"));
                    if viz_cue {
                        but_resp = but_resp.highlight();
                    }

                    if but_resp.clicked() {
                        match model.plotter_state() {
                            PlotterState::Ready => model.send_command(&edit_cmd),
                            PlotterState::Paused(_, _, _) => model.send_command(&edit_cmd),
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
        if model.toolbar_position() == DockPosition::Left
            || model.toolbar_position() == DockPosition::Right
        {
            let mut avail = ui.cursor();
            let width = ui.min_rect().width();
            avail.set_height(ui.available_height() - wtop - 90.);
            avail.set_width(width);
            ui.allocate_rect(avail, Sense::empty());
        };
    });
    if let Some(response) = win_response {
        model.set_toolbar_width(response.response.rect.width());
        if response.response.drag_stopped() {
            // println!("DRAG STOP.");
            if let DockPosition::Floating(_x, _y) = model.toolbar_position() {
                let Pos2 { x, y } = response.response.rect.min.clone();
                model.set_toolbar_position(&DockPosition::Floating(x, y));
                model.update_core_config_from_changes();
            }
        }
    } else {
    }
}
