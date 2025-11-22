use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use egui::{ColorImage, Context};

pub(crate) mod commands;
pub(crate) mod core_run;
pub(crate) mod group_ungroup;
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
    picked: Option<BTreeSet<u32>>,
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
            state: PlotterState::Disconnected,
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
        self.project.reindex_geometry();
        self.project.regenerate_extents();
        let (pick_image, tmp_extents) =
            pick_map::render_pick_map(&self.project, &self.state_change_out)
                .expect("Failed to generate pick map!");
        let mut found: BTreeSet<u32> = BTreeSet::new();
        for pixel in &pick_image {
            found.insert(*pixel);
        }
        // println!("FOUND: {:?}", found);

        // println!("Updating pick image.");
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

    pub fn yolo_send_plotter_cmd(&mut self, cmd: PlotterCommand) {
        self.plot_sender.send(cmd).unwrap_or_else(|err| {
            self.shutdown = true;
            eprintln!("Plot sender is dead and I cannot connect. Dying: {:?}", err);
            self.state_change_out
                .send(ApplicationStateChangeMsg::Dead)
                .expect(
                    "Can't even notify ViewModel I'm dead?! YOLO and apparently not for very long.",
                );
        });
    }

    pub fn handle_plotter_response(
        &mut self,
        response: PlotterResponse,
        last_sent_plotter_running_progress: &mut Instant,
    ) {
        match &response {
            PlotterResponse::Ok(_plotter_command, _) => (),
            PlotterResponse::Err(_plotter_command, _) => {}
            PlotterResponse::State(plotter_state) => {
                if let PlotterState::Running(line, of, _something) = plotter_state {
                    self.progress = (*line as usize, *of as usize, *_something as usize);
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
        if let PlotterResponse::State(PlotterState::Running(line, of, other)) = &response {
            if Instant::now() - *last_sent_plotter_running_progress > Duration::from_secs(1) {
                // println!("Sending progress running stanza.");
                self.state_change_out
                    .send(ApplicationStateChangeMsg::PlotterResponse(
                        PlotterResponse::State(PlotterState::Running(*line, *of, *other)),
                    ))
                    .expect("Failed to send to ViewModel. Dead conn?");
                *last_sent_plotter_running_progress = Instant::now();
            }
        } else {
            self.state_change_out
                .send(ApplicationStateChangeMsg::PlotterResponse(response.clone()))
                .expect("Failed to send to ViewModel. Dead conn?");
        }
    }
}
