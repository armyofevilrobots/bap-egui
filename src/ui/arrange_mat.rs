// use crate::ui::bottom_panel::bottom_panel;
use super::tool_button::tool_button;
use crate::{
    core::{
        commands::{
            MatTarget::{self, Machine, Paper, Smart},
            MatValues,
        },
        paper::Orientation,
    },
    view_model::{BAPViewModel, CommandContext},
};
use eframe::egui;
use egui::{Color32, ComboBox, Id, Layout, Rect, Slider, Stroke, Ui, pos2, vec2};
use geo::algorithm::bool_ops::BooleanOps;
use geo::{BoundingRect, coord};

const BOX_WIDTH: f64 = 400.;
const BOX_HEIGHT: f64 = 700.;
const PAINTER_HEIGHT: f64 = 520.0;

pub(crate) fn mat_target_selector(model: &mut BAPViewModel, ui: &mut Ui) {
    ui.set_width(700.);
    ui.heading("Arrange content to matted area");

    model.set_command_context(matt_type_combobox(&mut model.command_context(), ui));
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
    ui.label("Note: This UI is provisional, and will change in the future.");
}

fn calculate_avail_size(
    paper_width_mm: f64,
    paper_height_mm: f64,
    machine_width_mm: f64,
    machine_height_mm: f64,
    mat_target: &MatTarget,
) -> (f64, f64) {
    match mat_target {
        MatTarget::Paper(_) => (paper_width_mm, paper_height_mm),
        MatTarget::Machine(_) => (machine_width_mm, machine_height_mm),
        MatTarget::Smart(_) => {
            let mrect = geo::Rect::new(
                coord! {x: 0., y: 0.},
                coord! { x: machine_width_mm, y: machine_height_mm },
            )
            .to_polygon();
            let prect = geo::Rect::new(
                coord! {x: 0., y: 0.},
                coord! { x: paper_width_mm, y: paper_height_mm },
            )
            .to_polygon();
            let urect = mrect.intersection(&prect).bounding_rect().unwrap();
            (urect.width(), urect.height())
        }
    }
}

fn calc_avail_display_dims_px(avail_width_mm: f64, avail_height_mm: f64) -> (f64, f64) {
    let avail_ratio_y_x = (avail_height_mm / avail_width_mm) as f64;
    if avail_ratio_y_x > 1. {
        (BOX_WIDTH / avail_ratio_y_x, BOX_WIDTH)
    } else {
        (BOX_WIDTH, BOX_WIDTH * avail_ratio_y_x)
    }
}

pub(crate) fn arrange_mat(model: &mut BAPViewModel, ctx: &egui::Context) {
    if let CommandContext::MatToTarget(mat_target) = &mut model.command_context() {
        // First, show the selector for paper/machine/smart and orientation of the paper.
        egui::Modal::new(Id::new("ArrangeMat")).show(ctx, |ui| {
            ui.vertical(|ui| {
                mat_target_selector(model, ui);

                let (paper_width_mm, paper_height_mm) = model
                    .paper_size()
                    .dimensions_oriented(&model.paper_orientation());
                let (machine_width_mm, machine_height_mm) = model.machine_config().limits();
                // Next, calculate the target area's dimensions.
                let (avail_width_mm, avail_height_mm) = calculate_avail_size(
                    paper_width_mm,
                    paper_height_mm,
                    machine_width_mm,
                    machine_height_mm,
                    mat_target,
                );
                // println!(
                //     "\n\nAvailable space:\n\tX: {:3.2} Y:{:3.2}",
                //     avail_width_mm, avail_height_mm
                // );

                // Now, calculate the size of the _displayed_ rect.
                let (ui_avail_width_px, ui_avail_height_px) =
                    calc_avail_display_dims_px(avail_width_mm, avail_height_mm);

                // OK, now we figure out the current size of the matted area.
                let (mut mat_top, mut mat_right, mut mat_bottom, mut mat_left) =
                    mat_target.get_trbl();

                // Next, we create the drawing area/painter.
                let precur = ui.cursor().min;
                let (painter_resp, painter) = ui.allocate_painter(
                    vec2(BOX_HEIGHT as f32, PAINTER_HEIGHT as f32),
                    egui::Sense::all(),
                );
                let cur = ui.cursor().min;
                let painter_rect = painter_resp.rect;

                // Then draw the mockup paper.
                let available_area_rect_px = Rect::from_center_size(
                    pos2(
                        precur.x + painter_rect.width() / 2.,
                        precur.y + painter_rect.height() / 2.,
                    ),
                    vec2(ui_avail_width_px as f32, ui_avail_height_px as f32),
                );
                // println!(
                //     "Available area rect:\n\t{}: {:3.2},{:3.2}",
                //     available_area_rect_px,
                //     available_area_rect_px.width(),
                //     available_area_rect_px.height()
                // );

                // Actually draw the paper rect.
                painter.rect(
                    available_area_rect_px,
                    0.,
                    model.paper_color(),
                    Stroke::new(1., Color32::from_black_alpha(128)),
                    egui::StrokeKind::Inside,
                );

                // Next, figure out the matted rectangle and draw that, correctly I would hope.
                // First, figure the scale and colors...
                let mat_scale = available_area_rect_px.width() / avail_width_mm as f32;
                let pcol = model.paper_color().to_tuple();
                let tcol = (
                    ((pcol.0 as u32 + 85) % 255) as u8,
                    ((pcol.0 as u32 + 85) % 255) as u8,
                    ((pcol.0 as u32 + 85) % 255) as u8,
                );
                // Then the actual rect.
                let mat_rect = Rect::from_min_max(
                    available_area_rect_px.min
                        + vec2(mat_left as f32 * mat_scale, mat_top as f32 * mat_scale),
                    available_area_rect_px.max
                        - vec2(mat_right as f32 * mat_scale, mat_bottom as f32 * mat_scale),
                );
                // println!(
                //     "MAT RECT:\n\t{}: {:3.2},{:3.2}",
                //     mat_rect,
                //     mat_rect.width(),
                //     mat_rect.height()
                // );

                // Finally, paint it.
                painter.rect_stroke(
                    mat_rect.clone(),
                    0.,
                    Stroke::new(1., Color32::from_rgb(tcol.0, tcol.1, tcol.2)),
                    egui::StrokeKind::Inside,
                );

                // Now draw the matting margin edit stuff.
                {
                    // How much space do we actually have, maximum?
                    let values = mat_target.values();
                    let mut mat_values_changed = false;
                    let (vert_space_mm, horiz_space_mm) = match values {
                        MatValues::Equal(xyval) => {
                            let vspace_mm = (avail_height_mm as f64 / 2.);
                            let hspace_mm = (avail_width_mm as f64 / 2.);
                            (vspace_mm.min(hspace_mm) - 1., hspace_mm.min(vspace_mm) - 1.)
                        }
                        MatValues::VertHoriz(yval, xval) => (
                            (avail_height_mm as f64 / 2.) - 1.,
                            (avail_width_mm as f64 / 2.) - 1.,
                        ),
                        MatValues::TopRightBottomLeft(_topval, _rightval, _bottomval, _leftval) => {
                            (
                                (avail_height_mm as f64 / 2.) - 1.,
                                (avail_width_mm as f64 / 2.) - 1.,
                            )
                        }
                    };

                    // println!("Current mat values:");
                    // println!(
                    //     "\tTOP: {:3.1}, RIGHT: {:3.1}, BOTTOM: {:3.1}, LEFT: {:3.1}",
                    //     mat_top, mat_right, mat_bottom, mat_left
                    // );
                    // println!("Current available space:");
                    // println!("\tV: {:3.1} H: {:3.1}", vert_space_mm, horiz_space_mm);

                    // Finally, draw the temporary sized art in the area we have.
                    if let Some(imghandle) = model.source_image_handle() {
                        if let Some(extents) = model.source_image_extents() {
                            let image_center = mat_rect.center();
                            let img_ratio_y_x = extents.height() / extents.width();
                            let mat_ratio_y_x = mat_rect.height() / mat_rect.width();
                            // If the image is taller/skinnier than the mat, then the width defines the size
                            let img_out_rect_px = if img_ratio_y_x <= mat_ratio_y_x {
                                Rect::from_center_size(
                                    image_center,
                                    vec2(mat_rect.width(), mat_rect.width() * img_ratio_y_x),
                                )
                            } else {
                                Rect::from_center_size(
                                    image_center,
                                    vec2(mat_rect.height() / img_ratio_y_x, mat_rect.height()),
                                )
                            };
                            painter.image(
                                imghandle.id(),
                                img_out_rect_px,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), //UV
                                Color32::WHITE,
                            );
                        }
                    }

                    #[allow(deprecated)]
                    let _top_margin_response = ui.allocate_ui_at_rect(
                        Rect::from_center_size(
                            pos2(
                                available_area_rect_px.center().x - 24.,
                                available_area_rect_px.min.y - 16.,
                            ),
                            vec2(96., 16.),
                        ),
                        |ui| {
                            if ui
                                .add(
                                    Slider::new(&mut mat_top, 0.1..=vert_space_mm)
                                        .logarithmic(true),
                                )
                                .changed()
                            {
                                mat_values_changed = true;
                            }
                        },
                    );

                    // This is the right margin, and is visible unless editing the ALL margins
                    #[allow(deprecated)]
                    let _right_margin_response = ui.allocate_ui_at_rect(
                        Rect::from_center_size(
                            pos2(
                                available_area_rect_px.max.x + 15.,
                                cur.y - available_area_rect_px.height() / 2. - 48.,
                            ),
                            vec2(16., 96.),
                        ),
                        |ui| {
                            if ui
                                .add_enabled(
                                    match values {
                                        MatValues::Equal(_) => false,
                                        _ => true,
                                    },
                                    Slider::new(&mut mat_right, 0.1..=horiz_space_mm)
                                        .vertical()
                                        .logarithmic(true), // .text("Top"),
                                )
                                .changed()
                            {
                                mat_values_changed = true;
                            };
                        },
                    );

                    // This is the bottom margin, and is visible unless editing the ALL margins
                    #[allow(deprecated)]
                    let _bottom_margin_response = ui.allocate_ui_at_rect(
                        Rect::from_center_size(
                            pos2(
                                available_area_rect_px.center().x - 24.,
                                available_area_rect_px.max.y + 16.,
                            ),
                            vec2(96., 16.),
                        ),
                        |ui| {
                            if ui
                                .add_enabled(
                                    match values {
                                        MatValues::TopRightBottomLeft(_, _, _, _) => true,
                                        _ => false,
                                    },
                                    Slider::new(&mut mat_bottom, 0.1..=vert_space_mm)
                                        .logarithmic(true), // .text("Top"),
                                )
                                .changed()
                            {
                                mat_values_changed = true;
                            };
                        },
                    );

                    // This is the left margin, and is INvisible unless editing the individual margins
                    #[allow(deprecated)]
                    let _left_margin_response = ui.allocate_ui_at_rect(
                        Rect::from_center_size(
                            pos2(
                                available_area_rect_px.min.x - 36.,
                                cur.y - available_area_rect_px.height() / 2. - 48.,
                            ),
                            vec2(16., 96.),
                        ),
                        |ui| {
                            if ui
                                .add_enabled(
                                    match values {
                                        MatValues::TopRightBottomLeft(_, _, _, _) => true,
                                        _ => false,
                                    },
                                    Slider::new(&mut mat_left, 0.1..=horiz_space_mm)
                                        .vertical()
                                        .logarithmic(true), // .text("Top"),
                                )
                                .changed()
                            {
                                mat_values_changed = true;
                            };
                        },
                    );

                    // Handle the change in state if we moved stuff around.
                    if mat_values_changed {
                        let new_values = match values {
                            MatValues::Equal(_) => MatValues::Equal(mat_top),
                            MatValues::VertHoriz(_, _) => MatValues::VertHoriz(mat_top, mat_right),
                            MatValues::TopRightBottomLeft(_, _, _, _) => {
                                MatValues::TopRightBottomLeft(
                                    mat_top, mat_right, mat_bottom, mat_left,
                                )
                            }
                        };
                        let new_target = match mat_target {
                            Machine(_mat_values) => Machine(new_values),
                            Paper(_mat_values) => Paper(new_values),
                            Smart(_mat_values) => Smart(new_values),
                        };
                        model.set_command_context(CommandContext::MatToTarget(new_target));
                    }
                }

                // Finally the OK/Cancel stuff.
                ui.advance_cursor_after_rect(Rect::from_min_size(cur, vec2(0., 0.)));
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Ok").clicked() {
                        if let CommandContext::MatToTarget(target) = model.command_context() {
                            model.mat_to_target(target);
                            model.cancel_command_context(false);
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        model.cancel_command_context(true);
                    }
                });
            });
        });
    }
}

pub(crate) fn matt_type_combobox(
    command_context: &mut CommandContext,
    ui: &mut egui::Ui,
) -> CommandContext {
    if let CommandContext::MatToTarget(mut target) = command_context.clone() {
        let target_string = target.to_string();
        let mut changed = false;
        let values = match target.clone() {
            Machine(mat_values) => mat_values,
            Paper(mat_values) => mat_values,
            Smart(mat_values) => mat_values,
        };
        let (_mtop, _mright, _mbottom, _mleft) = match values.clone() {
            MatValues::Equal(all) => (all, all, all, all),
            MatValues::VertHoriz(vert, horiz) => (vert, horiz, vert, horiz),
            MatValues::TopRightBottomLeft(t, r, b, l) => (t, r, b, l),
        };

        ComboBox::from_label("Mat Target")
            .selected_text(format!("{}", target_string))
            .show_ui(ui, |ui| {
                for target_opt in MatTarget::options_with_values(&values).iter() {
                    if ui
                        .selectable_value(
                            &mut target,
                            target_opt.clone(),
                            format!("{}", target_opt),
                        )
                        .clicked()
                    {
                        changed = true;
                        target = target_opt.clone();
                    };
                }
            });
        CommandContext::MatToTarget(target)
    } else {
        command_context.clone()
    }
}
