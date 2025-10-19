// use crate::ui::bottom_panel::bottom_panel;
use crate::ui::menu::main_menu;
use crate::ui::paper_chooser::paper_chooser_window;
use crate::ui::pen_crib::pen_crib_window;
use crate::view_model::{BAPViewModel, CommandContext, PIXELS_PER_MM};
use eframe::egui;
use egui::{
    Color32, Pos2, Rect, Stroke, StrokeKind, Vec2, pos2, vec2, was_tooltip_open_last_frame,
};

pub(crate) mod tool_window;
use tool_window::floating_tool_window;
pub(crate) mod bottom_panel;
pub(crate) mod menu;
pub(crate) mod paper_chooser;
pub(crate) mod pen_crib;
pub(crate) mod scene_toggle;
pub(crate) mod themes;
pub(crate) mod tool_button;

pub(crate) fn native_to_mm(native: Pos2, zoom: f32) -> Pos2 {
    (PIXELS_PER_MM * native) / zoom
}

pub(crate) fn mm_to_native(mm: Pos2, zoom: f32) -> Pos2 {
    (mm * zoom) / PIXELS_PER_MM
}

pub(crate) fn update_ui(model: &mut BAPViewModel, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Looks better on 4k montior
    ctx.set_pixels_per_point(1.5);

    let tbp = main_menu(model, ctx);
    scene_toggle::scene_toggle(model, ctx);

    let wtop = tbp.top();
    floating_tool_window(model, ctx, wtop);
    if model.paper_modal_open {
        paper_chooser_window(model, ctx);
    }
    if model.pen_crib_open {
        pen_crib_window(model, ctx);
    }

    let cp = egui::CentralPanel::default().show(ctx, |ui| {
        // ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

        let precursor = ui.cursor();
        // let painter = ui.painter();
        let (painter_resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::all());

        // println!("Painter rect: {:?}", painter_resp.rect);
        let (min, max) = (painter_resp.rect.min, painter_resp.rect.max);
        model.center_coords = pos2((min.x + max.x) / 2.0_f32, (min.y + max.y) / 2.0_f32);
        // println!("Center coords: {:?}", model.center_coords);

        // // Draw the paper
        let paper_rect = model.mm_rect_to_screen_rect(model.get_paper_rect());
        painter.rect(
            paper_rect,
            0.,
            model.paper_color,
            Stroke::NONE,
            egui::StrokeKind::Outside,
        );

        if let Some(imghandle) = &model.svg_img_handle {
            // let size_raw = imghandle.size_vec2();
            // let size = size_raw * model.view_zoom as f32 / PIXELS_PER_MM;
            // let center = mm_to_native(mm, zoom)
            let svgrect = model.svg_img_dims.expect(
                "Somehow we have an image handle with no dims.
                    This should be impossible. Dying.",
            );

            let rect = Rect::from_min_size(
                model.mm_to_frame_coords(svgrect.min),
                model.scale_mm_to_screen(svgrect.size()),
            );
            // println!("Rect for svg image {:?} is {:?}", svgrect, rect);
            painter.image(
                imghandle.id(),
                rect,
                // Rect::from_center_size(center, size),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Draw these lines _last_ so they overlap the drawing itself.
        if model.command_context == CommandContext::Origin {
            // println!("Drawing origin lines.");

            if let Some(pos) = ctx.pointer_latest_pos() {
                // println!("Got pointer pos: {:?}", &pos);
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

                painter.rect(
                    paper_tmp_rect,
                    0.,
                    model.paper_color.gamma_multiply(0.5),
                    Stroke::new(2., Color32::from_gray(128)),
                    StrokeKind::Middle,
                );

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
                painter.rect(
                    machine_rect,
                    0.,
                    Color32::TRANSPARENT,
                    Stroke::new(1., Color32::YELLOW),
                    StrokeKind::Outside,
                );
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
            painter.rect(
                machine_rect,
                0.,
                Color32::TRANSPARENT,
                Stroke::new(1., Color32::YELLOW),
                StrokeKind::Outside,
            );
        }

        if painter_resp.clicked() {
            match model.command_context {
                CommandContext::Origin => {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        model.origin = model.frame_coords_to_mm(pos)
                    }
                }
                CommandContext::None => (),
                CommandContext::Clip(pos2, pos3) => todo!(),
            }
            model.command_context = CommandContext::None;
        }

        if painter_resp.dragged() {
            model.look_at =
                // model.look_at - (PIXELS_PER_MM * painter_resp.drag_delta() / model.view_zoom as f32)
                model.look_at - model.scale_screen_to_mm(painter_resp.drag_delta())
        }

        if painter_resp.contains_pointer() {
            let delta = ui.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::MouseWheel {
                        unit: _,
                        delta,
                        modifiers,
                    } => Some(*delta),
                    _ => None,
                })
            });
            if let Some(delta) = delta {
                if delta.y > 0. {
                    // println!("Zoom +");
                    model.view_zoom = model.view_zoom * 1.1 * delta.y.abs() as f64;
                } else {
                    // println!("Zoom -");
                    model.view_zoom = model.view_zoom * (1.0 / 1.1) * delta.y.abs() as f64;
                }
                // println!("New view zoom: {}", model.view_zoom);
            }
        }

        bottom_panel::bottom_panel(model, ctx);

        (precursor, ui.cursor())
    });
}
