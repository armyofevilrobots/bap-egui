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

pub(crate) fn floating_hatch_tool_window(
    model: &mut BAPViewModel,
    ctx: &egui::Context,
    wtop: f32,
    _toasts: &mut Toasts,
) {
    let corner_radius_save = ctx.style().visuals.window_corner_radius.clone();
    let default_height = ctx.content_rect().height() - wtop - 22.; //23.;
    let win = egui::Window::new("Hatch")
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
        .default_pos((128., 32.))
        .collapsible(false)
        .resizable([false, false]);

    let win = win.title_bar(false); //.current_pos(Pos2 { x, y }),

    let _win_response = win.show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(4.);
            ui.heading("Hatch/Fill");
            ui.shrink_width_to_current();
            ui.add_space(8.);

            egui::ScrollArea::vertical()
                .max_height(default_height - ui.cursor().top() - 8.)
                // .min_scrolled_height(default_height - ui.cursor().top() - 8.)
                .auto_shrink(true)
                .show(ui, |ui| {}); // End scrollarea
        });
        ctx.style_mut(|style| style.visuals.window_corner_radius = corner_radius_save);
    });
}
