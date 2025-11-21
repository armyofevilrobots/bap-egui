use std::collections::BTreeSet;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime};

use aoer_plotty_rs::context::pgf_file::PlotGeometry;
use egui::{ColorImage, Context};

use super::commands::{ApplicationStateChangeMsg, ViewCommand};
use gcode::GCode;
use geo::{Geometry, MultiLineString};

use super::ApplicationCore;
use super::post;
use super::project::Project;
use super::sender::{PlotterCommand, PlotterConnection, PlotterResponse, PlotterState};
use super::serial;
use super::{PICKED_ROTATE_TIME, render_plot_preview, render_source};
use crate::view_model::view_model_patch::ViewModelPatch;

impl ApplicationCore {
    pub fn run(&mut self) {
        // First, send the default image to display:
        let mut last_sent_plotter_running_progress = Instant::now() - Duration::from_secs(60); // Just pretend it's been a while.

        while !self.shutdown {
            match self
                .view_command_in
                .recv_timeout(Duration::from_millis(100))
            {
                Err(_err) => (),
                Ok(msg) => match msg {
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
                        self.project.set_origin(&Some((x,y)));
                    },
                    ViewCommand::SetClipBoundary {
                        min: _min,
                        max: _max,
                    } => todo!(),
                    ViewCommand::RotateSource {
                        center,
                        degrees,
                    } => {
                        self.checkpoint();
                        // println!("Rotating source data around {},{} by {} degrees", center.0, center.1, degrees);
                        // println!("PRE EXTENTS: {:?}", self.project.extents());
                        self.project.rotate_geometry_around_point_mut(center, degrees);
                        // println!("POST EXTENTS: {:?}", self.project.extents());
                        self.rebuild_after_content_change();
                        // println!("POST RESEND EXTENTS: {:?}", self.project.extents());

                    },
                    ViewCommand::Post => {
                        let prep_sender = match post::post(&self.project){
                            Ok(program) => {
                                self.program = Some(program.clone());
                                self.project.set_program(Some(Box::new(program.clone())));
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::PostComplete(
                                        self.program.as_ref().unwrap().len()))
                                    .expect("Failed to send state change up to VM. Dead view?");
                                self.progress=(0, 0, 0); // Reset progress
                                self.gcode = Some(gcode::parse(self.program.clone().unwrap().join("\n").as_str()).collect());
                                self.ctx.request_repaint();
                                true
                            },
                            Err(err) => {
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Error(
                                        format!("Failed to post due to {:?}.", err).into())).expect("Failed to send error to viewmodel.");

                                self.ctx.request_repaint();
                                false
                            }
                        };
                        if prep_sender{
                            let _resp = self.plot_sender.send(PlotterCommand::Program(Box::new(self.program.as_ref().unwrap().clone())));
                        }
                    },
                    ViewCommand::StartPlot => {
                        self.plot_sender.send(PlotterCommand::Run).expect("Failed to run plot.");
                    },
                    ViewCommand::PausePlot => {
                        self.plot_sender.send(PlotterCommand::Stop).expect("Failed to pause plot.");
                    },
                    ViewCommand::CancelPlot => {
                        self.plot_sender.send(PlotterCommand::Reset).expect("Failed to reset plotter.");
                    },
                    ViewCommand::None => todo!(),
                    ViewCommand::Quit => {
                        self.shutdown = true;
                        self.plot_sender.send(PlotterCommand::Shutdown).expect("Failed to shut down plotter worker.");
                    },
                    ViewCommand::UpdateMachineConfig(_machine_config) => todo!(),
                    ViewCommand::SendCommand(cmd)=>{
                        self.plot_sender
                            .send(PlotterCommand::Command(cmd))
                            .unwrap_or_else(|err|{
                                self.shutdown = true;
                                eprintln!("Plot sender is dead and I cannot connect. Dying: {:?}", err);
                                self.state_change_out.send(ApplicationStateChangeMsg::Dead)
                                    .expect("Can't even notify ViewModel I'm dead?! YOLO and apparently not for very long.");
                            });
                    }
                    ViewCommand::ConnectPlotter(port_path) => self
                        .plot_sender
                        .send(PlotterCommand::Connect(port_path))
                        .unwrap_or_else(|err| {
                            self.shutdown = true;
                            eprintln!("Plot sender is dead and I cannot connect. Dying: {:?}", err);
                            self.state_change_out.send(ApplicationStateChangeMsg::Dead)
                                .expect("Can't even notify ViewModel I'm dead?! YOLO and apparently not for very long.");
                        }),
                    ViewCommand::DisconnectPlotter => self
                        .plot_sender
                        .send(PlotterCommand::Disconnect)
                        .unwrap_or_else(|err| {
                            self.shutdown = true;
                            eprintln!("Plot sender is dead and I cannot connect. Dying: {:?}", err);
                            self.state_change_out.send(ApplicationStateChangeMsg::Dead)
                                .expect("Can't even notify ViewModel I'm dead?! YOLO and apparently not for very long.");
                            self.ctx.request_repaint();
                        }),
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

                    },
                    ViewCommand::RequestPlotPreviewImage { extents, resolution } => {
                        let empty = Vec::new();
                        let _gcode = match &self.gcode{
                            Some(gcode)=>gcode,
                            None=>&empty,
                        };
                        let cimg = match render_plot_preview(
                            &self.project,
                            // gcode,
                            extents,
                            (self.progress.0, self.progress.1),
                            resolution,
                            &self.state_change_out,
                            &self.cancel_render
                        ) {
                            Ok(cimg) => Some(cimg),
                            Err(_) => None,
                        };

                        if let Some(cimg) = cimg{
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
                                    eprintln!("Failed to send message from bap core. Shutting down.");
                                });

                        }
                        self.ctx.request_repaint();

                    },
                    ViewCommand::ApplyPens(pen_details) => {
                        // println!("GOT NEW PENS: {:?}", pen_details);
                        self.checkpoint();
                        self.project.update_pen_details(&pen_details);
                    },
                    ViewCommand::Undo => self.undo(),
                    ViewCommand::SetPaper(paper) => {
                        self.checkpoint();
                        // println!("Setting project paper to: {:?}", paper);
                        self.project.paper = paper
                    },
                    ViewCommand::LoadProject(path_buf) => {
                        while let Ok(_) = self.cancel_render.try_recv() {
                            eprintln!("Draining excessive cancels.");
                        }
                        self.checkpoint();
                        self.project = match Project::load_from_path(&path_buf){
                            Ok(prj) => {
                                prj
                            },
                            Err(err) => {
                                // println!("Failed to load due to {:?}", &err);
                                self.state_change_out
                                    .send(ApplicationStateChangeMsg::Error(
                                        format!("Failed to load project file: {:?}", err).into())).expect("Failed to send error to viewmodel.");
                                self.ctx.request_repaint();
                                self.history.pop().unwrap()
                            },
                        };
                        self.state_change_out
                            .send(ApplicationStateChangeMsg::PatchViewModel(
                                ViewModelPatch::from(self.project.clone())))
                            .expect("Failed to send error to viewmodel.");
                        self.ctx.request_repaint();
                        self.rebuild_after_content_change();

                    },
                    ViewCommand::SaveProject(path_buf) => {
                        if let Ok(path) = match path_buf.clone() {
                            Some(path)=> self.project.save_to_path(&path),
                            None => self.project.save(),
                        } {
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::ProgressMessage{
                                    message: format!("Saved to {}", path.file_name().unwrap_or(path.as_os_str()).to_string_lossy()).into(),
                                    percentage: 100,
                                }).expect("Failed to send state change progress 100%");
                            self.ctx.request_repaint();
                        } else {
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::Error(
                                    format!("Failed to save project file: {:?}", path_buf).into())).expect("Failed to send error to viewmodel.");
                            self.ctx.request_repaint();

                        }
                    }
                    ViewCommand::LoadPGF(path_buf) => {
                        while let Ok(_) = self.cancel_render.try_recv() {
                            eprintln!("Draining excessive cancels.");
                        }
                        self.checkpoint();
                        self.project.load_pgf(&path_buf).unwrap_or_else(|err|{
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::Error(
                                    format!("Failed to load project file: {:?} due to {}", path_buf, err).into())).expect("Failed to send error to viewmodel.");
                            self.ctx.request_repaint();

                        });
                        self.rebuild_after_content_change();
                    },
                    ViewCommand::ResetProject => {
                        while let Ok(_) = self.cancel_render.try_recv() {
                            eprintln!("Draining excessive cancels.");
                        }
                        self.checkpoint();
                        self.project = Project::default();
                        self.state_change_out
                            .send(ApplicationStateChangeMsg::PatchViewModel(
                                ViewModelPatch::from(self.project.clone())))
                            .expect("Failed to send error to viewmodel.");
                        self.rebuild_after_content_change();
                    },
                    ViewCommand::TryPickAt(x, y) => {
                        let picked = self.try_pick(x,y);
                        if let Some(id) = picked {
                            // println!("Got a geo pick at {}", geo.id);
                            //
                            if self.picked.is_none(){
                                self.picked = Some(BTreeSet::new());
                            }

                            self.picked.as_mut().unwrap().insert(id as u32);
                            self.state_change_out.send(ApplicationStateChangeMsg::Picked(Some(self.picked.as_ref().unwrap()
                                .iter()
                                .map(|i| *i as usize)
                                .collect::<Vec<usize>>()))).expect("OMFG ViewModel is borked sending pick id");
                        }else{
                            self.state_change_out.send(ApplicationStateChangeMsg::Picked(None)).expect("OMFG ViewModel is borked sending pick id");
                            self.picked = None
                        }
                        self.last_rendered=Instant::now()+Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                        self.ctx.request_repaint();
                    },
                    ViewCommand::ClearPick => {
                        self.state_change_out.send(ApplicationStateChangeMsg::Picked(None)).expect("OMFG ViewModel is borked sending pick id");
                        self.picked = None
                    },
                    ViewCommand::UnGroup => {
                        // println!("Would have ungrouped {:?}", self.picked);
                        if let Some(picked) = &self.picked{
                            // Make copies of all the stuff we're breaking up.
                            let geo_items: Vec<PlotGeometry> = picked
                                .iter()
                                .filter_map(|idx| self.project.geometry.get(*idx as usize))
                                .map(|item| item.clone())
                                .collect();
                            // Then remove them from the geometry list. We reverse the order
                            // to prevent shrinking and removing the wrong shit.
                            for idx in picked.iter().rev(){
                                self.project.geometry.remove(*idx as usize);
                            }

                            for geo in geo_items{
                                let geo = geo.clone();
                                match geo.geometry{
                                    Geometry::MultiLineString(mls)=>{
                                        for linestring in mls.0{
                                            self.project.geometry.push(
                                                PlotGeometry{
                                                    geometry: Geometry::MultiLineString(MultiLineString::new(vec![linestring])),
                                                    id: u32::MAX as u64,
                                                    stroke: geo.stroke.clone(),
                                                    keepdown_strategy: geo.keepdown_strategy.clone(),
                                                });
                                        }
                                    }
                                    _ =>()
                                }
                            }
                            self.state_change_out.send(ApplicationStateChangeMsg::Picked(None)).expect("OMFG ViewModel is borked sending pick id");
                            self.picked = None;
                            self.pick_image = None;
                            self.rebuild_after_content_change();
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PatchViewModel(
                                    ViewModelPatch::from(self.project.clone())))
                                .expect("Failed to send error to viewmodel.");
                            self.ctx.request_repaint();

                        }

                    },
                    ViewCommand::Group => todo!(),
                    ViewCommand::TogglePickAt(x, y) => {
                        let picked = self.try_pick(x,y);
                        if let Some(id) = picked {
                            //
                            if self.picked.is_none(){
                                self.picked = Some(BTreeSet::new());
                            }

                            if self.picked.as_mut().unwrap().contains(&id){
                                self.picked.as_mut().unwrap().remove(&id);
                            }else{
                                self.picked.as_mut().unwrap().insert(id.clone());
                            }
                            self.state_change_out.send(ApplicationStateChangeMsg::Picked(Some(self.picked.as_ref().unwrap()
                                .iter()
                                .map(|i| *i as usize)
                                .collect::<Vec<usize>>()))).expect("OMFG ViewModel is borked sending pick id");
                        }else{
                            // self.state_change_out.send(ApplicationStateChangeMsg::Picked(None)).expect("OMFG ViewModel is borked sending pick id");
                            // self.picked = None
                        }
                        self.last_rendered=Instant::now()+Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                        self.ctx.request_repaint();

                    },
                },
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
                    .expect("OMFG ViewModel is borked sending pick id");
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
                    match &response {
                        PlotterResponse::Ok(_plotter_command, _) => (),
                        PlotterResponse::Err(_plotter_command, _) => {}
                        PlotterResponse::State(plotter_state) => {
                            if let PlotterState::Running(line, of, _something) = plotter_state {
                                self.progress =
                                    (*line as usize, *of as usize, *_something as usize);
                            };
                            self.state = plotter_state.clone();
                        }
                        PlotterResponse::Loaded(_msg) => {
                            // println!("MSG ON PLOTTER REPONSE LOADED: {:?}", msg);
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PlotterResponse(
                                    PlotterResponse::Loaded("OK!".into()),
                                ))
                                .expect("Failed to send to ViewModel. Dead conn?");

                            self.ctx.request_repaint();
                        }
                    }
                    if let PlotterResponse::State(PlotterState::Running(line, of, other)) =
                        &response
                    {
                        if Instant::now() - last_sent_plotter_running_progress
                            > Duration::from_secs(1)
                        {
                            // println!("Sending progress running stanza.");
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::PlotterResponse(
                                    PlotterResponse::State(PlotterState::Running(
                                        *line, *of, *other,
                                    )),
                                ))
                                .expect("Failed to send to ViewModel. Dead conn?");
                            last_sent_plotter_running_progress = Instant::now();
                        }
                    } else {
                        self.state_change_out
                            .send(ApplicationStateChangeMsg::PlotterResponse(response.clone()))
                            .expect("Failed to send to ViewModel. Dead conn?");
                    }
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
                _ => false,
            };
            // Housekeeping stuffs.
            if self.last_serial_scan + Duration::from_secs(10) < Instant::now() && can_scan {
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
