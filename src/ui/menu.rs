use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{self};
use std::thread::spawn;

use crate::BAPViewModel;
use crate::core::commands::ViewCommand;
use eframe::egui;
use egui::{Rect, Separator, Visuals};
use rfd::FileDialog;

pub(crate) fn main_menu(model: &mut BAPViewModel, ctx: &egui::Context) -> Rect {
    let tbp = egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Project").clicked() {}
                if ui.button("Save Project").clicked() {
                    //functionality
                }
                ui.add(Separator::default());
                if ui.button("Import SVG").clicked() {
                    let (tx, rx) = mpsc::channel::<PathBuf>();
                    model.svg_import_mpsc = Some(rx);
                    spawn(move || {
                        let file = FileDialog::new()
                            .add_filter("svg", &["svg"])
                            .add_filter("hpgl", &["hpgl"])
                            .add_filter("wkt", &["wkt"])
                            .set_directory("")
                            .pick_file();
                        if let Some(path) = file {
                            tx.send(path.into())
                                .expect("Failed to send SVG import over MPSC.");
                        }
                    });
                };
                if ui.button("Quit").clicked() {
                    if let Some(cmd_out) = &model.cmd_out {
                        cmd_out.send(ViewCommand::Quit).unwrap_or_else(|err| {
                            eprintln!("Failed to quit due to: {:?}. Terminating.", err);
                            exit(-1);
                        })
                    };
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Cut").clicked() {
                    //functionality
                }
                if ui.button("Copy").clicked() {
                    //functionality
                }
                if ui.button("Paste").clicked() {
                    //funtionality
                }
            });

            let mut dark_mode = ui.visuals().dark_mode.clone();

            ui.menu_button("View", |ui| {
                // if ui.tobutton("Dark")..clicked() {
                if ui.toggle_value(&mut dark_mode, "Dark mode").clicked() {
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
