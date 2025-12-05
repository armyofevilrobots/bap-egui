use crate::core::config::DockPosition;
use crate::view_model::BAPViewModel;
use eframe::egui;
#[allow(unused)]
use egui::Stroke;
use egui::{CornerRadius, Frame, Grid, ImageSource, Pos2};
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
        // ui.shrink_width_to_current();
        // super::scene_toggle::scene_toggle_toolbox(model, ctx, ui);
        egui::ScrollArea::vertical()
            .max_height(default_height - 110.)
            .min_scrolled_height(default_height - 110.)
            .auto_shrink(match model.geo_layer_position() {
                DockPosition::Floating(_, _) => true,
                _ => false,
            })
            .show(ui, |ui| {
                // This is the actual window content.
                Grid::new("GeoLayersGrid").striped(true).show(ui, |ui| {
                    for (_idx, layer) in model.geo_layers().iter().enumerate() {
                        ui.image(ImageSource::Texture(layer.preview));
                        ui.label(&layer.name);
                        ui.label(layer.pen_uuid.as_urn().to_string());
                        ui.end_row();
                    }
                });
            });
    });
}
