use std::collections::VecDeque;
use std::f32;
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, TextureHandle, Vec2, pos2, vec2};
use egui_toast::{Toast, ToastKind, ToastOptions};
use rfd::FileDialog;

use crate::core::commands::{ApplicationStateChangeMsg, ViewCommand};

use crate::core::config::{AppConfig, DockPosition};
use crate::core::machine::MachineConfig;
use crate::core::project::{Orientation, PaperSize, PenDetail};
use crate::core::sender::{PlotterResponse, PlotterState};
use view_model_patch::ViewModelPatch;
pub(crate) mod command_context;
pub(crate) mod default;
pub(crate) mod file_ops;
pub(crate) mod paper;
pub(crate) mod pick;
pub(crate) mod space_commands;
pub(crate) mod util;
pub(crate) mod view_core_update;
pub(crate) mod view_model_eframe;
pub(crate) mod view_model_get_set;
pub(crate) mod view_model_patch;
use crate::core::config::RulerOrigin;
pub use command_context::CommandContext;
pub use util::*;

pub const PIXELS_PER_MM: f32 = 4.; // This is also scaled by the PPP value, but whatever.
pub const MAX_SIZE: usize = 8192;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BAPDisplayMode {
    SVG,
    Plot,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileSelector {
    ImportSVG(PathBuf),
    LoadPGF(PathBuf),
    OpenProject(PathBuf),
    SaveProjectAs(PathBuf),
    //SaveProject,
}

pub struct BAPViewModel {
    config: AppConfig,
    toolbar_position: DockPosition,
    display_mode: BAPDisplayMode,
    state_in: Option<Receiver<ApplicationStateChangeMsg>>,
    cmd_out: Option<Sender<ViewCommand>>,
    status_msg: Option<String>,
    progress: Option<(String, usize)>,
    file_selector: Option<Receiver<FileSelector>>,
    source_image_handle: Option<Box<TextureHandle>>,
    source_image_extents: Option<Rect>, // Again, this is in mm, and needs conversion before display.
    timeout_for_source_image: Option<Instant>,
    dirty: bool, // If we request a new image while one is already rendering, we set this so that it retries right after.
    look_at: Pos2, // What coordinate is currently at the center of the screen
    center_coords: Pos2, // Where in the window (cursor) is the center of the view
    view_zoom: f64, // What is our coordinate/zoom multiplier
    ppp: f32,    // Pixels per point.
    command_context: CommandContext,
    paper_orientation: Orientation,
    paper_size: PaperSize,
    paper_color: Color32,
    origin: Pos2,
    machine_config: MachineConfig,
    show_machine_limits: bool,
    show_paper: bool,
    show_rulers: bool,
    show_extents: bool,
    edit_cmd: String,
    container_rect: Option<Rect>,
    serial_ports: Vec<String>,
    current_port: String,
    move_increment: f32,
    join_handle: Option<JoinHandle<()>>,
    plotter_state: PlotterState,
    queued_toasts: VecDeque<Toast>,
    pen_crib: Vec<PenDetail>,
    cancel_render: Option<Sender<()>>,
    undo_available: bool,
    file_path: Option<PathBuf>,
    ruler_origin: RulerOrigin,
    last_pointer_pos: Option<Pos2>,
    picked: Option<Vec<usize>>,
}

impl BAPViewModel {
    pub fn with_appstate_recv(mut self, state: Receiver<ApplicationStateChangeMsg>) -> Self {
        self.state_in = Some(state);
        self
    }

    pub fn with_viewcommand_send(mut self, state: Sender<ViewCommand>) -> Self {
        self.cmd_out = Some(state);
        self
    }

    pub fn ungroup(&mut self) {
        if self.picked().is_some() {
            self.yolo_view_command(ViewCommand::UnGroup);
        } else {
            self.toast(
                "Can't ungroup with no selection".to_string(),
                ToastKind::Error,
                5.,
            );
        }
    }

    pub fn merge_group(&mut self) {
        if self.picked().is_some() {
            self.yolo_view_command(ViewCommand::Group);
        } else {
            self.toast(
                "Can't merge/group with no selection".to_string(),
                ToastKind::Error,
                5.,
            );
        }
    }

    /// Takes a given bounding box (extents) and calculates how big it would be if rotated d degrees.
    #[allow(unused)]
    pub fn calc_rotated_bounding_box(around: Pos2, angle: f32, r: Rect) -> Rect {
        let _points = vec![
            rotate_pos2_around_pos2(r.left_top(), around, angle),
            rotate_pos2_around_pos2(r.right_top(), around, angle),
            rotate_pos2_around_pos2(r.right_bottom(), around, angle),
            rotate_pos2_around_pos2(r.left_bottom(), around, angle),
        ];
        todo!()
    }

    pub fn undo(&self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out.send(ViewCommand::Undo).unwrap_or_else(|err| {
                eprintln!("Failed to undo due to: {:?}. Terminating.", err);
                exit(-1);
            })
        };
    }

    pub fn patch(&mut self, patch: ViewModelPatch) {
        // println!("Got patch: {:?}", patch);
        // let mut redraw = false;
        if let Some(pens) = patch.pens {
            self.pen_crib = pens
        }
        if let Some(paper) = patch.paper {
            self.paper_size = paper.size;
            self.paper_color = Color32::from_rgb(
                (255.0 * paper.rgb.0).max(0.).min(255.) as u8,
                (255.0 * paper.rgb.1).max(0.).min(255.) as u8,
                (255.0 * paper.rgb.2).max(0.).min(255.) as u8,
            );
            self.set_paper_orientation(&paper.orientation, false);
        }
        if let Some(opt_origin) = patch.origin {
            self.origin = match opt_origin {
                Some(origin) => pos2(origin.0 as f32, origin.1 as f32),
                None => pos2(0., 0.),
            }
        }
        if let Some(extents) = patch.extents {
            self.source_image_extents = Some(Rect::from_min_max(
                pos2(extents.0 as f32, extents.1 as f32),
                pos2(extents.2 as f32, extents.3 as f32),
            ));
        }
        if let Some(opt_machine) = patch.machine_config {
            self.machine_config = match opt_machine {
                Some(machine) => machine,
                None => MachineConfig::default(),
            };
        }
        if let Some(_program) = patch.program {
            // TODO: Have the program available for editing.
        }
        if let Some(opt_file_path) = patch.file_path {
            // A bit weird. A NONE value means don't patch, but None is also a valid path setting
            // for a new project that hasn't been saved. The workaround is to just blank it out.
            self.file_path = opt_file_path;
        }
    }

    pub fn yolo_view_command(&self, cmd: ViewCommand) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(cmd.clone())
                .expect(format!("Failed to send {:?} over MPSC.", &cmd).as_str());
        }
    }

    pub fn degrees_between_two_vecs(a: Vec2, b: Vec2) -> f32 {
        Self::radians_between_two_vecs(a, b) * (180.0 / f32::consts::PI)
    }

    pub fn radians_between_two_vecs(a: Vec2, b: Vec2) -> f32 {
        let a = a.normalized() * vec2(1., -1.);
        let b = b.normalized() * vec2(1., -1.);
        // let a_dot_b_old = a.x * b.x + a.y + b.y;
        let a_dot_b = a.dot(b);
        let a_det_b = a.x * b.y - a.y * b.x;
        let radians = a_det_b.atan2(a_dot_b);
        -radians // Because compass vs trig degrees
    }

    pub fn rotate_around_point(&mut self, point: (f64, f64), degrees: f64) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::RotateSource {
                    center: point,
                    degrees,
                })
                .expect("Failed to send Scale Factor command?");
            self.request_new_source_image();
        }
    }

    pub fn update_pen_details(&mut self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::ApplyPens(self.pen_crib.clone()))
                .expect("Failed to send Scale Factor command?");
            self.request_new_source_image();
        }
    }

    pub fn scale_by_factor(&mut self, factor: f64) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::Scale(factor))
                .expect("Failed to send Scale Factor command?");
        }
    }

    pub fn quit(&mut self) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out.send(ViewCommand::Quit).unwrap_or_else(|err| {
                eprintln!("Failed to quit due to: {:?}. Terminating.", err);
            })
        };
        if let Some(handle) = &self.join_handle {
            let now = Instant::now();
            while !handle.is_finished() && Instant::now() - now < Duration::from_secs(5) {
                sleep(Duration::from_millis(200));
                eprintln!("Waiting for CORE thread to exit...");
            }
        }

        eprintln!("Terminating BAP.");
        exit(-1);
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
        // println!("Connecting port: {:?}", port);
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
            self.set_origin(
                pos2(
                    0.0 - (left_gap - extents.min.x),
                    avail_height as f32 - (bottom_gap - extents.min.y),
                ),
                true,
            );
        } else {
            self.toast_error(
                "Cannot center when source image has no extents.\
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
            self.set_origin(
                pos2(
                    0.0 - (left_gap - extents.min.x),
                    avail_height as f32 - (bottom_gap - extents.min.y),
                ),
                true,
            );
        } else {
            self.toast_error(
                "Cannot center when source image has no extents.\
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
            self.set_origin(
                pos2(
                    0.0 - (left_gap - extents.min.x),
                    avail_height as f32 - (bottom_gap - extents.min.y),
                ),
                true,
            );
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
        if let Some(container_rect) = self.container_rect {
            let extents = match self.source_image_extents {
                Some(extents) => extents,
                None => self.get_paper_rect(),
            };
            let zoom_height = (container_rect.height() - 64.) / extents.height();
            let zoom_width = (container_rect.width() - 64.) / extents.width();
            let zoom_final = 0.9 * (PIXELS_PER_MM * zoom_height.min(zoom_width)) as f64;
            self.set_zoom(zoom_final);
        }
    }

    fn cancel_render(&mut self) {
        if let Some(_timeout) = &self.timeout_for_source_image {
            if let Some(cancel) = &self.cancel_render {
                // println!("Sending cancel, dirty is... {}", self.dirty);
                self.timeout_for_source_image = None;
                cancel
                    .send(())
                    .expect("Failed to send cancellation of render?!");
            }
        }
    }

    pub fn request_new_source_image(&mut self) {
        self.dirty = true
    }

    pub fn check_for_new_source_image(&mut self) {
        if let Some(timeout) = self.timeout_for_source_image {
            if timeout < Instant::now() {
                self.timeout_for_source_image = None;
                self.cancel_render();
            }
        }
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
                    let zoom = if pixel_size_rect.width() > pixel_size_rect.height() {
                        (pixel_size_rect.width().ceil() * self.ppp()) / extents.width()
                    } else {
                        (pixel_size_rect.height().ceil() * self.ppp()) / extents.height()
                    };
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

                    let rotation: Option<((f64, f64), f64)> =
                        if let CommandContext::Rotate(Some(center_mm), Some(ref1_mm), _second) =
                            &self.command_context
                            && let Some(pos) = self.last_pointer_pos
                        {
                            // stuff.
                            let ref2_mm = self.frame_coords_to_mm(pos);
                            // model.command_context =
                            //     CommandContext::Rotate(Some(center_mm), Some(ref1_mm), Some(ref2_mm));
                            let vec_a = *ref1_mm - *center_mm;
                            let vec_b = ref2_mm - *center_mm;
                            let degrees = BAPViewModel::degrees_between_two_vecs(vec_a, vec_b);
                            Some(((center_mm.x as f64, center_mm.y as f64), degrees as f64))
                        } else {
                            None
                        };

                    if let Some(handle) = &self.source_image_handle {
                        let hs = handle.size();
                        // println!("Self::hs {:?}", hs);
                        if hs[0] < MAX_SIZE && hs[1] < MAX_SIZE {
                            //eprintln!("Smaller than max size. Requesting.");
                            sender
                                .send(match self.display_mode {
                                    BAPDisplayMode::SVG => {
                                        // println!("REQUESTING SVG PREVIEW!");
                                        ViewCommand::RequestSourceImage {
                                            // extents: cmd_extents,
                                            zoom: zoom as f64,
                                            // resolution: resolution,
                                            rotation,
                                        }
                                    }
                                    BAPDisplayMode::Plot => {
                                        // println!("REQUESTING PLOT PREVIEW!");
                                        ViewCommand::RequestPlotPreviewImage {
                                            extents: cmd_extents,
                                            resolution: resolution,
                                        }
                                    }
                                })
                                .unwrap_or_else(|err| {
                                    eprintln!("Failed to send request for updated image to core.");
                                    eprintln!("ERR: {:?}", err);
                                });
                            self.timeout_for_source_image =
                                Some(Instant::now() + Duration::from_secs(3));
                        } else if hs[0] > resolution.0 || hs[1] > resolution.1 {
                            sender
                                .send(ViewCommand::RequestSourceImage {
                                    // extents: cmd_extents,
                                    zoom: zoom as f64,
                                    // resolution: resolution,
                                    rotation: None,
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

    /// Helper to orient a rect appropriately.
    pub fn mm_rect_to_screen_rect(&self, rect: Rect) -> Rect {
        let min: Pos2 = self.mm_to_frame_coords(rect.min);
        let max: Pos2 = self.mm_to_frame_coords(rect.max);
        Rect::from_min_max(min, max)
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
            PlotterResponse::Ok(_plotter_command, _) => (),
            PlotterResponse::Loaded(_msg) => self.queued_toasts.push_back(Toast {
                kind: ToastKind::Success,
                text: "GCODE ready to run.".into(),
                options: ToastOptions::default().duration_in_seconds(15.),
                ..Default::default()
            }),
            PlotterResponse::Err(plotter_command, msg) => {
                self.toast_error(format!("{:?} : {}", plotter_command, msg).to_string())
            }
            PlotterResponse::State(plotter_state) => {
                self.plotter_state = plotter_state.clone();
                // println!("Got plotter state: {:?}", plotter_state);
                match &plotter_state {
                    PlotterState::Running(lines, oflines, _) => {
                        // println!("Received running stanza: {:?}", plotter_state);
                        self.progress = Some((
                            format!(
                                "Plotting: {}/{} @{:2}%",
                                lines,
                                oflines,
                                ((lines * 100) as f32 / *oflines as f32).floor() as usize
                            )
                            .to_string(),
                            ((lines * 100) / oflines) as usize,
                        ));
                        if self.timeout_for_source_image.is_none() {
                            // self.dirty = true;
                            // println!("Requesting new source image.");
                            self.request_new_source_image();
                        }
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

#[cfg(test)]
mod test {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    pub fn test_rotate_pos2() {
        let p = Pos2::new(1., 0.);
        let p2 = rotate_pos2(p, 180. * PI / 180.);
        println!("P2->{}", p2);
    }

    #[test]
    pub fn test_rotate_pos2_around_pos2() {
        let p = Pos2::new(2., 0.);

        let around = Pos2::new(1., 0.);
        let p2 = rotate_pos2_around_pos2(p, around, 90. * PI / 180.);
        println!("A90- P->{}, AROUND->{}, P2->{}", p, around, p2);
        let p2 = rotate_pos2_around_pos2(p, around, 180. * PI / 180.);
        println!("A180- P->{}, AROUND->{}, P2->{}", p, around, p2);
    }
}
