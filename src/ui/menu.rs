use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{self};
use std::thread::spawn;

use crate::BAPViewModel;
use crate::core::commands::ViewCommand;
use crate::view_model::FileSelector;
use eframe::egui;
use egui::{Button, Rect, Separator, Visuals};
use rfd::FileDialog;

pub(crate) fn main_menu(model: &mut BAPViewModel, ctx: &egui::Context) -> Rect {
    let tbp = egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Project [spc-p-o]").clicked() {
                    let (tx, rx) = mpsc::channel::<FileSelector>();
                    model.file_selector = Some(rx);
                    spawn(move || {
                        let file = FileDialog::new()
                            .add_filter("bap2", &["bap2"])
                            .set_directory("")
                            .pick_file();
                        if let Some(path) = file {
                            tx.send(FileSelector::OpenProject(path.into()))
                                .expect("Failed to load project");
                        }
                    });
                }
                if ui.button("Save Project [spc-p-s]").clicked() {
                    //functionality
                }
                if ui.button("Save Project As [spc-p-a]").clicked() {
                    //functionality
                }
                ui.add(Separator::default());
                if ui.button("Import SVG [spc f i]").clicked() {
                    let (tx, rx) = mpsc::channel::<FileSelector>();
                    model.file_selector = Some(rx);
                    spawn(move || {
                        let file = FileDialog::new()
                            .add_filter("svg", &["svg"])
                            .add_filter("hpgl", &["hpgl"])
                            .add_filter("wkt", &["wkt"])
                            .set_directory("")
                            .pick_file();
                        if let Some(path) = file {
                            tx.send(FileSelector::ImportSVG(path.into()))
                                .expect("Failed to send SVG import over MPSC.");
                        }
                    });
                };
                if ui.button("Load PGF [space f p]").clicked() {
                    let (tx, rx) = mpsc::channel::<FileSelector>();
                    model.file_selector = Some(rx);
                    spawn(move || {
                        let file = FileDialog::new()
                            .add_filter("pgf", &["pgf"])
                            .set_directory("")
                            .pick_file();
                        if let Some(path) = file {
                            tx.send(FileSelector::LoadPGF(path.into()))
                                .expect("Failed to send SVG import over MPSC.");
                            eprintln!("Not implemented yet!");
                        }
                    });
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
                    if let Some(cmd_out) = &model.cmd_out {
                        cmd_out.send(ViewCommand::Undo).unwrap_or_else(|err| {
                            eprintln!("Failed to undo due to: {:?}. Terminating.", err);
                            exit(-1);
                        })
                    };
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
