use crate::BAPViewModel;
use crate::core::commands::ViewCommand;
use crate::view_model::CommandContext;
use eframe::egui;
use egui::{Align2, Layout};
use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};

pub(crate) fn gcode_editor_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::Window::new("Edit GCODE")
        .default_size(ctx.available_rect().shrink(128.).size())
        .anchor(Align2::CENTER_CENTER, (0., 0.))
        .show(ctx, |ui| {
            let mut builder = SyntaxSetBuilder::new();
            // builder.add_from_folder("../../resources/syntax", true).unwrap();
            let syntax_def = SyntaxDefinition::load_from_str(
                include_str!("../../resources/syntax/gcode.tmLanguage.sublime-syntax"),
                true,
                Some("gcode"),
            )
            .unwrap();
            builder.add(syntax_def);
            let syntax_set = builder.build();
            let theme_set = syntect::highlighting::ThemeSet::load_defaults();
            let syntax = egui_extras::syntax_highlighting::SyntectSettings {
                ps: syntax_set,
                ts: theme_set,
            };
            let theme =
                egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut layout_job = egui_extras::syntax_highlighting::highlight_with(
                    ui.ctx(),
                    ui.style(),
                    &theme,
                    buf.as_str(),
                    "gcode",
                    &syntax,
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts_mut(|f| f.layout_job(layout_job))
            };
            ui.set_width(600.);
            ui.heading("Edit GCODE");
            egui::ScrollArea::vertical()
                .max_height(ctx.available_rect().shrink(128.).height())
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(model.gcode_mut())
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    )
                });
            ui.add_space(8.);

            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("Cancel").clicked() {
                    // model.paper_modal_open = false
                    if let CommandContext::EditGcode(gcode) = model.command_context() {
                        if let Some(gcode) = gcode {
                            eprintln!("Resetting gcode.");
                            model.set_gcode(gcode)
                        }
                    }
                    model.cancel_command_context(true);
                }
                if ui.button("Ok").clicked() {
                    // model.paper_modal_open = false
                    // model.set_command_context(crate::view_model::CommandContext::None);
                    model.yolo_view_command(ViewCommand::SetGCode(model.gcode().clone()));
                    model.cancel_command_context(false);
                }
                ui.add_space(16.);
            });
        });
}
