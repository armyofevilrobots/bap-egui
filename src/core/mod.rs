use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use egui::{ColorImage, Context};

pub(crate) mod commands;
pub(crate) mod machine;
pub(crate) mod post;
pub(crate) mod project;
pub(crate) mod render_plot;
pub(crate) mod render_source;
pub(crate) mod serial;

use commands::{ApplicationStateChangeMsg, ViewCommand};
use gcode::GCode;
use nalgebra::{Affine2, Matrix3, Scale2, Transform2};
use tera::Context as TeraContext;

use crate::core::project::Project;
use crate::core::render_plot::render_plot_preview;
use crate::sender::{PlotterCommand, PlotterConnection, PlotterResponse, PlotterState};
use machine::MachineConfig;

/// The actual application core that does shit.
///
///

#[derive(Debug)]
pub struct ApplicationCore {
    view_command_in: Receiver<ViewCommand>,
    state_change_out: Sender<ApplicationStateChangeMsg>,
    cancel_render: Receiver<()>,
    shutdown: bool,
    project: Project,
    machine: MachineConfig,
    last_render: Option<ColorImage>,
    ctx: Context, // Just a repaint context, not used for ANYTHING else.
    plot_sender: Sender<PlotterCommand>,
    plot_receiver: Receiver<PlotterResponse>,
    last_serial_scan: Instant,
    program: Option<Vec<String>>,
    gcode: Option<Vec<GCode>>,
    progress: (usize, usize, usize),
    state: PlotterState,
}

impl ApplicationCore {
    pub fn new(
        ctx: Context,
    ) -> (
        ApplicationCore,
        Sender<ViewCommand>,
        Receiver<ApplicationStateChangeMsg>,
        Sender<()>,
    ) {
        let (vm_to_app, app_from_vm) = mpsc::channel::<ViewCommand>();
        let (app_to_vm, vm_from_app) = mpsc::channel::<ApplicationStateChangeMsg>();
        let (cancel_render_sender, cancel_render_receiver) = mpsc::channel::<()>();
        // let (app_to_plotter, plotter_from_app) = mpsc::channel::<PlotterCommand>();
        // let (plotter_to_app, app_from_plotter) = mpsc::channel::<PlotterResponse>();
        let (app_to_plotter, plotter_to_app) =
            PlotterConnection::spawn().expect("Failed to create PlotterConnection worker.");
        app_to_vm
            .send(ApplicationStateChangeMsg::FoundPorts(serial::scan_ports()))
            .expect("Failed to send serial port list up to view.");
        let core = ApplicationCore {
            view_command_in: app_from_vm,
            state_change_out: app_to_vm,
            shutdown: false,
            project: Project::new(),
            machine: MachineConfig::default(),
            plot_receiver: plotter_to_app,
            plot_sender: app_to_plotter,
            ctx,
            last_serial_scan: Instant::now(),
            program: None,
            cancel_render: cancel_render_receiver,
            last_render: None,
            gcode: None,
            progress: (0, 0, 0),
            state: PlotterState::Busy,
        };
        (core, vm_to_app, vm_from_app, cancel_render_sender)
    }

    pub fn set_pen_position(&mut self, down: bool) {
        match self.machine.post_template() {
            Ok(tpl) => {
                let context = TeraContext::new();
                match tpl.render(if down { "pendown" } else { "penup" }, &context) {
                    Ok(cmd) => {
                        self.plot_sender
                        .send(PlotterCommand::Command(cmd))
                        .unwrap_or_else(|err|{
                            self.shutdown = true;
                            eprintln!("Plot sender is dead and I cannot connect. Dying: {:?}", err);
                            self.state_change_out.send(ApplicationStateChangeMsg::Dead)
                                .expect("Can't even notify ViewModel I'm dead?! YOLO and apparently not for very long.");
                        });
                    }
                    Err(err) => {
                        self.state_change_out
                            .send(ApplicationStateChangeMsg::Error(
                                format!("Error creating machine template: {:?}", err).into(),
                            ))
                            .expect("Cannot send error message. Dead?");
                    }
                }
            }
            Err(err) => {
                self.state_change_out
                    .send(ApplicationStateChangeMsg::Error(
                        format!("Error creating machine template: {:?}", err).into(),
                    ))
                    .expect("Cannot send error message. Dead?");
            }
        }
    }

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
                        extents,
                        resolution,
                    } => {
                        let cimg = match render_source::render_svg_preview(
                            &self.project,
                            extents,
                            resolution,
                            &self.state_change_out,
                            &self.cancel_render
                        ) {
                            Ok(cimg) => Some(cimg),
                            Err(_) => None,
                        };

                        if let Some(cimg) = cimg{
                            self.last_render = Some(cimg.clone());
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::UpdateSourceImage {
                                    image: cimg,
                                    extents,
                                })
                                .unwrap_or_else(|_err| {
                                    self.shutdown = true;
                                    eprintln!("Failed to send message from bap core. Shutting down.");
                                });

                        }
                        self.ctx.request_repaint();
                    }
                    ViewCommand::ImportSVG(path_buf) => {
                        self.project.import_svg(&path_buf, true);

                        self.state_change_out
                            .send(ApplicationStateChangeMsg::SourceChanged {
                                extents: (
                                    self.project.extents().min().x,
                                    self.project.extents().min().y,
                                    self.project.extents().width(),
                                    self.project.extents().height(),
                                ),
                            })
                            .unwrap_or_else(|_err| {
                                self.shutdown = true;
                                eprintln!("Failed to send message from bap core. Shutting down.");
                            });

                        self.ctx.request_repaint();
                    }
                    ViewCommand::SetOrigin(x, y) => {
                        self.project.set_origin(&Some((x,y)));
                    },
                    ViewCommand::SetClipBoundary {
                        min: _min,
                        max: _max,
                    } => todo!(),
                    ViewCommand::RotateSource {
                        center: _center,
                        theta: _theta,
                    } => todo!(),
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
                        self.project.scale_by_factor(factor);

                        self.state_change_out
                            .send(ApplicationStateChangeMsg::SourceChanged {
                                extents: (
                                    self.project.extents().min().x,
                                    self.project.extents().min().y,
                                    self.project.extents().width(),
                                    self.project.extents().height(),
                                ),
                            })
                            .unwrap_or_else(|_err| {
                                self.shutdown = true;
                                eprintln!("Failed to send message from bap core. Shutting down.");
                            });

                        self.ctx.request_repaint();
                    },
                    ViewCommand::RequestPlotPreviewImage { extents, resolution } => {
                        let empty = Vec::new();
                        let gcode = match &self.gcode{
                            Some(gcode)=>gcode,
                            None=>&empty,
                        };
                        let cimg = match render_plot_preview(
                            &self.project,
                            gcode,
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
                            self.last_render = Some(cimg.clone());
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::UpdateSourceImage {
                                    image: cimg,
                                    extents,
                                })
                                .unwrap_or_else(|_err| {
                                    self.shutdown = true;
                                    eprintln!("Failed to send message from bap core. Shutting down.");
                                });

                        }
                        self.ctx.request_repaint();

                    },
                    ViewCommand::ApplyPens(pen_details) => {
                        println!("GOT NEW PENS: {:?}", pen_details);
                        self.project.update_pen_details(&pen_details);
                    },
                },
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
                        PlotterResponse::Loaded(msg) => {
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
