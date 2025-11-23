use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use super::commands::{ApplicationStateChangeMsg, ViewCommand};

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
                        ViewCommand::Ping => {
                            // println!("PING!");
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::Pong)
                                .unwrap_or_else(|_op| self.shutdown = true);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::RequestSourceImage {
                            // extents,
                            zoom,
                            // resolution,
                            rotation,
                        } => {
                            self.handle_request_source_image(zoom, rotation);
                        }
                        ViewCommand::ImportSVG(path_buf) => {
                            self.checkpoint();
                            self.project.import_svg(&path_buf, true);
                            self.rebuild_after_content_change();
                        }
                        ViewCommand::SetOrigin(x, y) => {
                            self.checkpoint();
                            self.project.set_origin(&Some((x, y)));
                        }
                        ViewCommand::SetClipBoundary {
                            min: _min,
                            max: _max,
                        } => todo!(),
                        ViewCommand::RotateSource { center, degrees } => {
                            self.checkpoint();
                            // println!("Rotating source data around {},{} by {} degrees", center.0, center.1, degrees);
                            // println!("PRE EXTENTS: {:?}", self.project.extents());
                            self.project
                                .rotate_geometry_around_point_mut(center, degrees);
                            // println!("POST EXTENTS: {:?}", self.project.extents());
                            self.rebuild_after_content_change();
                            // println!("POST RESEND EXTENTS: {:?}", self.project.extents());
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
                        ViewCommand::UpdateMachineConfig(_machine_config) => todo!(),
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
                            // println!("Setting project paper to: {:?}", paper);
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
                            let picked = self.try_pick(x, y);
                            if let Some(id) = picked {
                                if self.picked.is_none() {
                                    self.picked = Some(BTreeSet::new());
                                }

                                self.picked.as_mut().unwrap().insert(id as u32);
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Picked(Some(
                                        self.picked
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .map(|i| *i as usize)
                                            .collect::<Vec<usize>>(),
                                    )))
                                    .expect("OMFG ViewModel is borked sending pick id");
                            } else {
                                self.state_change_out
                                .send(ApplicationStateChangeMsg::Picked(None))
                                .unwrap_or_else(|_err| if self.shutdown {
                                    eprintln!("Cannot update pick image while shutting down...")
                                }else{
                                    eprintln!("Cannot send pick image because ViewModel has hung up.")
                                });
                                // .expect("OMFG ViewModel is borked sending pick id");
                            }
                            self.last_rendered = Instant::now()
                                + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::TryPickAt(x, y) => {
                            let picked = self.try_pick(x, y);
                            if let Some(id) = picked {
                                if self.picked.is_none() {
                                    self.picked = Some(BTreeSet::new());
                                }

                                self.picked.as_mut().unwrap().insert(id as u32);
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Picked(Some(
                                        self.picked
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .map(|i| *i as usize)
                                            .collect::<Vec<usize>>(),
                                    )))
                                    .expect("OMFG ViewModel is borked sending pick id");
                            } else {
                                self.state_change_out
                                .send(ApplicationStateChangeMsg::Picked(None))
                                .unwrap_or_else(|_err| if self.shutdown {
                                    eprintln!("Cannot update pick image while shutting down...")
                                }else{
                                    eprintln!("Cannot send pick image because ViewModel has hung up.")
                                });
                                // .expect("OMFG ViewModel is borked sending pick id");
                                self.picked = None
                            }
                            self.last_rendered = Instant::now()
                                + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::ClearPick => {
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::Picked(None))
                                .expect("OMFG ViewModel is borked sending pick id");
                            self.picked = None
                        }
                        ViewCommand::UnGroup => {
                            self.checkpoint();
                            self.apply_ungroup();
                        }
                        ViewCommand::Group => {
                            self.checkpoint();
                            self.apply_group();
                        }
                        ViewCommand::TogglePickAt(x, y) => {
                            let picked = self.try_pick(x, y);
                            if let Some(id) = picked {
                                //
                                if self.picked.is_none() {
                                    self.picked = Some(BTreeSet::new());
                                }

                                if self.picked.as_mut().unwrap().contains(&id) {
                                    self.picked.as_mut().unwrap().remove(&id);
                                } else {
                                    self.picked.as_mut().unwrap().insert(id.clone());
                                }
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Picked(Some(
                                        self.picked
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .map(|i| *i as usize)
                                            .collect::<Vec<usize>>(),
                                    )))
                                    .expect("OMFG ViewModel is borked sending pick id");
                            }
                            self.last_rendered = Instant::now()
                                + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::SelectAll => {
                            self.picked = Some(BTreeSet::from_iter(
                                (0..self.project.geometry.len()).map(|i| i as u32),
                            ));
                            self.last_rendered = Instant::now()
                                + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                            self.ctx.request_repaint();
                        }
                        ViewCommand::ApplyPenToSelection(tool_id) => {
                            self.checkpoint();
                            self.apply_pen_to_selection(tool_id);
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
