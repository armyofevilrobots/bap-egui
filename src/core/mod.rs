use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use egui::{ColorImage, Context};

pub(crate) mod commands;
pub(crate) mod project;
pub(crate) mod render;
use commands::{ApplicationStateChangeMsg, ViewCommand};
use egui_extras::Size;
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
                    ViewCommand::ZoomView(_) => todo!(),
                    ViewCommand::ImportSVG(path_buf) => {
                        // println!("Received import SVG command for path: {:?}", &path_buf);
                        //

                        self.project.import_svg(&path_buf, true);

                        let cimg = match render::render_svg_preview(
                            &Tree::from_str("<svg/>", &Options::default()).unwrap(),
                        ) {
                            Ok(cimg) => cimg,
                            Err(_) => ColorImage::example(),
                        };

                        self.state_change_out
                            .send(ApplicationStateChangeMsg::UpdateSVGImage {
                                image: cimg,
                                min: (0., 0.),
                                size: (200., 200.),
                                zoom: 1.,
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
}
