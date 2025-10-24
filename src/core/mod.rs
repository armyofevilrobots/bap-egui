use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use egui::{ColorImage, Context};

pub(crate) mod commands;
pub(crate) mod project;
pub(crate) mod render;
pub(crate) mod serial;
use commands::{ApplicationStateChangeMsg, ViewCommand};
use egui_extras::Size;
use geo::{Coord, Rect};
use tiny_skia::{LineCap, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};
use usvg::{Options, Tree};

use crate::core::project::Project;
use crate::machine::MachineConfig;
use crate::sender::{PlotterCommand, PlotterConnection, PlotterResponse};

/// The actual application core that does shit.
///
///

#[derive(Debug)]
pub struct ApplicationCore {
    view_command_in: Receiver<ViewCommand>,
    state_change_out: Sender<ApplicationStateChangeMsg>,
    shutdown: bool,
    project: Project,
    machine: MachineConfig,
    ctx: Context, // Just a repaint context, not used for ANYTHING else.
    plot_sender: Sender<PlotterCommand>,
    plot_receiver: Receiver<PlotterResponse>,
    last_serial_scan: Instant,
}

impl ApplicationCore {
    pub fn new(
        ctx: Context,
    ) -> (
        ApplicationCore,
        Sender<ViewCommand>,
        Receiver<ApplicationStateChangeMsg>,
    ) {
        let (vm_to_app, app_from_vm) = mpsc::channel::<ViewCommand>();
        let (app_to_vm, vm_from_app) = mpsc::channel::<ApplicationStateChangeMsg>();
        // let (app_to_plotter, plotter_from_app) = mpsc::channel::<PlotterCommand>();
        // let (plotter_to_app, app_from_plotter) = mpsc::channel::<PlotterResponse>();
        let (app_to_plotter, plotter_to_app) =
            PlotterConnection::spawn().expect("Failed to create PlotterConnection worker.");
        app_to_vm.send(ApplicationStateChangeMsg::FoundPorts(serial::scan_ports()));
        let new = ApplicationCore {
            view_command_in: app_from_vm,
            state_change_out: app_to_vm,
            shutdown: false,
            project: Project::new(),
            machine: MachineConfig::default(),
            plot_receiver: plotter_to_app,
            plot_sender: app_to_plotter,
            ctx,
            last_serial_scan: Instant::now(),
        };
        (new, vm_to_app, vm_from_app)
    }

    pub fn run(&mut self) {
        // First, send the default image to display:

        while !self.shutdown {
            match self
                .view_command_in
                .recv_timeout(Duration::from_millis(100))
            {
                Err(_err) => (),
                Ok(msg) => match msg {
                    ViewCommand::Ping => {
                                        println!("PING!");
                                        self.state_change_out
                                            .send(ApplicationStateChangeMsg::Pong)
                                            .unwrap_or_else(|_op| self.shutdown = true);
                                        self.ctx.request_repaint();
                                    }
                    ViewCommand::RequestSourceImage {
                                        extents,
                                        resolution,
                                    } => {
                                        let cimg = match render::render_svg_preview(
                                            &self.project,
                                            extents,
                                            resolution,
                                            &self.state_change_out,
                                        ) {
                                            Ok(cimg) => cimg,
                                            Err(_) => ColorImage::example(),
                                        };

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
                                    }
                    ViewCommand::SetOrigin(_, _) => todo!(),
                    ViewCommand::SetClipBoundary {
                                        min: _min,
                                        max: _max,
                                    } => todo!(),
                    ViewCommand::RotateSource {
                                        center: _center,
                                        theta: _theta,
                                    } => todo!(),
                    ViewCommand::Post => todo!(),
                    ViewCommand::StartPlot => todo!(),
                    ViewCommand::PausePlot => todo!(),
                    ViewCommand::CancelPlot => todo!(),
                    ViewCommand::None => todo!(),
                    ViewCommand::Quit => self.shutdown = true,
                    ViewCommand::UpdateMachineConfig(machine_config) => todo!(),
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
                        }),                },
            }

            // Also, we need to check for plotter responses...

            if let Ok(response) = self.plot_receiver.recv_timeout(Duration::from_millis(100)) {
                match response {
                    PlotterResponse::Ok(plotter_command, _) => {}
                    PlotterResponse::Loaded(_) => {}
                    PlotterResponse::Err(plotter_command, _) => {}
                    PlotterResponse::State(plotter_state) => self
                        .state_change_out
                        .send(ApplicationStateChangeMsg::PlotterState(plotter_state))
                        .expect("Failed to send to ViewModel. Dead conn?"),
                }
            }

            // Housekeeping stuffs.
            if self.last_serial_scan + Duration::from_secs(10) < Instant::now() {
                self.state_change_out
                    .send(ApplicationStateChangeMsg::FoundPorts(serial::scan_ports()))
                    .expect("Failed to send serial port update. Dead ViewModel?");
                self.last_serial_scan = Instant::now();
            }
        }

        self.state_change_out
            .send(ApplicationStateChangeMsg::Dead)
            .expect("Failed to send shutdown status back to viewmodel.");
    }

    pub fn import_svg(&mut self, path: &PathBuf, keepdown: bool) {
        self.state_change_out
            .send(ApplicationStateChangeMsg::ProgressMessage {
                message: format!(
                    "Loading SVG from {}",
                    path.as_os_str()
                        .to_str()
                        .expect("Can't convert path to str")
                ),
                percentage: 0,
            });
        // sleep(Duration::from_millis(5000 as u64));
        self.project.import_svg(path, keepdown);
        self.state_change_out
            .send(ApplicationStateChangeMsg::ProgressMessage {
                message: format!(
                    "Loaded SVG from {}",
                    path.as_os_str()
                        .to_str()
                        .expect("Can't convert path to str")
                ),
                percentage: 100,
            });
    }
}
