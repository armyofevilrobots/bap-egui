// use crate::ui::bottom_panel::bottom_panel;
use crate::ui::menu::main_menu;
use crate::ui::paper_chooser::paper_chooser_window;
use crate::ui::pen_crib::pen_crib_window;
use crate::view_model::{BAPViewModel, CommandContext};
use eframe::egui;
use egui::Direction::BottomUp;
use egui::{Align2, Color32, Key, Rect, Stroke, StrokeKind, pos2};

pub(crate) mod tool_window;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use tool_window::floating_tool_window;
pub(crate) mod bottom_panel;
pub(crate) mod menu;
pub(crate) mod paper_chooser;
pub(crate) mod pen_crib;
pub(crate) mod scene_toggle;
pub(crate) mod themes;
pub(crate) mod tool_button;

// pub(crate) fn native_to_mm(native: Pos2, zoom: f32) -> Pos2 {
//     (PIXELS_PER_MM * native) / zoom
// }

// pub(crate) fn mm_to_native(mm: Pos2, zoom: f32) -> Pos2 {
//     (mm * zoom) / PIXELS_PER_MM
// }

pub(crate) fn update_ui(model: &mut BAPViewModel, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Looks better on 4k montior
    ctx.set_pixels_per_point(model.ppp());

    model.check_for_new_source_image();

    let tbp = main_menu(model, ctx);
    scene_toggle::scene_toggle(model, ctx);
    let mut toasts = Toasts::new()
        .anchor(Align2::RIGHT_BOTTOM, (-10.0, -25.0)) // 10 units from the bottom right corner
        .direction(BottomUp);

    let wtop = tbp.top();
    floating_tool_window(model, ctx, wtop, &mut toasts);
    if model.paper_modal_open {
        paper_chooser_window(model, ctx);
    }
    if model.pen_crib_open {
        pen_crib_window(model, ctx);
    }

    let _cp = egui::CentralPanel::default().show(ctx, |ui| {
        // ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

        let precursor = ui.cursor();
        // let painter = ui.painter();
        let (painter_resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::all());

        model.container_rect = Some(painter_resp.rect.clone());

        let (min, max) = (painter_resp.rect.min, painter_resp.rect.max);
        model.center_coords = pos2((min.x + max.x) / 2.0_f32, (min.y + max.y) / 2.0_f32);

        // // Draw the paper
        if model.show_paper {
            let paper_rect = model.mm_rect_to_screen_rect(model.get_paper_rect());
            painter.rect(
                paper_rect,
                0.,
                model.paper_color,
                Stroke::NONE,
                egui::StrokeKind::Outside,
            );
        }
        if let Some(imghandle) = &model.source_image_handle {
            // let size_raw = imghandle.size_vec2();
            // let size = size_raw * model.view_zoom as f32 / PIXELS_PER_MM;
            // let center = mm_to_native(mm, zoom)
            // let svgrect = model.svg_img_dims.expect(
            //     "Somehow we have an image handle with no dims.
            //         This should be impossible. Dying.",
            // );
            if let Some(svgrect) = model.source_image_extents {
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

        // Draw these lines _last_ so they overlap the drawing itself.
        if model.command_context == CommandContext::Origin {
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

                if model.show_paper {
                    painter.rect(
                        paper_tmp_rect,
                        0.,
                        model.paper_color.gamma_multiply(0.5),
                        Stroke::new(2., Color32::from_gray(128)),
                        StrokeKind::Middle,
                    );
                }

                // Also a temporary machine bounds to make that more obvious...
                let machine_rect = model.mm_rect_to_screen_rect(Rect::from_min_max(
                    pos2(
                        tmp_origin.x,
                        tmp_origin.y - model.machine_config.limits().1 as f32,
                    ),
                    pos2(
                        tmp_origin.x + model.machine_config.limits().0 as f32,
                        tmp_origin.y,
                    ),
                ));
                if model.show_machine_limits {
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

        {
            let machine_rect = model.mm_rect_to_screen_rect(Rect::from_min_max(
                pos2(
                    model.origin.x,
                    model.origin.y - model.machine_config.limits().1 as f32,
                ),
                pos2(
                    model.origin.x + model.machine_config.limits().0 as f32,
                    model.origin.y,
                ),
            ));
            if model.show_machine_limits {
                painter.rect(
                    machine_rect,
                    0.,
                    Color32::TRANSPARENT,
                    Stroke::new(1., Color32::YELLOW),
                    StrokeKind::Outside,
                );
            }
            if model.show_extents {
                if let Some(extents) = model.source_image_extents {
                    let extents_rect = model.mm_rect_to_screen_rect(extents);
                    if model.show_extents {
                        painter.rect(
                            extents_rect,
                            0.,
                            Color32::TRANSPARENT,
                            Stroke::new(1., Color32::BLUE),
                            StrokeKind::Outside,
                        );
                    }
                }
            }

            // This is the ruler display
            if model.show_rulers {
                let p1 = painter_resp.rect.min;
                let p2 = painter_resp.rect.max;
                let p3 = pos2(p2.x, p1.y + 16.);
                let p4 = pos2(p1.x, p1.y + 16.);
                let p5 = pos2(p1.x + 16., p2.y);
                let color = ui.visuals().faint_bg_color.clone();
                painter.rect(
                    Rect::from_min_max(p1, p3),
                    0.,
                    color,
                    Stroke::NONE,
                    StrokeKind::Outside,
                );
                painter.rect(
                    Rect::from_min_max(p4, p5),
                    0.,
                    color,
                    Stroke::NONE,
                    StrokeKind::Outside,
                );
            }
        }

        if painter_resp.clicked() {
            match model.command_context {
                CommandContext::Origin => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        // model.origin = model.frame_coords_to_mm(pos)
                        model.set_origin(model.frame_coords_to_mm(pos));
                    }
                }
                CommandContext::None => (),
                CommandContext::Clip(_pos2, _pos3) => todo!(),
            }
            model.command_context = CommandContext::None;
        }

        if painter_resp.dragged() {
            model.look_at =
                // model.look_at - (PIXELS_PER_MM * painter_resp.drag_delta() / model.view_zoom as f32)
                model.look_at - model.scale_screen_to_mm(painter_resp.drag_delta())
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
                            model.look_at = model.look_at + delta;
                        }
                    }
                    egui::Event::Key {
                        key: _,
                        physical_key,
                        pressed: _,
                        repeat: _,
                        modifiers: _,
                    } => {
                        if let Some(pkey) = physical_key {
                            if *pkey == Key::Escape {
                                if model.command_context != CommandContext::None {
                                    toasts.add(Toast {
                                        kind: ToastKind::Info,
                                        text: format!(
                                            "Exited command context {:?}",
                                            model.command_context
                                        )
                                        .into(),
                                        options: ToastOptions::default()
                                            .duration_in_seconds(5.0)
                                            .show_progress(true),
                                        ..Default::default()
                                    });
                                    model.command_context = CommandContext::None;
                                }
                            }
                        };
                        // None
                    }
                    _ => (),
                });
            });
        }

        bottom_panel::bottom_panel(model, ctx);
        while !model.queued_toasts.is_empty() {
            if let Some(toast) = model.queued_toasts.pop_front() {
                toasts.add(toast);
            }
        }
        toasts.show(ctx);
        (precursor, ui.cursor())
    });
}
