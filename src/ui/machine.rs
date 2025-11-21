use std::{collections::HashMap, f64::consts::PI};

use csscolorparser::Color;
use eframe::Frame;
use egui::{
    Color32, Id, Layout, Rect, Slider, Stroke, StrokeKind, Style, TextEdit, Vec2,
    epaint::PathStroke, pos2, vec2,
};
use indexmap::IndexMap;

use crate::{
    core::project::PenDetail,
    view_model::{BAPViewModel, CommandContext},
};

pub fn machine_editor_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::Modal::new(Id::new("Pen Editor")).frame(egui::containers::Frame::window(&Style::default())).show(ctx, |ui| {
        ui.vertical(|ui|{
            let scrollarea_resp=egui::ScrollArea::vertical()
                .auto_shrink(true)
                .max_height(600.)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show(ui, |ui| {
                // ui.set_height(600.);

                ui.set_width(600.);
                ui.heading(format!("Edit Machine {}", model.machine_config.name()));
                // Editor for the TOOL ID (this is a tracking ID, not the machine ID)
                ui.label("Name:");
                let mut tmp_name = model.machine_config.name();
                if ui.text_edit_singleline(&mut tmp_name).changed() {
                    model.machine_config.set_name(tmp_name);
                };

                // This is the skim height section. It handles how high we lift
                // the pen when doing rapids between lines.
                ui.collapsing("Configuration", |ui|{
                    {
                        let mut skim = model.machine_config.skim().unwrap_or(0.0);
                        ui.add(
                            Slider::new(&mut skim, 0.0f64..=50.0f64)
                                .text(if model.machine_config.skim().unwrap_or(0.0) > 0.0 {
                                    "Skim height"
                                } else {
                                    "Skim disabled"
                                })
                                // .logarithmic(true)
                                .custom_formatter(|n, _| {
                                    if n > 0.0 {
                                        format!("{:2.1}mm", n)
                                    } else {
                                        format!("None")
                                    }
                                }),
                        );
                        if skim == 0.0 {
                            model.machine_config.set_skim(None);
                        } else {
                            model.machine_config.set_skim(Some(skim));
                        }
                        ui.label("Skim defines the height above the media that the pen will rise to before high-speed travel moves between lines.");
                        ui.add_space(4.);
                    } //skim height
                    // Keepdown
                    {
                        let mut tmp_keepdown=model.machine_config.keepdown().unwrap_or(0.);
                        ui.add(
                            Slider::new(&mut tmp_keepdown, 0.0f64..=5.0f64)
                                .text(if model.machine_config.keepdown().unwrap_or(0.0) > 0.0 {
                                    "Keepdown"
                                } else {
                                    "Keepdown Disabled"
                                })
                                // .logarithmic(true)
                                .custom_formatter(|n, _| {
                                    if n > 0.0 {
                                        format!("{:1.2}mm", n)
                                    } else {
                                        format!("None")
                                    }
                                }),
                        );
                        model.machine_config.set_keepdown(if tmp_keepdown>0.0 {Some(tmp_keepdown)}else{None});



                        ui.label("Keepdown is the distance that a pen will travel between lines before lifting to the skim height. \
                            This allows the plotter to skip lift/drop cycles when you wouldn't be able to perceive the difference.\
                            Usually this is set to the diameter of the pen, because those lines would be touching anyhow.");
                        ui.add_space(4.);
                    }

                    // Feedrate
                    {
                        let mut tmp_vel = model.machine_config.feedrate();
                        ui.horizontal(|ui| {
                            ui.add(Slider::new(&mut tmp_vel, 100.0..=10000.0))
                                .labelled_by(ui.label("Feed(mm/s)").id);
                        }); //Feedrate
                        model.machine_config.set_feedrate(tmp_vel);
                        ui.label("Feedrate is the speed at which the pen moves by default. You can tune this per-pen in the pen-crib. \
                            Units are in millimeters/second, and usually 1200 or so is a good safe speed for most pens. Too high a number\
                            can result in fading or tearing.");
                        ui.add_space(4.);
                    }

                });

                ui.collapsing("Post Templates", |ui|{
                    let mut templates: IndexMap<String, String> = IndexMap::from_iter(
                        model.machine_config.get_post_template()
                            .iter()
                            .map(|(k,v)| (k.clone(), v.clone())));
                    let mut update=false;
                    for (name, mut tpl) in templates.iter_mut(){
                        ui.label(name.clone());
                        let te = TextEdit::multiline(tpl).desired_width(560.0);
                        if ui.add(te).changed(){update=true}
                    }

                    if update{
                        model.machine_config.set_post_template(&templates.iter().map(|(k,v)|(k.clone(), v.clone())).collect());

                    }
                    // The painter for the machine mockup
                    // let (painter_resp, painter) = ui.allocate_painter(vec2(390., 420.), egui::Sense::all());
                    // let prect = painter_resp.rect;
                    // let ofs = (prect.min.clone() + vec2(10., 10.)).to_vec2();

                });
            });

            // println!("Scrollarea size: {:?}", scrollarea_resp.content_size);
            ui.add_space(8.);
            // ui.separator();


            ui.allocate_ui(Vec2::new(scrollarea_resp.content_size.x, 16.), |ui|{
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        // if let CommandContext::MachineEdit(saved_machine) = &model.command_context {
                            // if let Some(machine) = saved_machine {
                            //     model.machine_config = machine.clone();
                            // }
                            //model.set_command_context(CommandContext::None);
                            model.cancel_command_context(true);
                        // }
                    }
                    if ui.button("Ok").clicked() {
                        // model.paper_modal_open = false
                        // model.set_command_context(CommandContext::None);
                        model.cancel_command_context(false);
                    }

                });

            });
        });
    });
}
