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
use egui::{Color32, ComboBox, Id, Layout, Rect, Slider, Stroke, pos2, vec2};
use geo::algorithm::bool_ops::BooleanOps;
use geo::{BoundingRect, coord};

pub(crate) fn arrange_mat(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    // ui: &mut egui::Ui,
) /*-> ModalResponse<()>*/
{
    const BOX_SIZE: f64 = 400.;
    const PAINTER_HEIGHT: f32 = 520.0;

    if let CommandContext::MatToTarget(mat_target) = &mut model.command_context() {
        egui::Modal::new(Id::new("ArrangeMat")).show(ctx, |ui| {
            ui.vertical(|ui| {
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

                let precur = ui.cursor().min;
                let (painter_resp, painter) =
                    ui.allocate_painter(vec2(700., PAINTER_HEIGHT), egui::Sense::all());
                let cur = ui.cursor().min;
                let prect = painter_resp.rect;
                let ofs = (prect.min.clone() + vec2(10., 10.)).to_vec2();
                let (target_width, target_height, pwidth, pheight, ratio, scale) =
                    if let MatTarget::Paper(_whatever) = mat_target {
                        // let (px, py) = model.paper_size().dimensions();
                        // let (px, py) = match model.paper_orientation() {
                        //     Orientation::Landscape => (py, px),
                        //     Orientation::Portrait => (px, py),
                        // };
                        let (px, py) = model
                            .paper_size()
                            .dimensions_oriented(&model.paper_orientation());
                        let ratio = py / px;
                        let (pwidth, pheight, scale) = if ratio > 1. {
                            (BOX_SIZE / ratio, BOX_SIZE, BOX_SIZE * ratio)
                        } else {
                            (BOX_SIZE, BOX_SIZE * ratio, BOX_SIZE / ratio)
                        };
                        (px, py, pwidth as f32, pheight as f32, ratio as f32, scale)
                    } else if let MatTarget::Machine(_somethingelse) = mat_target {
                        let (px, py) = model.machine_config().limits();
                        let ratio = py / px;
                        let (pwidth, pheight, scale) = if ratio > 1. {
                            (BOX_SIZE / ratio, BOX_SIZE, BOX_SIZE / ratio)
                        } else {
                            (BOX_SIZE, BOX_SIZE * ratio, BOX_SIZE * ratio)
                        };
                        (px, py, pwidth as f32, pheight as f32, ratio as f32, scale)
                    } else {
                        // It has to be machine.
                        let (mx, my) = model.machine_config().limits();
                        let (px, py) = model.paper_size().dimensions();
                        let (px, py) = match model.paper_orientation() {
                            Orientation::Landscape => (py, px),
                            Orientation::Portrait => (px, py),
                        };

                        let mrect = geo::Rect::new(coord! {x: 0., y: 0.}, coord! { x: mx, y: my })
                            .to_polygon();
                        let prect = geo::Rect::new(coord! {x: 0., y: 0.}, coord! { x: px, y: py })
                            .to_polygon();
                        let urect = mrect.intersection(&prect).bounding_rect().unwrap();
                        let (px, py) = (urect.width() as f32, urect.height() as f32);
                        let ratio = (py / px) as f64;
                        let (pwidth, pheight, scale) = if ratio > 1. {
                            (BOX_SIZE / ratio, BOX_SIZE, BOX_SIZE * ratio)
                        } else {
                            (BOX_SIZE, BOX_SIZE * ratio, BOX_SIZE / ratio)
                        };
                        (
                            px as f64,
                            py as f64,
                            pwidth as f32,
                            pheight as f32,
                            ratio as f32,
                            scale,
                        )
                    };

                let scale = pwidth / BOX_SIZE as f32;
                // println!("SCALE IS {}", scale);

                let mut values = mat_target.values();
                let mut mat_values_changed = false;
                // let (mut mtop, mut mright, mut mbottom, mut mleft) = match values {
                //     MatValues::Equal(all) => (all, all, all, all),
                //     MatValues::VertHoriz(vert, horiz) => (vert, horiz, vert, horiz),
                //     MatValues::TopRightBottomLeft(t, r, b, l) => (t, r, b, l),
                // };
                let (mut mtop, mut mright, mut mbottom, mut mleft) = values.get_trbl();

                let drp_rect = Rect::from_center_size(
                    pos2(cur.x + prect.width() / 2., cur.y - prect.height() / 2.),
                    vec2(pwidth as f32, pheight as f32),
                );

                // println!("Cursor: {:?}", cur);
                painter.rect(
                    drp_rect,
                    0.,
                    // Color32::from_white_alpha(128),
                    model.paper_color(),
                    Stroke::new(1., Color32::from_black_alpha(128)),
                    egui::StrokeKind::Inside,
                );
                let pcol = model.paper_color().to_tuple();
                let tcol = (
                    ((pcol.0 as u32 + 85) % 255) as u8,
                    ((pcol.0 as u32 + 85) % 255) as u8,
                    ((pcol.0 as u32 + 85) % 255) as u8,
                );

                // Turns out the scale from above is just 1/ratio.
                let scale = pwidth / model.paper_size().dimensions().0 as f32;
                let dimensions_text_color = Color32::from_rgb(tcol.0, tcol.1, tcol.2);
                let mat_center = pos2(
                    (mleft - mright) as f32 * scale,
                    (mtop - mbottom) as f32 * scale,
                );

                if let Some(extents) = model.source_image_extents() {
                    let mat_size = vec2(
                        pwidth - ((mleft + mright) as f32 * scale),
                        pheight - ((mtop + mbottom) as f32 * scale),
                    );
                    // let mat_rect =
                    //     Rect::from_min_max(precur+vec2(mleft as f32*scale, mtop as f32*scale), cur-vec2(prect.width()-prect.width() as f32*scale, mbottom as f32*scale));
                    let mat_rect = Rect::from_min_max(
                        drp_rect.min + vec2(mleft as f32 * scale, mtop as f32 * scale),
                        drp_rect.max - vec2(mright as f32 * scale, mbottom as f32 * scale),
                    );

                    // println!(
                    //     "The layout is:\n\tPaper Size:{:3.1},{:3.2}\n\nPrecur: {:?},{:?}\n\tPrect:{:3.1},{:3.2}
                    //     \tRatio:{:3.3}\n\tScale:{:3.3}\n\tTRBL: {:3.1},{:3.1},{:3.1},{:3.1},
                    //     \tArt Center:{}\n\tArt Size:{}\n\tMat Rect:{}",
                    //     model.paper_size().dimensions().0,
                    //     model.paper_size().dimensions().1,
                    //     precur.x, precur.y,
                    //     pwidth,
                    //     pheight,
                    //     ratio,
                    //     scale,
                    //     mtop,
                    //     mright,
                    //     mbottom,
                    //     mleft,
                    //     mat_center,
                    //     mat_size,
                    //     mat_rect
                    // );
                    painter.rect_stroke(
                        mat_rect.clone(),
                        0.,
                        Stroke::new(1., Color32::from_rgb(tcol.0, tcol.1, tcol.2)),
                        egui::StrokeKind::Inside,
                    );

                    // Fuck it, draw the image over top, even if it's super expensive.
                    if let Some(imghandle) = model.source_image_handle() {
                        if let Some(img_extents) = model.source_image_extents() {
                            // let img_extents = model.scale_mm_to_screen(img_extents);
                            // let img_size = model.scale_mm_to_screen(img_extents.size());
                            let img_size = img_extents.size();
                            let mat_center = mat_rect.min + ((mat_rect.max - mat_rect.min) / 2.0);
                            let img_ratio = img_size.y / img_size.x;
                            let img_scale = if img_ratio < ratio {
                                mat_rect.width() / img_size.x
                            } else {
                                mat_rect.height() / img_size.y
                            };
                            // println!("SCALE: {:3.2}", img_scale);
                            // println!("\tmat width: {:3.2}", mat_rect.width());
                            // println!("\timg_width: {:3.2}", img_extents.width());

                            painter.image(
                                imghandle.id(),
                                // mat_rect.clone(),
                                Rect::from_center_size(
                                    mat_center,
                                    vec2(img_size.x * img_scale, img_size.y * img_scale),
                                ),
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    }
                }

                // We have to clamp the available margins to avoid inverted matt areas.
                let (paper_width, paper_height) = model.paper_size().dimensions();
                let (mut paper_width, mut paper_height) = match model.paper_orientation() {
                    Orientation::Landscape => (paper_height, paper_width),
                    Orientation::Portrait => (paper_width, paper_height),
                };

                let (vert_space, horiz_space) = match values {
                    MatValues::Equal(xyval) => (
                        (target_height as f64 - xyval).min(target_width as f64 - xyval),
                        (target_height as f64 - xyval).min(target_width as f64 - xyval),
                    ),
                    MatValues::VertHoriz(yval, xval) => (100., 100.),
                    MatValues::TopRightBottomLeft(topval, rightval, bottomval, leftval) => {
                        (100., 100.)
                    }
                };
                println!(
                    "TOTAL HEIGHT/WIDTH: {:3.2}/{:3.2}",
                    target_height, target_width
                );
                println!("VERT SPACE: {}", vert_space);
                println!("HORIZ SPACE: {}", horiz_space);
                // This is the top margin, and is always visible and editable.
                #[allow(deprecated)]
                let top_margin_response = ui.allocate_ui_at_rect(
                    Rect::from_center_size(
                        pos2(
                            cur.x + prect.width() / 2. - 24.,
                            cur.y - prect.height() / 2. - pheight / 2. - 16.,
                        ),
                        vec2(96., 16.),
                    ),
                    |ui| {
                        if ui
                            .add(
                                Slider::new(&mut mtop, 0.1..=vert_space).logarithmic(true), // .text("Top"),
                            )
                            .changed()
                        {
                            mat_values_changed = true;
                        }
                    },
                );

                // This is the right margin, and is visible unless editing the ALL margins
                #[allow(deprecated)]
                let right_margin_response = ui.allocate_ui_at_rect(
                    Rect::from_center_size(
                        pos2(
                            cur.x + prect.width() / 2. + pwidth / 2. + 15.,
                            cur.y - prect.height() / 2.,
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
                                Slider::new(&mut mright, 0.1..=horiz_space)
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
                let bottom_margin_response = ui.allocate_ui_at_rect(
                    Rect::from_center_size(
                        pos2(
                            cur.x + prect.width() / 2. - 24.,
                            cur.y - prect.height() / 2. + pheight / 2. + 16., // - 15.,
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
                                Slider::new(&mut mbottom, 0.1..=vert_space).logarithmic(true), // .text("Top"),
                            )
                            .changed()
                        {
                            mat_values_changed = true;
                        };
                    },
                );

                // This is the left margin, and is INvisible unless editing the individual margins
                #[allow(deprecated)]
                let left_margin_response = ui.allocate_ui_at_rect(
                    Rect::from_center_size(
                        pos2(
                            cur.x + prect.width() / 2. - pwidth / 2. - 32.,
                            cur.y - prect.height() / 2.,
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
                                Slider::new(&mut mleft, 0.1..=horiz_space)
                                    .vertical()
                                    .logarithmic(true), // .text("Top"),
                            )
                            .changed()
                        {
                            mat_values_changed = true;
                        };
                    },
                );
                if mat_values_changed {
                    let new_values = match values {
                        MatValues::Equal(_) => MatValues::Equal(mtop),
                        MatValues::VertHoriz(_, _) => MatValues::VertHoriz(mtop, mright),
                        MatValues::TopRightBottomLeft(_, _, _, _) => {
                            MatValues::TopRightBottomLeft(mtop, mright, mbottom, mleft)
                        }
                    };
                    let new_target = match mat_target {
                        Machine(_mat_values) => Machine(new_values),
                        Paper(_mat_values) => Paper(new_values),
                        Smart(_mat_values) => Smart(new_values),
                    };
                    model.set_command_context(CommandContext::MatToTarget(new_target));
                }

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
        let mut values = match target.clone() {
            Machine(mat_values) => mat_values,
            Paper(mat_values) => mat_values,
            Smart(mat_values) => mat_values,
        };
        let (mut mtop, mut mright, mut mbottom, mut mleft) = match values.clone() {
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
        /*
        ComboBox::from_label("Matting Type")
            .selected_text(format!("{}", values))
            .show_ui(ui, |ui| {
                for val in [
                    MatValues::Equal(mtop),
                    MatValues::VertHoriz(mtop, mright),
                    MatValues::TopRightBottomLeft(mtop, mright, mbottom, mleft),
                ] {}
            });
            */
        CommandContext::MatToTarget(target)
    } else {
        command_context.clone()
    }
}
