use crate::{BAPViewModel, core::commands::ViewCommand};
use eframe::egui;
use egui::{CollapsingHeader, Layout, ScrollArea};

pub(crate) fn config_editor_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::Modal::new(egui::Id::new("Global Configuration"))
        // .id(egui::Id::new("global_config_win"))
        // .collapsible(false)
        // .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        // .constrain_to(ctx.content_rect().shrink(32.))
        .show(ctx, |ui| {
            ui.set_width(400.);
            ui.set_height(400.);
            ui.heading("Configuration");
            ScrollArea::vertical().show(ui, |ui| {
                CollapsingHeader::new("NC Post")
                    .open(Some(true))
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut model.config_mut().post_options.reorder_by_tool,
                            "Reorder operations by tool id",
                        );
                        ui.label(
                            "Enabling this option ensures that tool operations will \
                        be kept together by tool, reducing tool changes at the expense \
                        of losing the ability to interleave layers of colors.",
                        )
                    });
            });
            ScrollArea::vertical().show(ui, |ui| {
                CollapsingHeader::new("Import Options")
                    .open(Some(true))
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut model.config_mut().import_options.import_pgf_pens,
                            "Import pens from PGF files",
                        );
                        ui.label(
                            "PGF files have their own pens defined. Turning this off \
                            results in setting them all to the default pen (in the future we'll \
                            map to the closest pen in the crib).",
                        );
                        ui.checkbox(
                            &mut model.config_mut().import_options.generate_pens_from_svg,
                            "Generate pens for SVG imports",
                        );
                        ui.label(
                            "Generate new pens for SVG imports automatically. Turning this off \
                            results in setting them all to the default pen (in the future we'll \
                            map to the closest pen in the crib).",
                        );
                    });
            });
            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("Cancel").clicked() {
                    model.cancel_command_context(true);
                }
                if ui.button("Ok").clicked() {
                    model.yolo_view_command(ViewCommand::UpdateConfig(model.config().clone()));
                    model.cancel_command_context(false);
                }
            });
        });
}
