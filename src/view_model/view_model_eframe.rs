use std::process::exit;

use eframe::egui;
use egui::{Color32, Rect, pos2, vec2};
use egui::{Pos2, Vec2};
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::core::commands::ApplicationStateChangeMsg;

use super::BAPDisplayMode;
use super::BAPViewModel;

impl eframe::App for BAPViewModel {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.last_pointer_pos = ctx.pointer_hover_pos();
        if let Some(handle) = &self.join_handle {
            if handle.is_finished() {
                eprintln!("Core thread died! Bailing out.");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        self.handle_file_selector();

        loop {
            let received = if let Some(msg_in) = &self.state_in {
                match msg_in.try_recv() {
                    Ok(msg) => msg,
                    Err(_nomsg) => ApplicationStateChangeMsg::None,
                }
            } else {
                ApplicationStateChangeMsg::None
            };
            if received == ApplicationStateChangeMsg::None {
                break;
            }
            if received != ApplicationStateChangeMsg::None {
                // println!("Received: {:?}", received);
            };
            match received {
                ApplicationStateChangeMsg::Dead => {
                    exit(0);
                }
                ApplicationStateChangeMsg::Pong => {}
                ApplicationStateChangeMsg::None => {}
                ApplicationStateChangeMsg::ResetDisplay => todo!(),
                ApplicationStateChangeMsg::UpdateSourceImage {
                    image,
                    extents: (x, y, width, height),
                    rotation: _opt_rot,
                } => {
                    if let Some(handle) = &mut self.source_image_handle {
                        //println!(
                        // "Got incoming extents with image: {},{},{}w,{}h",
                        // x, y, width, height
                        // );
                        let _tmp_source_image_extents = Some(Rect::from_min_size(
                            pos2(x as f32, y as f32),
                            vec2(width as f32, height as f32),
                        ));
                        //println!(
                        //     "Incoming extents are : {:?} and known extents are: {:?}",
                        //     tmp_source_image_extents, self.source_image_extents
                        // );
                        handle.set(image, egui::TextureOptions::LINEAR);
                        self.source_image_extents = Some(Rect::from_min_size(
                            Pos2 {
                                x: x as f32,
                                y: y as f32,
                            },
                            Vec2 {
                                x: width as f32,
                                y: height as f32,
                            },
                        ));
                    }
                    // self.dirty = false;
                    self.timeout_for_source_image = None;
                }
                ApplicationStateChangeMsg::UpdateMachineConfig(_machine_config) => todo!(),
                ApplicationStateChangeMsg::ProgressMessage {
                    message,
                    percentage,
                } => {
                    //println!("Got a progress message.");
                    self.progress = Some((message, percentage));
                }
                ApplicationStateChangeMsg::SourceChanged { extents } => {
                    // self.waiting_for_source_image=true;
                    self.source_image_extents = Some(Rect::from_min_size(
                        pos2(extents.0 as f32, extents.1 as f32),
                        vec2(extents.2 as f32, extents.3 as f32),
                    ));
                    self.request_new_source_image();
                }
                ApplicationStateChangeMsg::PlotterState(plotter_state) => {
                    self.plotter_state = plotter_state
                    // self.handle_plotter_response(plotter_response);
                }
                ApplicationStateChangeMsg::FoundPorts(items) => {
                    // self.serial_ports = items
                    let old_ports = self.serial_ports.clone();
                    self.serial_ports = items;
                    for port in &old_ports {
                        if !self.serial_ports.contains(&port) {
                            self.queued_toasts.push_back(Toast {
                                kind: ToastKind::Info,
                                text: format!("Serial port {} removed.", &port).into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(5.)
                                    .show_progress(true),
                                ..Default::default()
                            })
                        }
                    }
                    for port in &self.serial_ports {
                        if !old_ports.contains(&port) {
                            self.queued_toasts.push_back(Toast {
                                kind: ToastKind::Info,
                                text: format!("Serial port {} discovered.", &port).into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(5.)
                                    .show_progress(true),
                                ..Default::default()
                            })
                        }
                    }
                }
                ApplicationStateChangeMsg::PostComplete(lines) => {
                    self.queued_toasts.push_back(Toast {
                        kind: ToastKind::Success,
                        text: format!("Post completed with {} GCODE lines.", &lines).into(),
                        options: ToastOptions::default().duration_in_seconds(15.),
                        ..Default::default()
                    });
                    // self.display_mode = BAPDisplayMode::Plot;
                    self.set_display_mode(BAPDisplayMode::Plot);
                }
                ApplicationStateChangeMsg::Error(msg) => self.queued_toasts.push_back(Toast {
                    kind: ToastKind::Error,
                    text: msg.into(),
                    options: ToastOptions::default().duration_in_seconds(15.),
                    ..Default::default()
                }),
                ApplicationStateChangeMsg::PlotterResponse(plotter_response) => {
                    self.handle_plotter_response(plotter_response);
                }
                ApplicationStateChangeMsg::PlotPreviewChanged { extents: _ } => todo!(),
                ApplicationStateChangeMsg::TransformPreviewImage {
                    image: _,
                    extents: _,
                } => todo!(),
                ApplicationStateChangeMsg::OriginChanged(x, y) => {
                    // println!("Got new origin: {},{}", x, y);
                    self.set_origin(pos2(x as f32, y as f32), false)
                }
                ApplicationStateChangeMsg::UndoAvailable(is_avail) => {
                    // println!("Undo available? {}", is_avail);
                    self.undo_available = is_avail
                }
                ApplicationStateChangeMsg::PaperChanged(paper) => {
                    // println!("Got a new paper of: {:?}", paper);
                    self.set_paper_color(
                        &Color32::from_rgb(
                            (255.0 * paper.rgb.0 as f32).min(255.).max(0.) as u8,
                            (255.0 * paper.rgb.1 as f32).min(255.).max(0.) as u8,
                            (255.0 * paper.rgb.2 as f32).min(255.).max(0.) as u8,
                        ),
                        false,
                    );
                    self.set_paper_orientation(&paper.orientation, false);
                    self.set_paper_size(&paper.size, false);
                }
                ApplicationStateChangeMsg::PatchViewModel(patch) => self.patch(patch),
                ApplicationStateChangeMsg::Picked(id) => {
                    println!("Picked {:?}", id);
                    if self.display_mode() == BAPDisplayMode::SVG {
                        self.request_new_source_image();
                    }
                }
            }
        }

        crate::ui::update_ui(self, ctx, frame);

        // This is how to go into continuous mode - uncomment this to see example of continuous mode
        // ctx.request_repaint();
    }
}
