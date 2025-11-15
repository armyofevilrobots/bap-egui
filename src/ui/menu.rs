use std::process::exit;
use std::sync::mpsc::{self};

use crate::BAPViewModel;
use crate::core::commands::ViewCommand;
use crate::view_model::FileSelector;
use eframe::egui;
use egui::{Button, Rect, Separator, Visuals};

pub(crate) fn main_menu(model: &mut BAPViewModel, ctx: &egui::Context) -> Rect {
    let tbp = egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New project [spc-p-n]").clicked() {
                    model.yolo_view_command(ViewCommand::ResetProject);
                }
                if ui.button("Open Project [spc-p-o]").clicked() {
                    model.open_project_with_dialog();
                }
                if ui
                    .add_enabled(
                        model.file_path.is_some(),
                        Button::new("Save Project [spc-p-s]"),
                    )
                    .clicked()
                {
                    //functionality
                    let (tx, rx) = mpsc::channel::<FileSelector>();
                    model.file_selector = Some(rx);
                    tx.send(FileSelector::SaveProject)
                        .expect("failed to send project load signal");
                }
                if ui.button("Save Project As [spc-p-a]").clicked() {
                    //functionality
                    model.save_project_with_dialog();
                }
                ui.add(Separator::default());
                if ui.button("Import SVG [spc f i]").clicked() {
                    model.import_svg_with_dialog();
                };
                if ui.button("Load PGF [space f p]").clicked() {
                    model.load_pgf_with_dialog();
                };
                if ui.button("Quit [cmd-Q]").clicked() {
                    if let Some(cmd_out) = &model.cmd_out {
                        cmd_out.send(ViewCommand::Quit).unwrap_or_else(|err| {
                            eprintln!("Failed to quit due to: {:?}. Terminating.", err);
                            exit(-1);
                        })
                    };
                }
            });

            ui.menu_button("Edit", |ui| {
                // if ui.button("Undo").clicked() {
                //     if let Some(cmd_out) = &model.cmd_out {
                //         cmd_out.send(ViewCommand::Undo).unwrap_or_else(|err| {
                //             eprintln!("Failed to undo due to: {:?}. Terminating.", err);
                //             exit(-1);
                //         })
                //     };
                // }
                if ui
                    .add_enabled(model.undo_available, Button::new("Undo [u]"))
                    .clicked()
                {
                    model.undo();
                };
            });

            let mut dark_mode = ui.visuals().dark_mode.clone();

            ui.menu_button("View", |ui| {
                // if ui.tobutton("Dark")..clicked() {
                if ui
                    .toggle_value(&mut dark_mode, "Dark mode [space t d]")
                    .clicked()
                {
                    ctx.set_visuals(if dark_mode {
                        Visuals::dark()
                    } else {
                        Visuals::light()
                    });
                };
            })
        });
        ui.cursor()
    });
    tbp.inner
}
