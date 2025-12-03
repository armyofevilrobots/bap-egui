use crate::ui::machine::machine_editor_window;
// use crate::ui::bottom_panel::bottom_panel;
use crate::ui::menu::main_menu;
use crate::ui::paper_chooser::paper_chooser_window;
use crate::ui::pen_crib::pen_crib_window;
use crate::ui::pen_delete::pen_delete_window;
use crate::ui::themes::theme_window;
use crate::view_model::command_context::SpaceCommandStatus;
use crate::view_model::{BAPViewModel, CommandContext};
use eframe::egui;
use egui::Direction::BottomUp;
use egui::{Align2, Color32, FontId, Frame, Key, Rect, Stroke, StrokeKind, pos2, vec2};
use egui_toast::Toasts;

pub(crate) mod bottom_panel;
pub(crate) mod machine;
pub(crate) mod menu;
pub(crate) mod paper_chooser;
pub(crate) mod pen_crib;
pub(crate) mod pen_delete;
pub(crate) mod pen_editor;
pub(crate) mod rulers;
pub(crate) mod scale_window;
pub(crate) mod scene_toggle;
pub(crate) mod space_command_palette;
pub(crate) mod themes;
pub(crate) mod tool_button;
pub(crate) mod tool_window;
use tool_window::floating_tool_window;

pub(crate) fn update_ui(model: &mut BAPViewModel, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Looks better on 4k montior
    ctx.set_pixels_per_point(model.ppp());
    model.set_modifiers(&ctx.input(|i| i.modifiers));

    model.check_for_new_source_image();

    let tbp = main_menu(model, ctx);
    scene_toggle::scene_toggle(model, ctx);
    let mut toasts = Toasts::new()
        .anchor(Align2::RIGHT_BOTTOM, (-10.0, -25.0)) // 10 units from the bottom right corner
        .direction(BottomUp);

    let wtop = tbp.top();
    floating_tool_window(model, ctx, wtop, &mut toasts);

    match &model.command_context() {
        CommandContext::PaperChooser => paper_chooser_window(model, ctx),
        CommandContext::PenCrib => pen_crib_window(model, ctx),
        CommandContext::PenEdit(pen_idx, _pen) => {
            pen_editor::pen_editor_window(model, ctx, *pen_idx)
        }
        CommandContext::Scale(_factor) => scale_window::scale_window(model, ctx),
        CommandContext::PenDelete(pen_idx) => pen_delete_window(model, ctx, *pen_idx),
        CommandContext::MachineEdit(_opt_machine) => machine_editor_window(model, ctx),
        CommandContext::SelectTheme => theme_window(model, ctx),
        CommandContext::Space(keys) => {
            let keys = keys.clone();
            match CommandContext::dispatch_space_cmd(model, &keys) {
                SpaceCommandStatus::Dispatched(dispatched) => {
                    // Special case handling for when we trigger a new command context
                    // if let CommandContext::Space(_) = model.command_context {
                    //     model.command_context = CommandContext::None;
                    // }
                    if let CommandContext::Space(_) = model.command_context() {
                        model.cancel_command_context(false);
                    } // Otherwise, we changed contexts in the command itself.
                    model.toast_info(dispatched);
                }
                SpaceCommandStatus::Ongoing => (),
                SpaceCommandStatus::Invalid => {
                    // model.command_context = CommandContext::None;
                    model.cancel_command_context(false);
                    let msg = format!(
                        "Invalid key sequence - {}",
                        keys.clone()
                            .iter()
                            .map(|&k| k.symbol_or_name())
                            .collect::<Vec<&str>>()
                            .join("-")
                            .to_string()
                    );
                    model.toast_error(msg);
                }
            }
        }

        _ => (),
    }

    let _cp = egui::CentralPanel::default().frame(Frame::new().fill(ctx.style().visuals.window_fill.clone()).stroke(Stroke::NONE)).show(ctx, |ui| {
        // ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

        let precursor = ui.cursor();
        // let painter = ui.painter();
        let (painter_resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::all());
        let painter_resp = painter_resp.on_hover_cursor(match model.command_context() {
            CommandContext::Origin => egui::CursorIcon::Crosshair,
            _ => egui::CursorIcon::Default,
        });

        model.set_container_rect(painter_resp.rect.clone());

        let (min, max) = (painter_resp.rect.min, painter_resp.rect.max);
        model.set_center_coords(pos2((min.x + max.x) / 2.0_f32, (min.y + max.y) / 2.0_f32));

        // // Draw the paper
        if model.show_paper() {
            let paper_rect = model.mm_rect_to_screen_rect(model.get_paper_rect());
            painter.rect(
                paper_rect,
                0.,
                model.paper_color(),
                Stroke::NONE,
                egui::StrokeKind::Outside,
            );
        }
        if let Some(imghandle) = model.source_image_handle() {
            if let Some(svgrect) = model.source_image_extents() {
                let rect = Rect::from_min_size(
                    model.mm_to_frame_coords(svgrect.min),
                    model.scale_mm_to_screen(svgrect.size()),
                );
                painter.image(
                    imghandle.id(),
                    rect,
                    // Rect::from_center_size(center, size),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
        // The translation line
        if let CommandContext::ScaleAround(start, opt_ref) = model.command_context() {
            let p1 = painter_resp.rect.min.clone();
            let p2 = painter_resp.rect.max.clone();
            if let Some(ptr_pos) = ctx.pointer_latest_pos() {
                if let Some(start_xy) = start {
                    let pos = model.mm_to_frame_coords(start_xy);
                    painter.line(
                        vec![pos2(pos.x - 8., pos.y), pos2(pos.x + 8., pos.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.line(
                        vec![pos2(pos.x, pos.y - 8.), pos2(pos.x, pos.y + 8.)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.circle(
                        pos,
                        8.,
                        Color32::TRANSPARENT,
                        Stroke::new(0.5, Color32::RED),
                    );
                    let rad_ref1 = (ptr_pos - pos).length();
                    if rad_ref1 > 0. {
                        painter.circle(
                            pos,
                            rad_ref1,
                            Color32::TRANSPARENT,
                            Stroke::new(0.5, Color32::RED),
                        );
                    }
                    if let Some(ref_xy) = opt_ref {
                        let ref1_vec_mm = ref_xy - start_xy;
                        let rad_ref1_mm = ref1_vec_mm.length();
                        if rad_ref1_mm.abs() > 0.001 {
                            let rad_base_px = model.scale_mm_to_screen(ref1_vec_mm).length();
                            let ref2_xy = model.frame_coords_to_mm(ptr_pos);
                            let ref2_vec_mm = ref2_xy - start_xy;

                            if ref2_vec_mm.length().abs()>0.001{
                                model.request_new_source_image();
                                painter.text(
                                    model.mm_to_frame_coords(start_xy+vec2(8., 8.)),
                                    Align2::LEFT_BOTTOM,
                                    format!("{:3.1}%", 100.*(ref2_vec_mm.length()/rad_ref1_mm)),
                                    FontId::proportional(8.),
                                    Color32::RED,
                                );

                            }
                            painter.circle(
                                pos,
                                rad_base_px,
                                Color32::TRANSPARENT,
                                Stroke::new(0.5, Color32::RED),
                            );
                        }
                    }
                } else {
                    painter.line(
                        vec![pos2(ptr_pos.x, p1.y), pos2(ptr_pos.x, p2.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.line(
                        vec![pos2(p1.x, ptr_pos.y), pos2(p2.x, ptr_pos.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                }
            }
        };
        // The translation line
        if let CommandContext::Translate(start) = model.command_context() {
            let p1 = painter_resp.rect.min.clone();
            let p2 = painter_resp.rect.max.clone();
            if let Some(ptr_pos) = ctx.pointer_latest_pos() {
                if let Some(start_xy) = start {
                    let pos = model.mm_to_frame_coords(start_xy);
                    painter.line(
                        vec![pos2(pos.x, p1.y), pos2(pos.x, p2.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.line(
                        vec![pos2(p1.x, pos.y), pos2(p2.x, pos.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.arrow(
                        pos.clone(),
                        ptr_pos.clone() - pos.clone(),
                        Stroke::new(0.5, Color32::RED),
                    );
                } else {
                    painter.line(
                        vec![pos2(ptr_pos.x, p1.y), pos2(ptr_pos.x, p2.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.line(
                        vec![pos2(p1.x, ptr_pos.y), pos2(p2.x, ptr_pos.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                }
            }
        }

        // Draw these lines _last_ so they overlap the drawing itself.
        if model.command_context() == CommandContext::Origin {
            if let Some(pos) = ctx.pointer_latest_pos() {
                let p1 = painter_resp.rect.min.clone();
                let p2 = painter_resp.rect.max.clone();
                painter.line(
                    vec![pos2(pos.x, p1.y), pos2(pos.x, p2.y)],
                    Stroke::new(0.5, Color32::RED),
                );
                painter.line(
                    vec![pos2(p1.x, pos.y), pos2(p2.x, pos.y)],
                    Stroke::new(0.5, Color32::RED),
                );
                let tmp_origin = model.frame_coords_to_mm(pos);
                let paper_tmp_rect =
                    model.mm_rect_to_screen_rect(model.calc_paper_rect(tmp_origin));

                if model.show_paper() {
                    painter.rect(
                        paper_tmp_rect,
                        0.,
                        model.paper_color().gamma_multiply(0.5),
                        Stroke::new(2., Color32::from_gray(128)),
                        StrokeKind::Middle,
                    );
                }

                // Also a temporary machine bounds to make that more obvious...
                let machine_rect = model.mm_rect_to_screen_rect(Rect::from_min_max(
                    pos2(
                        tmp_origin.x,
                        tmp_origin.y - model.machine_config().limits().1 as f32,
                    ),
                    pos2(
                        tmp_origin.x + model.machine_config().limits().0 as f32,
                        tmp_origin.y,
                    ),
                ));
                if model.show_machine_limits() {
                    painter.rect(
                        machine_rect,
                        0.,
                        Color32::TRANSPARENT,
                        Stroke::new(1., Color32::YELLOW),
                        StrokeKind::Outside,
                    );
                }
            };
        }

        // The rotation tool.
        if let CommandContext::Rotate(center, ref1, _ref2) = model.command_context() {
            if let Some(pos) = ctx.pointer_latest_pos() {
                let p1 = painter_resp.rect.min.clone();
                let p2 = painter_resp.rect.max.clone();
                if let Some(center_pos) = center {
                    // Center is set.
                    let center_pos_frame = model.mm_to_frame_coords(center_pos);
                    painter.line(
                        vec![
                            pos2(center_pos_frame.x, p1.y),
                            pos2(center_pos_frame.x, p2.y),
                        ],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.line(
                        vec![
                            pos2(p1.x, center_pos_frame.y),
                            pos2(p2.x, center_pos_frame.y),
                        ],
                        Stroke::new(0.5, Color32::RED),
                    );
                    let center_as_frame = model.mm_to_frame_coords(center_pos);

                    // Then we draw the live arc and second ref, if available...
                    if let Some(ref1_mm) = ref1 {
                        // println!("We have an initial ref... Request new image?");
                        let ref_circle_rad =
                            (center_as_frame - model.mm_to_frame_coords(ref1_mm)).length();
                        let ref2_vec = (pos - model.mm_to_frame_coords(center_pos)).normalized()
                            * ref_circle_rad;
                        let vec_ref1 = ref1_mm-center_pos;
                        let vec_ref2 = model.frame_coords_to_mm(pos)-center_pos;
                        let mut angle = BAPViewModel::degrees_between_two_vecs(vec_ref1, vec_ref2);

                        let mods = ctx.input(|i| i.modifiers.clone());
                        if mods.shift{
                            angle = (angle/5.0).round()*5.0;
                        }
                        painter.line_segment(
                            [center_as_frame.clone(), center_as_frame + ref2_vec],
                            Stroke::new(0.5, Color32::RED),
                        );
                        painter.text(
                            center_as_frame+vec2(2., -8.),
                            Align2::LEFT_TOP,
                            format!("Rotate {:3.1}°", angle),
                            FontId::proportional(8.),
                            Color32::RED,
                        );
                        model.request_new_source_image();

                    } else {
                        painter.circle(
                            center_as_frame.clone(),
                            (center_as_frame - pos).length(),
                            Color32::TRANSPARENT,
                            Stroke::new(0.5, Color32::RED),
                        );
                    }

                    // Draw the ref1 angle line.
                    if let Some(ref_pos) = ref1 {
                        painter.line_segment(
                            [
                                model.mm_to_frame_coords(center_pos),
                                model.mm_to_frame_coords(ref_pos),
                            ],
                            Stroke::new(0.5, Color32::RED),
                        );
                    } else {
                        painter.line_segment(
                            [model.mm_to_frame_coords(center_pos), pos],
                            Stroke::new(0.5, Color32::RED),
                        );
                    }
                } else {
                    // No center is set
                    painter.line(
                        vec![pos2(pos.x, p1.y), pos2(pos.x, p2.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    painter.line(
                        vec![pos2(p1.x, pos.y), pos2(p2.x, pos.y)],
                        Stroke::new(0.5, Color32::RED),
                    );
                    if ref1.is_none() {
                        painter.circle(
                            pos,
                            32.,
                            Color32::TRANSPARENT,
                            Stroke::new(0.5, Color32::RED),
                        );
                    }
                };
            }; // Only do stuff if we're actually in the window.
        } // End rotate display context.

        let machine_rect = model.mm_rect_to_screen_rect(Rect::from_min_max(
            pos2(
                model.origin().x,
                model.origin().y - model.machine_config().limits().1 as f32,
            ),
            pos2(
                model.origin().x + model.machine_config().limits().0 as f32,
                model.origin().y,
            ),
        ));
        if model.show_machine_limits() {
            painter.rect(
                machine_rect,
                0.,
                Color32::TRANSPARENT,
                Stroke::new(1., Color32::YELLOW),
                StrokeKind::Outside,
            );
        }
        if model.show_extents() {
            if let Some(extents) = model.source_image_extents() {
                let extents_rect = model.mm_rect_to_screen_rect(extents);
                // if model.show_extents {
                painter.rect(
                    extents_rect,
                    0.,
                    Color32::TRANSPARENT,
                    Stroke::new(1., Color32::BLUE),
                    StrokeKind::Outside,
                );
            }
            // }
        }

        // The ruler display bit.
        if model.show_rulers() {
            rulers::draw_rulers(model, &ui, ctx, &painter, &painter_resp)
        };

        // The rotation thing.

        if painter_resp.clicked() {
            match model.command_context() {
                CommandContext::Origin => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        // model.origin = model.frame_coords_to_mm(pos)
                        model.set_origin(model.frame_coords_to_mm(pos), true);
                        // model.command_context = CommandContext::None;
                        model.cancel_command_context(false);
                    }
                }
                CommandContext::Clip(_pos2, _pos3) => todo!(),
                CommandContext::Rotate(None, None, None) => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        model.set_command_context(CommandContext::Rotate(
                            Some(model.frame_coords_to_mm(pos)),
                            None,
                            None,
                        ));
                    }
                }
                CommandContext::Rotate(Some(center_mm), None, None) => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        model.set_command_context(CommandContext::Rotate(
                            Some(center_mm),
                            Some(model.frame_coords_to_mm(pos)),
                            None,
                        ));
                    }
                }
                CommandContext::Rotate(Some(center_mm), Some(ref1_mm), None) => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        let ref2_mm = model.frame_coords_to_mm(pos);
                        // model.command_context =
                        //     CommandContext::Rotate(Some(center_mm), Some(ref1_mm), Some(ref2_mm));
                        let vec_a = ref1_mm - center_mm;
                        let vec_b = ref2_mm - center_mm;
                        let mut degrees = BAPViewModel::degrees_between_two_vecs(vec_a, vec_b);
                        let mods = ctx.input(|i| i.modifiers.clone());
                        if mods.shift{
                            degrees = (degrees/5.0).round()*5.0;
                            // let vec_ref2 = vec2(vec_ref1*angle.cos)

                        }
                        // println!("Calculated angle is {}", degrees);
                        model.rotate_around_point(
                            (center_mm.x as f64, center_mm.y as f64),
                            degrees as f64,
                        );
                        // model.command_context = CommandContext::None;
                        model.cancel_command_context(false);
                    }
                }
                CommandContext::ScaleAround(opt_pos, opt_ref) => {
                    if let Some(hover_pos) = ctx.pointer_hover_pos() {
                        if opt_pos.is_none() {
                            let opt_pos = Some(model.frame_coords_to_mm(hover_pos));
                            model.set_command_context(CommandContext::ScaleAround(opt_pos, None));
                        }
                        if let Some(pos) = opt_pos
                            && opt_ref.is_none()
                        {
                            let ref_pos = model.frame_coords_to_mm(hover_pos);
                            if (ref_pos - pos).length().abs() > 0.01 {
                                model.set_command_context(CommandContext::ScaleAround(
                                    opt_pos,
                                    Some(ref_pos),
                                ));
                            }
                        } else if let Some(center) = opt_pos
                            && let Some(ref1) = opt_ref
                        {
                            let pos2 = model.frame_coords_to_mm(hover_pos);
                            let rad2 = (pos2 - center).length();
                            let rad1 = (ref1 - center).length();
                            if rad2 != 0. && rad1 != 0. {
                                let ratio = rad2 / rad1;
                                model.scale_around(center, ratio);
                                model.cancel_command_context(false);
                            } else {
                                eprintln!("Cannot scale with a reference position or scale position of 0.0");
                            }
                            // model.apply_translation(delta.x as f64, delta.y as f64);
                            // model.cancel_command_context(false);
                            ()
                        }
                    }
                }
                CommandContext::Space(_) => (),
                CommandContext::SelectColorAt(_) => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        model.select_by_color_pick(pos);
                        model.cancel_command_context(false);
                    }
                }
                CommandContext::Translate(opt_pos) => {
                    if let Some(hover_pos) = ctx.pointer_hover_pos() {
                        if opt_pos.is_none() {
                            let opt_pos = Some(model.frame_coords_to_mm(hover_pos));
                            model.set_command_context(CommandContext::Translate(opt_pos));
                        } else {
                            let pos1 = opt_pos.unwrap();
                            let pos2 = model.frame_coords_to_mm(hover_pos);
                            let delta = pos2 - pos1;
                            model.apply_translation(delta.x as f64, delta.y as f64);
                            model.cancel_command_context(false);
                        }
                    }
                }
                _ => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        // println!("Clicked at {:?}", model.frame_coords_to_mm(pos));
                        let mods = ctx.input(|i| i.modifiers.clone());

                        if !mods.shift && !mods.ctrl {
                            model.pick_clear();
                        }
                        if ctx.input(|i| i.modifiers.clone()).shift {
                            model.add_pick_at_point(model.frame_coords_to_mm(pos));
                        } else if ctx.input(|i| i.modifiers.clone()).ctrl {
                            model.toggle_pick_at_point(model.frame_coords_to_mm(pos));
                        } else {
                            model.pick_at_point(model.frame_coords_to_mm(pos));
                        }
                    }
                    model.cancel_command_context(true);
                }
            }
        }

        if painter_resp.dragged() {
            model
                .set_look_at(model.look_at() - model.scale_screen_to_mm(painter_resp.drag_delta()));
        }

        if painter_resp.contains_pointer() {
            // let delta =
            let mouse_pos_screen = if let Some(pos) = ctx.pointer_interact_pos() {
                Some(pos /*  - model.center_coords.to_vec2()*/)
            } else {
                None
            };
            ui.input(|i| {
                i.events.iter().for_each(|e| match e {
                    egui::Event::MouseWheel {
                        unit: _,
                        delta,
                        modifiers: _modifiers,
                    } => {
                        // Some(*delta)
                        if let Some(mouse_pos) = mouse_pos_screen {
                            let mouse_pos_pre_mm = model.frame_coords_to_mm(mouse_pos.clone());
                            if delta.y > 0. {
                                model.set_zoom(model.zoom() * 1.1 * delta.y.abs() as f64);
                            } else {
                                model.set_zoom(model.zoom() * (1.0 / 1.1) * delta.y.abs() as f64);
                            }
                            let mouse_pos_post_mm = model.frame_coords_to_mm(mouse_pos.clone());
                            let delta = mouse_pos_pre_mm - mouse_pos_post_mm;
                            // let drag = model.scale_mm_to_screen(delta);
                            model.set_look_at(model.look_at() + delta);
                        }
                    }
                    _ => (),
                });
            });
        }

        ui.input(|i| {
            i.events.iter().for_each(|e| match e {
                egui::Event::Key {
                    key: _,
                    physical_key,
                    pressed,
                    repeat: _,
                    modifiers: mods,
                } => {
                    if let Some(pkey) = physical_key {
                        if *pkey == Key::Z && *pressed && mods.ctrl{
                            if model.undo_available(){
                                model.undo();
                            }
                        }
                        if *pkey == Key::Escape && *pressed {
                            // Only on depress, not release
                            if model.command_context() != CommandContext::None {
                                // model.command_context = CommandContext::None;
                                model.cancel_command_context(true);
                            }
                        } else if *pkey == Key::Space && *pressed {
                            // println!("SPACE MODE");
                            if model.command_context() == CommandContext::None {
                                model.set_command_context(CommandContext::Space(vec![]));
                            }
                        } else if *pressed
                            && let CommandContext::Space(keys) = model.command_context_mut()
                        {
                            if *pkey == Key::Backspace || *pkey == Key::Delete {
                                keys.pop();
                            } else {
                                keys.push(pkey.clone());
                            }
                        } else if *pressed && *pkey == Key::Delete {
                            model.delete_selection();
                        }
                    };
                    // None
                }
                _ => (),
            });
        });

        space_command_palette::space_command_panel(model, ctx);

        bottom_panel::bottom_panel(model, ctx);
        while let Some(toast) = model.next_toast() {
            toasts.add(toast);
        }
        toasts.show(ctx);
        (precursor, ui.cursor())
    });
}
