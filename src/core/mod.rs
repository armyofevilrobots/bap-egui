use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use egui::{ColorImage, Context};

pub(crate) mod commands;
pub(crate) mod project;
pub(crate) mod render;
use commands::{ApplicationStateChangeMsg, ViewCommand};
use egui_extras::Size;
use geo::{Coord, Rect};
use tiny_skia::{LineCap, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};
use usvg::{Options, Tree};

use crate::core::project::Project;
use crate::machine::MachineConfig;

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
        let new = ApplicationCore {
            view_command_in: app_from_vm,
            state_change_out: app_to_vm,
            shutdown: false,
            project: Project::new(),
            machine: MachineConfig::default(),
            ctx,
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

                        // let (cimg, extents) =
                        //     match render::render_svg_preview(&self.project, self.zoom as f32) {
                        //         Ok((cimg, extents)) => (cimg, extents),
                        //         Err(_) => (
                        //             ColorImage::example(),
                        //             Rect::new(Coord { x: 0., y: 0. }, Coord { x: 50., y: 50. }),
                        //         ),
                        //     };

                        // self.state_change_out
                        //     .send(ApplicationStateChangeMsg::UpdateSourceImage { image: (), extents: () } {
                        //         size: (extents.width(), extents.height()), //(cimg.width() as f64, cimg.height() as f64),
                        //         image: cimg,
                        //         min: (extents.min().x, extents.min().y),
                        //         zoom: self.zoom,
                        //     })
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
                },
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
