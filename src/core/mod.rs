use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime};

use egui::{ColorImage, Context};

pub(crate) mod commands;
pub(crate) mod machine;
pub(crate) mod pick_map;
pub(crate) mod post;
pub(crate) mod project;
pub(crate) mod render_plot;
pub(crate) mod render_source;
pub(crate) mod sender;
pub(crate) mod serial;

use commands::{ApplicationStateChangeMsg, ViewCommand};
use gcode::GCode;
use tera::Context as TeraContext;

use crate::core::project::Project;
use crate::core::render_plot::render_plot_preview;
use crate::view_model::view_model_patch::ViewModelPatch;
use machine::MachineConfig;
use sender::{PlotterCommand, PlotterConnection, PlotterResponse, PlotterState};

/// The actual application core that does shit.
///
///
const UNDO_MAX: usize = 16;
const PICKED_ROTATE_TIME: f64 = 0.1;

#[derive(Debug)]
pub struct ApplicationCore {
    config_dir: PathBuf,
    view_command_in: Receiver<ViewCommand>,
    state_change_out: Sender<ApplicationStateChangeMsg>,
    cancel_render: Receiver<()>,
    shutdown: bool,
    project: Project,
    history: Vec<Project>,
    machine: MachineConfig,
    last_render: Option<(ColorImage, (f64, f64, f64, f64))>,
    pick_image: Option<(Vec<u32>, (f64, f64, f64, f64))>,
    picked: Option<u32>,
    last_rendered: Instant,
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
            config_dir: std::env::home_dir().unwrap_or(
                std::env::current_dir().expect("Cannot determine homedir OR cwd. Dying."),
            ),
            view_command_in: app_from_vm,
            state_change_out: app_to_vm,
            cancel_render: cancel_render_receiver,
            shutdown: false,
            project: Project::new(),
            history: vec![],
            machine: MachineConfig::default(),
            last_render: None,
            pick_image: None,
            ctx,
            plot_sender: app_to_plotter,
            plot_receiver: plotter_to_app,
            last_serial_scan: Instant::now(),
            last_rendered: Instant::now(),
            program: None,
            gcode: None,
            progress: (0, 0, 0),
            state: PlotterState::Busy,
            picked: None,
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

    fn send_project_origin(&mut self) {
        if let Some(origin) = self.project.origin {
            // println!("Sending project origin: {:?}", self.project.origin);
            self.state_change_out
                .send(ApplicationStateChangeMsg::OriginChanged(origin.0, origin.1))
                .expect("Cannot send error message. Dead?");
        } else {
            eprintln!("Project has no origin to send!");
        };
    }

    fn rebuild_after_content_change(&mut self) {
        self.project.regenerate_extents();
        let (pick_image, tmp_extents) =
            pick_map::render_pick_map(&self.project, &self.state_change_out)
                .expect("Failed to generate pick map!");

        self.pick_image = Some((
            pick_image,
            (
                tmp_extents.min().x,
                tmp_extents.min().y,
                tmp_extents.width(),
                tmp_extents.height(),
            ),
        ));
        let app_extents = ApplicationStateChangeMsg::SourceChanged {
            extents: (
                self.project.extents().min().x,
                self.project.extents().min().y,
                self.project.extents().width(),
                self.project.extents().height(),
            ),
        };
        // println!("Going to send AppSCMSG: {:?}", app_extents);
        self.state_change_out
            .send(app_extents)
            .unwrap_or_else(|_err| {
                self.shutdown = true;
                eprintln!("Failed to send message from bap core. Shutting down.");
            });

        self.ctx.request_repaint();
    }

    pub fn checkpoint(&mut self) {
        self.history.push(self.project.clone());
        if self.history.len() > UNDO_MAX {
            self.history.remove(0);
        }
        self.rebuild_after_content_change();
        self.update_vm_undo_avail();
    }

    pub fn update_vm_undo_avail(&mut self) {
        self.state_change_out
            .send(ApplicationStateChangeMsg::UndoAvailable(
                !self.history.is_empty(),
            ))
            .unwrap_or_else(|_| eprintln!("Unexpected dead sender/receiver for app state change!"));
    }

    pub fn undo(&mut self) {
        self.project = match self.history.pop() {
            Some(project) => project,
            None => Project::new(),
        };
        self.send_project_origin();
        self.project.regenerate_extents();
        self.rebuild_after_content_change();
        self.update_vm_undo_avail();
        self.update_vm_paper();
    }

    pub fn update_vm_paper(&self) {
        self.state_change_out
            .send(ApplicationStateChangeMsg::PaperChanged(
                self.project.paper.clone(),
            ))
            .unwrap_or_else(|_| eprintln!("Unexpected dead sender/receiver for app state change!"));
    }

    pub fn run(&mut self) {
        // First, send the default image to display:
        let mut last_sent_plotter_running_progress = Instant::now() - Duration::from_secs(60); // Just pretend it's been a while.

        while !self.shutdown {
            while let Ok(_) = self.cancel_render.try_recv() {
                eprintln!("Draining excessive cancels.");
            }
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
                        let phase = (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).expect("Should never fail to calc unix seconds.");
                        let phase = phase.as_secs_f64();
                        let (cimg, extents_out) = match render_source::render_svg_preview(
                            &self.project,
                            zoom,
                            rotation.clone(),
                            self.picked.clone(),
                            phase,
                            &self.state_change_out,
                            &self.cancel_render,
                        ) {
                            Ok((cimg, xo)) => {
                                // eprintln!("Rendered CIMG of {:?}", cimg.size);
                                (Some(cimg), (xo.min().x, xo.min().y, xo.width(), xo.height()))
                            },
                            Err(err) => {
                                eprintln!("Error rendering source image: {:?}", err);
                                let min_x = self.project.extents().min().x;
                                let min_y = self.project.extents().min().y;

                                (None, (min_x, min_y, self.project.extents().width(), self.project.extents().height()))
                            },
                        };

                        if let Some(cimg) = cimg{
                            self.last_render = Some((cimg.clone(), extents_out.clone()));
                            self.last_rendered = Instant::now();
                            self.state_change_out
                                .send(ApplicationStateChangeMsg::UpdateSourceImage {
                                    image: cimg,
                                    extents: extents_out,
                                    rotation: rotation.clone(),
                                })
                                .unwrap_or_else(|_err| {
                                    self.shutdown = true;
                                    eprintln!("Failed to send message from bap core. Shutting down.");
                                });

                        }
                        self.ctx.request_repaint();
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
                            self.picked = Some(id as u32);
                            self.state_change_out.send(ApplicationStateChangeMsg::Picked(Some(id as usize))).expect("OMFG ViewModel is borked sending pick id");
                        }else{
                            self.state_change_out.send(ApplicationStateChangeMsg::Picked(None)).expect("OMFG ViewModel is borked sending pick id");
                            self.picked = None
                        }
                        self.last_rendered=Instant::now()+Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
                        self.ctx.request_repaint();
                    },
                },
            }

            if let Some(id) = self.picked
                && (Instant::now() - self.last_rendered)
                    > Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64)
            {
                // println!("Refreshing geo pick image for {}", id);
                self.picked = Some(id as u32);
                self.state_change_out
                    .send(ApplicationStateChangeMsg::Picked(Some(id as usize)))
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

    fn try_pick(&self, x: f64, y: f64) -> Option<u32> {
        if let Some((pick_img, extents)) = &self.pick_image {
            // println!("Click MM are {},{}", x, y);
            let xpx = (x - extents.0).ceil() as usize * pick_map::PICKS_PER_MM;
            let ypx = (y - extents.0).ceil() as usize * pick_map::PICKS_PER_MM;
            if x < extents.0
                || y < extents.1
                || y > (extents.1 + extents.3)
                || x > (extents.0 + extents.2)
            {
                // println!("Outside of extents.");
                return None;
            }
            let xspan = extents.2.ceil() as usize * pick_map::PICKS_PER_MM;
            // println!("Extents are: {:?}", extents);
            // println!("XPX and YPX are {},{}", xpx, ypx);
            if let Some(id) = pick_img.get(xpx + ypx * xspan) {
                if *id == u32::MAX {
                    return None;
                }
                match self.project.geometry.get(*id as usize) {
                    Some(geo) => Some(geo.id.clone() as u32),
                    None => None,
                }
            } else {
                None
            }
            // println!("Picked ID: {:?}", id);
            // None
        } else {
            None
        }
    }
}
