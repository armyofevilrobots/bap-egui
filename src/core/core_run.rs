use super::commands::{ApplicationStateChangeMsg, ViewCommand};
use std::time::{Duration, Instant};

use super::ApplicationCore;
use super::project::Project;
use super::sender::{PlotterCommand, PlotterState};
use super::serial;
use super::{PICKED_ROTATE_TIME, render_plot_preview};
use crate::view_model::view_model_patch::ViewModelPatch;

impl ApplicationCore {
    pub fn run(&mut self) {
        // First, send the default image to display:
        let mut last_sent_plotter_running_progress = Instant::now() - Duration::from_secs(60); // Just pretend it's been a while.
        self.state_change_out
            .send(ApplicationStateChangeMsg::NotifyConfig(self.config.clone()))
            .expect("Failed to send config to viewmodel at start. Bailing.");

        while !self.shutdown {
            match self
                .view_command_in
                .recv_timeout(Duration::from_millis(100))
            {
                Err(_err) => (),
                Ok(msg) => {
                    match msg {
                        ViewCommand::ScaleAround { center, factor } => {
                            // println!("Scaling geo around {:?} by {}", center, factor);
                            self.checkpoint();
                            self.project.scale_geometry_around_point_mut(
                                center,
                                factor,
                                &self.picked,
                            );
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::Ping => {
                            // println!("PING!");
                            self.yolo_app_state_change(ApplicationStateChangeMsg::Pong);
                        }
                        ViewCommand::RequestSourceImage {
                            // extents,
                            zoom,
                            // resolution,
                            rotation,
                            translation,
                            scale_around,
                        } => {
                            self.handle_request_source_image(
                                zoom,
                                rotation,
                                translation,
                                scale_around,
                            );
                        }
                        ViewCommand::ImportSVG(path_buf) => {
                            self.checkpoint();
                            self.project.import_svg(&path_buf, true);
                            self.yolo_app_state_change(ApplicationStateChangeMsg::PatchViewModel(
                                ViewModelPatch::from(self.project.clone()),
                            ));
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::SetOrigin(x, y) => {
                            self.checkpoint();
                            self.project.set_origin(&Some((x, y)));
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::SetClipBoundary {
                            min: _min,
                            max: _max,
                        } => todo!(),
                        ViewCommand::RotateSource { center, degrees } => {
                            self.checkpoint();
                            self.project.rotate_geometry_around_point_mut(
                                center,
                                degrees,
                                &self.picked,
                            );
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::Post => {
                            self.handle_post();
                        }
                        ViewCommand::StartPlot => {
                            self.yolo_send_plotter_cmd(PlotterCommand::Run);
                        }
                        ViewCommand::PausePlot => {
                            self.yolo_send_plotter_cmd(PlotterCommand::Stop);
                        }
                        ViewCommand::CancelPlot => {
                            self.yolo_send_plotter_cmd(PlotterCommand::Reset);
                        }
                        ViewCommand::None => todo!(),
                        ViewCommand::Quit => {
                            self.shutdown = true;
                            self.yolo_send_plotter_cmd(PlotterCommand::Shutdown);
                        }
                        ViewCommand::UpdateMachineConfig(machine_config) => {
                            self.project.set_machine(Some(machine_config));
                        }
                        ViewCommand::SendCommand(cmd) => {
                            self.yolo_send_plotter_cmd(PlotterCommand::Command(cmd));
                        }
                        ViewCommand::ConnectPlotter(port_path) => {
                            self.yolo_send_plotter_cmd(PlotterCommand::Connect(port_path))
                        }
                        ViewCommand::DisconnectPlotter => {
                            self.yolo_send_plotter_cmd(PlotterCommand::Disconnect)
                        }
                        ViewCommand::PenUp => {
                            self.set_pen_position(false);
                        }
                        ViewCommand::PenDown => {
                            self.set_pen_position(true);
                        }
                        ViewCommand::Scale(factor) => {
                            //TODO: MIgrate this into Project.
                            self.checkpoint();
                            self.project.scale_by_factor(factor);
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::RequestPlotPreviewImage {
                            extents,
                            resolution,
                        } => {
                            let empty = Vec::new();
                            let _gcode = match &self.gcode {
                                Some(gcode) => gcode,
                                None => &empty,
                            };
                            let cimg = match render_plot_preview(
                                &self.project,
                                // gcode,
                                extents,
                                (self.progress.0, self.progress.1),
                                resolution,
                                &self.state_change_out,
                                &self.cancel_render,
                            ) {
                                Ok(cimg) => Some(cimg),
                                Err(_) => None,
                            };

                            if let Some(cimg) = cimg {
                                self.last_render = Some((cimg.clone(), extents));
                                self.last_rendered = Instant::now();
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::UpdateSourceImage {
                                        image: cimg,
                                        extents,
                                        rotation: None,
                                    })
                                    .unwrap_or_else(|_err| {
                                        self.shutdown = true;
                                        eprintln!(
                                            "Failed to send message from bap core. Shutting down."
                                        );
                                    });
                            }
                            self.ctx.request_repaint();
                        }
                        ViewCommand::ApplyPens(pen_details) => {
                            // println!("GOT NEW PENS: {:?}", pen_details);
                            self.checkpoint();
                            self.project.update_pen_details(&pen_details);
                        }
                        ViewCommand::Undo => self.undo(),
                        ViewCommand::SetPaper(paper) => {
                            self.checkpoint();
                            self.project.paper = paper
                        }
                        ViewCommand::LoadProject(path_buf) => {
                            while let Ok(_) = self.cancel_render.try_recv() {
                                eprintln!("Draining excessive cancels.");
                            }
                            self.checkpoint();
                            self.project = match Project::load_from_path(&path_buf) {
                                Ok(prj) => prj,
                                Err(err) => {
                                    // println!("Failed to load due to {:?}", &err);
                                    self.state_change_out
                                        .send(ApplicationStateChangeMsg::Error(
                                            format!("Failed to load project file: {:?}", err)
                                                .into(),
                                        ))
                                        .expect("Failed to send error to viewmodel.");
                                    self.ctx.request_repaint();
                                    self.history.pop().unwrap()
                                }
                            };
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PatchViewModel(
                                    ViewModelPatch::from(self.project.clone()),
                                ))
                                .expect("Failed to send error to viewmodel.");
                            self.ctx.request_repaint();
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::SaveProject(path_buf) => {
                            if let Ok(path) = match path_buf.clone() {
                                Some(path) => self.project.save_to_path(&path),
                                None => self.project.save(),
                            } {
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::ProgressMessage {
                                        message: format!(
                                            "Saved to {}",
                                            path.file_name()
                                                .unwrap_or(path.as_os_str())
                                                .to_string_lossy()
                                        )
                                        .into(),
                                        percentage: 100,
                                    })
                                    .expect("Failed to send state change progress 100%");
                                self.ctx.request_repaint();
                            } else {
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Error(
                                        format!("Failed to save project file: {:?}", path_buf)
                                            .into(),
                                    ))
                                    .expect("Failed to send error to viewmodel.");
                                self.ctx.request_repaint();
                            }
                        }
                        ViewCommand::LoadPGF(path_buf) => {
                            while let Ok(_) = self.cancel_render.try_recv() {
                                eprintln!("Draining excessive cancels.");
                            }
                            self.checkpoint();
                            self.project.load_pgf(&path_buf).unwrap_or_else(|err| {
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Error(
                                        format!(
                                            "Failed to load project file: {:?} due to {}",
                                            path_buf, err
                                        )
                                        .into(),
                                    ))
                                    .expect("Failed to send error to viewmodel.");
                                self.ctx.request_repaint();
                            });
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::ResetProject => {
                            while let Ok(_) = self.cancel_render.try_recv() {
                                eprintln!("Draining excessive cancels.");
                            }
                            self.checkpoint();
                            self.project = Project::default();
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PatchViewModel(
                                    ViewModelPatch::from(self.project.clone()),
                                ))
                                .expect("Failed to send error to viewmodel.");
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::AddPickAt(x, y) => {
                            self.add_pick_at(x, y);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::TryPickAt(x, y) => {
                            self.try_pick_at(x, y);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::ClearPick => {
                            self.clear_pick();
                        }
                        ViewCommand::TogglePickAt(x, y) => {
                            self.toggle_pick_at(x, y);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::SelectAll => {
                            self.select_all();
                            self.ctx.request_repaint();
                        }
                        ViewCommand::PickByColorAt(x, y) => {
                            self.select_by_color_at(x, y);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::ApplyPenToSelection(tool_id) => {
                            self.checkpoint();
                            self.apply_pen_to_selection(tool_id);
                        }
                        ViewCommand::UnGroup => {
                            self.checkpoint();
                            self.apply_ungroup();
                        }
                        ViewCommand::Group => {
                            self.checkpoint();
                            self.apply_group();
                        }
                        ViewCommand::DeleteSelection => {
                            self.checkpoint();
                            self.delete_selection();
                            self.ctx.request_repaint();
                        }
                        ViewCommand::UpdateConfig(app_config) => {
                            self.config = app_config;
                            self.config.save_to(None).unwrap_or_else(|err| {
                                self.yolo_app_state_change(ApplicationStateChangeMsg::Error(
                                    format!("Failed to save config to disk! Err:{}", err)
                                        .to_string(),
                                ))
                            });
                        }
                        ViewCommand::Translate(x, y) => {
                            self.checkpoint();
                            self.project.translate_geometry_mut((x, y), &self.picked);
                            let new_ext = self.project.extents().clone();
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PatchViewModel(ViewModelPatch {
                                    extents: Some((
                                        new_ext.min().x,
                                        new_ext.min().y,
                                        new_ext.width(),
                                        new_ext.height(),
                                    )),
                                    ..Default::default()
                                }))
                                .expect("Failed to send error to viewmodel.");
                            self.ctx.request_repaint();
                        }
                        ViewCommand::LoadMachineConfig(path_buf) => {
                            self.project.load_machine(&path_buf).unwrap_or_else(|err| {
                                self.yolo_app_state_change(ApplicationStateChangeMsg::Error(
                                    format!(
                                        "Failed to load machine config from {}! Err:{}",
                                        path_buf.as_os_str().to_string_lossy(),
                                        err
                                    )
                                    .to_string(),
                                ))
                            });
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PatchViewModel(ViewModelPatch {
                                    machine_config: Some(self.project.machine().clone()),
                                    ..Default::default()
                                }))
                                .expect("Failed to send error to viewmodel.");
                        }
                        ViewCommand::SaveMachineConfig(path_buf) => {
                            self.project.save_machine(&path_buf).unwrap_or_else(|err| {
                                eprintln!("Failed to save: {:?}", err);
                                self.yolo_app_state_change(ApplicationStateChangeMsg::Error(
                                    format!(
                                        "Failed to save machine config to {}! Err:{}",
                                        path_buf.as_os_str().to_string_lossy(),
                                        err
                                    )
                                    .to_string(),
                                ))
                            });
                        } // ViewCommand::ReNumberGeometry(pen_map) => {
                          //     todo!();
                          // }
                    }
                }
            }

            if let Some(pickset) = &self.picked
                && (Instant::now() - self.last_rendered)
                    > Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64)
            {
                // println!("Refreshing geo pick image for {}", id);
                self.state_change_out
                    .send(ApplicationStateChangeMsg::Picked(Some(
                        pickset.iter().map(|i| *i as usize).collect(),
                    )))
                    .unwrap_or_else(|_err| {
                        if self.shutdown {
                            eprintln!("Cannot update pick image while shutting down...")
                        } else {
                            eprintln!("Cannot send pick image because ViewModel has hung up.")
                        }
                    });
                self.last_rendered = Instant::now() + Duration::from_secs(10); // Prevent spamming by putting this WAY in the future
                self.ctx.request_repaint();
            }

            // Also, we need to check for plotter responses...
            let now = Instant::now();
            loop {
                if Instant::now() - now > Duration::from_secs(1) {
                    // println!("Breaking plotter service loop to service UI requests.");
                    break;
                }; // Don't sit here forever reading responses.
                if let Ok(response) = self.plot_receiver.recv_timeout(Duration::from_millis(100)) {
                    // println!("Sending response: {:?}", &response);
                    self.handle_plotter_response(response, &mut last_sent_plotter_running_progress);
                } else {
                    // println!("Breaking plotter service loop because no messages available.");
                    break;
                }
                self.ctx.request_repaint();
            }

            let can_scan: bool = match self.state {
                PlotterState::Ready => true,
                PlotterState::Dead => true,
                PlotterState::Failed(_) => true,
                PlotterState::Paused(_, _, _) => true,
                PlotterState::Disconnected => true,
                PlotterState::Connecting(_) => false,
                PlotterState::Running(_, _, _) => false,
                PlotterState::Busy => false,
                PlotterState::Terminating => false,
            };
            // Housekeeping stuffs.
            // eprintln!("CAN SCAN {} - state {:?}", can_scan, self.state);
            if self.last_serial_scan + Duration::from_secs(10) < Instant::now() && can_scan {
                // eprintln!("SCAN");
                self.state_change_out
                    .send(ApplicationStateChangeMsg::FoundPorts(serial::scan_ports()))
                    .expect("Failed to send serial port update. Dead ViewModel?");
                self.last_serial_scan = Instant::now();
                self.ctx.request_repaint();
            }
        }

        self.state_change_out
            .send(ApplicationStateChangeMsg::Dead)
            .expect("Failed to send shutdown status back to viewmodel.");
    }
}
