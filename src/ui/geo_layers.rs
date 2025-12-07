use std::sync::Arc;

use crate::view_model::BAPViewModel;
use crate::{core::config::DockPosition, view_model::CommandContext};
use eframe::egui;
#[allow(unused)]
use egui::Stroke;
use egui::{Button, Color32, CornerRadius, Frame, Id, Image, Pos2, TextEdit, include_image, vec2};
#[allow(unused)]
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

#[allow(unused)]
use super::tool_button::tool_button;

pub(crate) fn floating_geo_layer_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    wtop: f32,
    _toasts: &mut Toasts,
) {
    let default_height = ctx.content_rect().height() - wtop - 22.; //23.;
    let win = egui::Window::new("Layers")
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

    let addl_offset: f32 = if model.toolbar_position() == model.geo_layer_position() {
        model.toolbar_width() + 2.0f32
    } else {
        0.0f32
    };
    let win = match model.geo_layer_position() {
        DockPosition::Floating(_x, _y) => win.title_bar(false), //.current_pos(Pos2 { x, y }),
        DockPosition::Left => {
            let ofs = if model.show_rulers() {
                (24.0 + addl_offset, wtop + 74.)
            } else {
                (2. + addl_offset, wtop + 49.)
            };
            ctx.style_mut(|style| style.visuals.window_corner_radius = CornerRadius::same(0));
            win.title_bar(false)
                .anchor(egui::Align2::LEFT_TOP, ofs)
                .default_height(default_height)
                .min_height(default_height)
                .max_height(default_height)
        }
        DockPosition::Right => {
            let ofs = if model.show_rulers() {
                (0.0 - addl_offset, wtop + 74.)
            } else {
                (0.0 - addl_offset, wtop + 49.)
            };

            ctx.style_mut(|style| style.visuals.window_corner_radius = CornerRadius::same(0));
            win.title_bar(false)
                .anchor(egui::Align2::RIGHT_TOP, ofs)
                .default_height(default_height)
                .min_height(default_height)
                .max_height(default_height)
        }
    };

    let _win_response = win.show(ctx, |ui| {
        ui.horizontal(|ui| {
            let mut docked = if let DockPosition::Floating(_x, _y) = model.geo_layer_position() {
                false
            } else {
                true
            };
            let dock_response = ui.toggle_value(&mut docked, "📌");
            model.set_geo_layer_position(&if docked {
                match model.geo_layer_position() {
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
        //
        ui.add_space(4.);
        ui.horizontal(|ui| {
            let mut toggle_pick_button =
                Button::new(include_image!("../../resources/images/invert_select.png"));
            if model.picked().is_some() {
                toggle_pick_button = toggle_pick_button
                    .selected(true)
                    .stroke(Stroke::new(1., ctx.style().visuals.strong_text_color()))
            }
            if ui.add(toggle_pick_button).clicked() {
                model.invert_pick();
            }

            if model.picked().is_some() {
                if ui
                    .add(
                        Button::new(include_image!("../../resources/images/select_none.png"))
                            .stroke(Stroke::new(1., ctx.style().visuals.strong_text_color())),
                    )
                    .clicked()
                {
                    model.pick_clear();
                }
            } else {
                if ui
                    .add(
                        Button::new(include_image!("../../resources/images/select_all.png"))
                            .stroke(Stroke::new(1., ctx.style().visuals.strong_text_color())),
                    )
                    .clicked()
                {
                    model.pick_all();
                }
            }
        });
        ui.shrink_width_to_current();
        ui.add_space(8.);
        // super::scene_toggle::scene_toggle_toolbox(model, ctx, ui);
        egui::ScrollArea::vertical()
            .max_height(default_height - ui.cursor().top() - 8.)
            .min_scrolled_height(default_height - ui.cursor().top() - 8.)
            .auto_shrink(match model.geo_layer_position() {
                DockPosition::Floating(_, _) => true,
                _ => false,
            })
            .show(ui, |ui| {
                let frame = Frame::default().inner_margin(4.0);
                let (_, _dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
                    // This is the actual window content.
                    let _drag_from: Option<Arc<usize>> = None;
                    let _drag_to: Option<Arc<usize>> = None;

                    // Grid::new("GeoLayersGrid").striped(true).show(ui, |ui| {
                    for idx in 0..model.geo_layers().len() {
                        // for (idx, layer) in model.geo_layers().iter().enumerate() {
                        // println!("Found texture:{:?} for layer {}", layer.preview, _idx);
                        let layer_drag_inner = ui.horizontal(|ui| {
                            let layer_drag_response = ui
                                .dnd_drag_source(
                                    Id::new(format!("layer-drag-source-{}", idx)),
                                    idx,
                                    |ui| {
                                        let img = egui::Image::new(include_image!(
                                            "../../resources/images/drag_indicator.png"
                                        ))
                                        .bg_fill(Color32::TRANSPARENT);
                                        let drag_handle = Box::new(Button::new(img))
                                            .fill(Color32::TRANSPARENT)
                                            .min_size(vec2(18., 40.));
                                        ui.add(drag_handle);
                                    },
                                )
                                .response;

                            let img = Image::new((
                                model.geo_layers()[idx].preview.id(),
                                model.geo_layers()[idx].preview.size_vec2(),
                            ))
                            .bg_fill(model.paper_color())
                            .corner_radius(0);
                            let mut toggle_pick_button = Button::new(img).min_size(vec2(32., 40.));
                            if let Some(picked) = model.picked() {
                                if picked.contains(&idx) {
                                    toggle_pick_button = toggle_pick_button.selected(true).stroke(
                                        Stroke::new(1., ctx.style().visuals.strong_text_color()),
                                    )
                                }
                            }
                            let tp_resp = ui.add(toggle_pick_button);
                            // let tp_resp = pen_drag_response.response;
                            if tp_resp.clicked() && model.modifiers().shift {
                                if let Some(picked) = model.picked() {
                                    if picked.len() == 1 {
                                        let first = picked.first().unwrap();
                                        if idx < *first {
                                            for add_idx in idx..*first {
                                                model.toggle_pick_by_id(add_idx);
                                            }
                                        } else if idx > *first {
                                            for add_idx in (*first + 1)..=idx {
                                                model.toggle_pick_by_id(add_idx);
                                            }
                                        } else {
                                            // SHift clicked the same one again.
                                            model.toggle_pick_by_id(idx);
                                        }
                                    } else if picked.is_empty() {
                                        model.toggle_pick_by_id(idx);
                                    }
                                }
                            } else if tp_resp.clicked() && model.modifiers().is_none() {
                                if let CommandContext::SelectColorAt(_foo) = model.command_context()
                                {
                                    model.select_layers_matching_color(
                                        model.geo_layers()[idx].pen_uuid,
                                    );
                                    model.cancel_command_context(false);
                                } else {
                                    model.toggle_pick_by_id(idx);
                                }
                            }

                            // let mut name_tmp = model.geo_layers().name.clone();
                            let name_edit =
                                TextEdit::singleline(&mut model.geo_layers_mut()[idx].name)
                                    .desired_width(128.)
                                    .min_size(vec2(128., 8.));
                            let name_edit_resp = ui.add(name_edit);
                            if name_edit_resp.lost_focus()
                                && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                model.update_layer_name(idx, model.geo_layers()[idx].name.clone());
                            };
                            if name_edit_resp.gained_focus() {
                                model.set_inhibit_space_command(true);
                            }
                            if name_edit_resp.lost_focus() {
                                model.set_inhibit_space_command(false);
                            }
                            layer_drag_response
                        }); // ui.horizontal

                        // ui.label(model.geo_layers()[idx].pen_uuid.as_urn().to_string());
                        // ui.end_row();
                        if let (Some(pointer), Some(hovered_payload)) = (
                            ui.input(|i| i.pointer.interact_pos()),
                            layer_drag_inner.response.dnd_hover_payload::<usize>(),
                        ) {
                            let rect = layer_drag_inner.response.rect;
                            let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                            // println!("Hovered payload {} idx {}", hovered_payload, idx);
                            let insert_idx = if *hovered_payload == idx {
                                // We are dragged onto ourselves
                                // println!("SELF");
                                ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                idx
                            } else if pointer.y < rect.center().y {
                                // Above us
                                // println!("ABOVE");
                                ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                // Because we move the destination UP when we remove the source
                                idx.max(0) - if *hovered_payload < idx { 1 } else { 0 }
                            } else {
                                // Below us
                                // println!("BELOW");
                                ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                // Because we move the destination UP when we remove the source
                                idx.min(model.geo_layers().len() - 1)
                                    + if *hovered_payload < idx { 0 } else { 1 }
                            };
                            // println!("Dragging from {} to {}", hovered_payload, insert_idx);
                            if let Some(dragged_payload) =
                                layer_drag_inner.response.dnd_release_payload()
                            {
                                // The user dropped onto this item.
                                // drag_from = Some(dragged_payload);
                                // drag_to = Some(Arc::new(insert_idx));
                                // println!("Reordering to: {}", insert_idx);
                                if let Some(picked) = model.picked()
                                    && picked.contains(&*dragged_payload)
                                {
                                    // println!(
                                    //     "Nothing picked. Selecting myself ({})",
                                    //     drag_from.clone().unwrap()
                                    // );
                                    // model.toggle_pick_by_id(*drag_from.clone().unwrap());
                                    model.reorder_selected_geometry_to(insert_idx);
                                } else {
                                    model.toggle_pick_by_id(*dragged_payload);
                                    model.reorder_selected_geometry_to(insert_idx);
                                    model.pick_clear();
                                }
                            }
                        }
                    }
                    // });
                });
            }); // End scrollarea
    });
}
