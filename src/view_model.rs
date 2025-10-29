use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, TextureHandle, Vec2, pos2, vec2};
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::core::commands::{ApplicationStateChangeMsg, ViewCommand};

use crate::core::project::{Orientation, PaperSize, PenDetail};
use crate::machine::MachineConfig;
use crate::sender::{PlotterResponse, PlotterState};

pub const PIXELS_PER_MM: f32 = 4.; // This is also scaled by the PPP value, but whatever.
pub const MAX_SIZE: usize = 2048;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BAPDisplayMode {
    SVG,
    Plot,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CommandContext {
    Origin,
    PaperChooser,
    PenCrib,
    PenEdit(usize), // The pen index in Vec<Pens>
    Clip(Option<Pos2>, Option<Pos2>),
    Scale,
    None,
}

pub struct BAPViewModel {
    pub docked: bool,
    pub display_mode: BAPDisplayMode,
    pub state_in: Option<Receiver<ApplicationStateChangeMsg>>,
    pub cmd_out: Option<Sender<ViewCommand>>,
    pub status_msg: Option<String>,
    pub progress: Option<(String, usize)>,
    pub svg_import_mpsc: Option<Receiver<PathBuf>>,
    pub source_image_handle: Option<Box<TextureHandle>>,
    pub source_image_extents: Option<Rect>, // Again, this is in mm, and needs conversion before display.
    pub timeout_for_source_image: Option<Instant>,
    dirty: bool, // If we request a new image while one is already rendering, we set this so that it retries right after.
    pub look_at: Pos2, // What coordinate is currently at the center of the screen
    pub center_coords: Pos2, // Where in the window (cursor) is the center of the view
    view_zoom: f64, // What is our coordinate/zoom multiplier
    ppp: f32,    // Pixels per point.
    pub command_context: CommandContext,
    pub paper_orientation: Orientation,
    pub paper_size: PaperSize,
    // pub paper_modal_open: bool,
    // pub pen_crib_open: bool,
    pub origin: Pos2,
    pub machine_config: MachineConfig,
    pub paper_color: Color32,
    pub show_machine_limits: bool,
    pub show_paper: bool,
    pub show_rulers: bool,
    pub show_extents: bool,
    pub edit_cmd: String,
    pub container_rect: Option<Rect>,
    pub serial_ports: Vec<String>,
    pub current_port: String,
    pub move_increment: f32,
    join_handle: Option<JoinHandle<()>>,
    pub plotter_state: PlotterState,
    pub queued_toasts: VecDeque<Toast>,
    pub pen_crib: Vec<PenDetail>,
    pub scale_factor_temp: f64,
}

pub trait IsPos2Able {
    fn into_pos2(&self) -> Pos2;
}

impl IsPos2Able for Pos2 {
    fn into_pos2(&self) -> Pos2 {
        self.clone()
    }
}
impl IsPos2Able for Vec2 {
    fn into_pos2(&self) -> Pos2 {
        self.to_pos2()
    }
}

impl BAPViewModel {
    pub fn name() -> &'static str {
        "Bot-a-Plot"
    }

    pub fn scale_by_factor(&mut self, factor: f64) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::Scale(factor))
                .expect("Failed to send Scale Factor command?");
        }
    }

    pub fn set_origin(&mut self, origin: Pos2) {
        if let Some(cmd_out) = &self.cmd_out {
            self.origin = origin;
            cmd_out
                .send(ViewCommand::SetOrigin(origin.x as f64, origin.y as f64))
                .expect("Failed to send ORIGIN command?");
        }
    }

    pub fn set_join_handle(&mut self, handle: JoinHandle<()>) {
        self.join_handle = Some(handle);
    }

    pub fn ppp(&self) -> f32 {
        self.ppp
    }

    pub fn request_post(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::Post)
                .expect("Failed to send POST command?");
        }
    }

    pub fn pen_up(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::PenUp)
                .expect("Failed to send Pen Up command?");
        }
    }

    pub fn pen_down(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::PenDown)
                .expect("Failed to send Pen Up command?");
        }
    }

    pub fn plot_start(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::StartPlot)
                .expect("Failed to send Pen Up command?");
        }
    }

    pub fn plot_pause(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::PausePlot)
                .expect("Failed to send Pen Up command?");
        }
    }

    pub fn plot_cancel(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::CancelPlot)
                .expect("Failed to send Pen Up command?");
        }
    }

    pub fn request_relative_move(&self, distance: Vec2) {
        // TODO: Don't send moves if we're currently running.
        let cmd = format!("G91 G0 X{} Y{}", distance.x, distance.y);
        self.send_command(&cmd);
    }

    pub fn close_serial(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::DisconnectPlotter)
                .expect("Failed to send port disconnect due to dead app-socket.")
        }
    }

    pub fn set_serial(&self, port: &String) {
        println!("Connecting port: {:?}", port);
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::ConnectPlotter(port.clone()))
                .expect("Failed to send port selection due to dead app-socket.")
        }
    }

    pub fn send_command(&self, cmd: &String) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::SendCommand(cmd.clone()))
                .expect("Failed to send port selection due to dead app-socket.")
        }
    }

    pub fn set_ppp(&mut self, ppp: f32) {
        self.ppp = ppp;
        // TODO: Reload the svg preview.
    }

    /// Just checks if the source image is too big for either paper or machine.
    fn warn_if_bigger_than_available(&mut self) {
        if let Some(extents) = self.source_image_extents {
            if extents.height() > self.machine_config.limits().1 as f32 {
                self.toast_warning(
                    "Source image is taller than machine extents. Trying flipping or scaling?"
                        .to_string(),
                );
            }
            if extents.width() > self.machine_config.limits().0 as f32 {
                self.toast_warning(
                    "Source image is wider than machine extents. Trying flipping or scaling?"
                        .to_string(),
                );
            }
        }
    }

    pub fn center_machine(&mut self) {
        if let Some(extents) = self.source_image_extents {
            let avail_width = self.machine_config.limits().0;
            let avail_height = self.machine_config.limits().1;

            let left_gap = (avail_width as f32 - extents.width()) / 2.;
            let bottom_gap = (avail_height as f32 - extents.height()) / 2.;
            self.set_origin(pos2(
                0.0 - (left_gap - extents.min.x),
                avail_height as f32 - (bottom_gap - extents.min.y),
            ));
        } else {
            self.toast_error(
                "Cannot smart center when source image has no extents.\
                Try importing an image first?"
                    .to_string(),
            );
        }
    }

    /// Just to the paper, ignoring machine limits
    pub fn center_paper(&mut self) {
        if let Some(extents) = self.source_image_extents {
            let avail_width = self.get_paper_size().x as f64;
            let avail_height = self.get_paper_size().y as f64;

            let left_gap = (avail_width as f32 - extents.width()) / 2.;
            let bottom_gap = (avail_height as f32 - extents.height()) / 2.;
            self.set_origin(pos2(
                0.0 - (left_gap - extents.min.x),
                avail_height as f32 - (bottom_gap - extents.min.y),
            ));
        } else {
            self.toast_error(
                "Cannot smart center when source image has no extents.\
                Try importing an image first?"
                    .to_string(),
            );
        }
    }

    /// This one will figure out the center of the paper, center of
    /// the machine, and try and arrange things to give us the nicest
    /// compromise based on the paper being _somewhere_ north-east of
    /// the machine origin.
    pub fn center_smart(&mut self) {
        if let Some(extents) = self.source_image_extents {
            self.warn_if_bigger_than_available();
            let avail_width = if self.get_paper_size().x as f64 > self.machine_config.limits().0 {
                self.machine_config.limits().0
            } else {
                self.get_paper_size().x as f64
            };
            let avail_height = if self.get_paper_size().y as f64 > self.machine_config.limits().1 {
                self.machine_config.limits().1
            } else {
                // self.paper_size.dims().1
                self.get_paper_size().y as f64
            };

            let left_gap = (avail_width as f32 - extents.width()) / 2.;
            let bottom_gap = (avail_height as f32 - extents.height()) / 2.;
            self.set_origin(pos2(
                0.0 - (left_gap - extents.min.x),
                avail_height as f32 - (bottom_gap - extents.min.y),
            ));
        } else {
            self.toast_error(
                "Cannot smart center when source image has no extents.\
                Try importing an image first?"
                    .to_string(),
            );
        }
    }

    pub fn zoom_fit(&mut self) {
        let rect = if let Some(rect) = self.source_image_extents {
            rect
        } else {
            self.get_paper_rect()
        };
        self.look_at = rect.center();
        if let Some(container_rect) = self.container_rect
            && let Some(extents) = self.source_image_extents
        {
            let zoom_height = (container_rect.height() - 64.) / extents.height();
            let zoom_width = (container_rect.width() - 64.) / extents.width();
            let zoom_final = (PIXELS_PER_MM * zoom_height.min(zoom_width)) as f64;
            self.set_zoom(zoom_final);
        }
    }

    pub fn zoom(&self) -> f64 {
        self.view_zoom
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.view_zoom = zoom;

        if let Some(_sender) = &self.cmd_out {
            // We know the extents of the svg, so we just need to
            // calculate a new image size for the current zoom level.
            self.request_new_source_image();
        }
    }

    pub fn request_new_source_image(&mut self) {
        self.dirty = true
    }

    pub fn check_for_new_source_image(&mut self) {
        if self.dirty && self.timeout_for_source_image.is_none() {
            if let Some(extents) = self.source_image_extents {
                let cmd_extents = (
                    extents.left() as f64,
                    extents.top() as f64,
                    extents.width() as f64,
                    extents.height() as f64,
                );
                if let Some(sender) = &self.cmd_out {
                    let pixel_size_rect = self.mm_rect_to_screen_rect(extents);
                    let ratio = pixel_size_rect.aspect_ratio();
                    let mut resolution = (
                        (self.ppp() * pixel_size_rect.width().ceil()) as usize,
                        (self.ppp() * pixel_size_rect.height().ceil()) as usize,
                    );
                    if resolution.0 > MAX_SIZE && ratio >= 1. {
                        resolution = (MAX_SIZE, (((MAX_SIZE as f32) / ratio) as usize));
                    } else if resolution.0 > MAX_SIZE && ratio < 1. {
                        resolution = ((((MAX_SIZE as f32) * ratio) as usize), MAX_SIZE);
                    }

                    if let Some(handle) = &self.source_image_handle {
                        let hs = handle.size();
                        // println!("Self::hs {:?}", hs);
                        if hs[0] < MAX_SIZE && hs[1] < MAX_SIZE {
                            eprintln!("Smaller than max size. Requesting.");
                            sender
                                .send(ViewCommand::RequestSourceImage {
                                    extents: cmd_extents,
                                    resolution: resolution,
                                })
                                .unwrap_or_else(|err| {
                                    eprintln!("Failed to send request for updated image to core.");
                                    eprintln!("ERR: {:?}", err);
                                });
                            self.timeout_for_source_image =
                                Some(Instant::now() + Duration::from_secs(3));
                        } else if hs[0] / 5 > resolution.0 / 4 || hs[1] / 5 > resolution.1 / 4 {
                            eprintln!(
                                "Requesting WAY smaller than current image to avoid jaggies."
                            );
                            sender
                                .send(ViewCommand::RequestSourceImage {
                                    extents: cmd_extents,
                                    resolution: resolution,
                                })
                                .unwrap_or_else(|err| {
                                    eprintln!("Failed to send request for updated image to core.");
                                    eprintln!("ERR: {:?}", err);
                                });
                            self.timeout_for_source_image =
                                Some(Instant::now() + Duration::from_secs(3));
                        } else {
                            eprintln!("Not updating image because it's already at max size.");
                        }
                    }
                    self.dirty = false;
                }
            }
        }
    }

    /// Orients the rect for the paper to the origin, and
    /// the landscape/portrait config
    ///
    pub fn get_paper_rect(&self) -> Rect {
        self.calc_paper_rect(self.origin)
    }

    pub fn get_paper_size(&self) -> Vec2 {
        match self.paper_orientation {
            Orientation::Portrait => vec2(
                self.paper_size.dims().0 as f32,
                self.paper_size.dims().1 as f32,
            ),
            Orientation::Landscape => vec2(
                self.paper_size.dims().1 as f32,
                self.paper_size.dims().0 as f32,
            ),
        }
    }

    pub fn calc_paper_rect(&self, origin: Pos2) -> Rect {
        match self.paper_orientation {
            Orientation::Portrait => {
                let tl = pos2(origin.x, origin.y - self.paper_size.dims().1 as f32);
                let size = vec2(
                    self.paper_size.dims().0 as f32,
                    self.paper_size.dims().1 as f32,
                );
                Rect::from_min_size(tl, size)
            }
            Orientation::Landscape => {
                let tl = pos2(origin.x, origin.y - self.paper_size.dims().0 as f32);
                let size = vec2(
                    self.paper_size.dims().1 as f32,
                    self.paper_size.dims().0 as f32,
                );
                Rect::from_min_size(tl, size)
            }
        }
    }

    /// Helper to orient a rect appropriately.
    pub fn mm_rect_to_screen_rect(&self, rect: Rect) -> Rect {
        let min: Pos2 = self.mm_to_frame_coords(rect.min);
        let max: Pos2 = self.mm_to_frame_coords(rect.max);
        Rect::from_min_max(min, max)
    }

    // Send a quit request to the application core.
    pub fn request_quit(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out.send(ViewCommand::Quit).unwrap_or_else(|err| {
                eprintln!("Failed to send request for updated image to core.");
                eprintln!("ERR: {:?}", err);
            });
        };
    }

    pub fn mm_to_frame_coords(&self, mm: Pos2) -> Pos2 {
        let tmp = mm - self.look_at.to_vec2(); // Then we push to where we're actually looking.
        let tmp = tmp / PIXELS_PER_MM; // Then we adjust for the native pixel/mm density
        let tmp = tmp * self.view_zoom as f32; // Then we do the zoom adjustment
        let tmp = self.center_coords.into_pos2() + tmp.to_vec2(); // Then translate to center
        // (self.center_coords.into_pos2() + (mm.to_vec2() * (self.view_zoom as f32 / PIXELS_PER_MM))
        //     - self.look_at.to_vec2())
        tmp
    }

    pub fn frame_coords_to_mm(&self, frame_coords: Pos2) -> Pos2 {
        // (((frame_coords - self.center_coords + self.look_at.to_vec2()) * PIXELS_PER_MM)
        //     / self.view_zoom as f32)
        //     .to_pos2();

        let tmp = frame_coords - self.center_coords.into_pos2();
        let tmp = tmp / self.view_zoom as f32;
        let tmp = tmp * PIXELS_PER_MM;
        let tmp = tmp + self.look_at.to_vec2();
        tmp.to_pos2()
    }

    pub fn scale_mm_to_screen(&self, mm: Vec2) -> Vec2 {
        mm * self.view_zoom as f32 / PIXELS_PER_MM
    }
    pub fn scale_screen_to_mm(&self, pt: Vec2) -> Vec2 {
        pt * PIXELS_PER_MM / self.view_zoom as f32
    }

    /// Sends a quick info message.

    pub fn toast(&mut self, message: String, kind: ToastKind, duration: f64) {
        self.queued_toasts.push_back(Toast {
            kind,
            text: message.into(),
            options: ToastOptions::default().duration_in_seconds(duration),
            ..Default::default()
        })
    }

    pub fn toast_info(&mut self, message: String) {
        self.toast(message, ToastKind::Info, 5.);
    }

    /// Sends a quick error message.
    pub fn toast_warning(&mut self, message: String) {
        self.toast(message, ToastKind::Warning, 10.);
    }

    /// Sends a quick error message.
    pub fn toast_error(&mut self, message: String) {
        self.toast(message, ToastKind::Error, 15.);
    }

    pub fn handle_plotter_response(&mut self, plotter_response: PlotterResponse) {
        match plotter_response {
            crate::sender::PlotterResponse::Ok(_plotter_command, _) => (),
            crate::sender::PlotterResponse::Loaded(_msg) => self.queued_toasts.push_back(Toast {
                kind: ToastKind::Success,
                text: "GCODE ready to run.".into(),
                options: ToastOptions::default().duration_in_seconds(15.),
                ..Default::default()
            }),
            crate::sender::PlotterResponse::Err(plotter_command, msg) => {
                self.toast_error(format!("{:?} : {}", plotter_command, msg).to_string())
            }
            crate::sender::PlotterResponse::State(plotter_state) => {
                self.plotter_state = plotter_state.clone();
                match &plotter_state {
                    PlotterState::Running(lines, oflines, _) => {
                        self.progress = Some((
                            format!("Plotting: {}/{} GCODE commands", lines, oflines).to_string(),
                            ((lines * 100) / oflines) as usize,
                        ));
                    }
                    PlotterState::Disconnected => {
                        self.toast_error("Plotter disconnected.".to_string())
                    }
                    PlotterState::Connecting(_) => (),
                    PlotterState::Ready => self.toast_info("Plotter ready.".to_string()),
                    PlotterState::Paused(line, oflines, _) => self.toast_info(
                        format!("Plotter paused at line {}/{}", line, oflines).to_string(),
                    ),
                    PlotterState::Busy => (),
                    PlotterState::Failed(msg) => {
                        self.toast_error(format!("Plotter failed: {}", msg).to_string())
                    }
                    PlotterState::Terminating => (),
                    PlotterState::Dead => self.toast_error("Plotter is dead.".to_string()),
                }
            }
        }
    }
}

impl Default for BAPViewModel {
    fn default() -> Self {
        Self {
            docked: true,
            display_mode: BAPDisplayMode::SVG,
            state_in: None,
            cmd_out: None,
            status_msg: None,
            progress: None,
            svg_import_mpsc: None,
            source_image_handle: None,
            source_image_extents: None,
            timeout_for_source_image: None,
            look_at: Pos2 { x: 0., y: 0. },
            view_zoom: 4.,
            command_context: CommandContext::None,
            paper_orientation: Orientation::Portrait,
            // paper_modal_open: false,
            // pen_crib_open: false,
            paper_size: PaperSize::Letter,
            origin: pos2(0., 0.),
            paper_color: Color32::WHITE,
            center_coords: pos2(0., 0.),
            machine_config: MachineConfig::default(),
            show_machine_limits: true,
            show_paper: true,
            show_rulers: true,
            show_extents: true,
            ppp: 1.5,
            dirty: false,
            container_rect: None,
            edit_cmd: String::new(),
            serial_ports: Vec::new(), //Just a default
            current_port: "".to_string(),
            join_handle: None,
            move_increment: 5.,
            plotter_state: PlotterState::Disconnected,
            queued_toasts: VecDeque::new(),
            pen_crib: vec![
                Default::default(),
                PenDetail {
                    tool_id: 2,
                    name: "Red Pen".to_string(),
                    stroke_width: 1.0,
                    stroke_density: 1.0,
                    feed_rate: Some(2000.0),
                    color: "#FF0000".to_string(),
                },
                PenDetail {
                    tool_id: 3,
                    name: "Blue Pen".to_string(),
                    stroke_width: 0.25,
                    stroke_density: 0.5, // It's runny
                    feed_rate: Some(1000.0),
                    color: "#0000FF".to_string(),
                },
            ],
            scale_factor_temp: 1.,
        }
    }
}

impl eframe::App for BAPViewModel {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(msg_in) = &self.svg_import_mpsc {
            match msg_in.try_recv() {
                Ok(path) => {
                    if let Some(cmd_out) = &self.cmd_out {
                        self.svg_import_mpsc = None;
                        cmd_out
                            .send(ViewCommand::ImportSVG(path))
                            .expect("Failed to send Import SVG command over MPSC.");
                    }
                }
                Err(_) => (),
            }
        }

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
                } => {
                    if let Some(handle) = &mut self.source_image_handle {
                        handle.set(image, egui::TextureOptions::NEAREST);
                        println!("Got incoming extents: {},{},{}w,{}h", x, y, width, height);
                        self.source_image_extents = Some(Rect::from_min_size(
                            pos2(x as f32, y as f32),
                            vec2(width as f32, height as f32),
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
                    self.progress = Some((message, percentage));
                }
                ApplicationStateChangeMsg::SourceChanged { extents } => {
                    // self.waiting_for_source_image=true;
                    self.source_image_extents = Some(Rect::from_min_size(
                        pos2(extents.0 as f32, extents.1 as f32),
                        vec2(extents.2 as f32, extents.3 as f32),
                    ));
                    self.dirty = true;
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
                    self.display_mode = BAPDisplayMode::Plot;
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
            }
        }

        crate::ui::update_ui(self, ctx, frame);

        // This is how to go into continuous mode - uncomment this to see example of continuous mode
        // ctx.request_repaint();
    }
}
