use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, TextureHandle, Vec2, pos2, vec2};

use crate::core::commands::{ApplicationStateChangeMsg, ViewCommand};

use crate::core::project::{Orientation, PaperSize};
use crate::machine::MachineConfig;

pub const PIXELS_PER_MM: f32 = 4.;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BAPDisplayMode {
    SVG,
    Plot,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CommandContext {
    Origin,
    Clip(Option<Pos2>, Option<Pos2>),
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
    ppp: f32,
    /// Pixels per point.
    pub command_context: CommandContext,
    pub paper_orientation: Orientation,
    pub paper_size: PaperSize,
    pub paper_modal_open: bool,
    pub pen_crib_open: bool,
    pub origin: Pos2,
    pub machine_config: MachineConfig,
    pub paper_color: Color32,
    pub show_machine_limits: bool,
    pub show_paper: bool,
    pub show_rulers: bool,
    pub show_extents: bool,
    pub show_center_tools: bool,
    pub edit_cmd: String,
    pub container_rect: Option<Rect>,
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

    pub fn ppp(&self) -> f32 {
        self.ppp
    }

    pub fn set_ppp(&mut self, ppp: f32) {
        self.ppp = ppp;
        // TODO: Reload the svg preview.
    }

    pub fn center_paper(&mut self, ppp: f32) {
        // self.set_origin
        // let top = self.get_paper_rect()
    }

    /// This one will figure out the center of the paper, center of
    /// the machine, and try and arrange things to give us the nicest
    /// compromise based on the paper being _somewhere_ north-east of
    /// the machine origin.
    pub fn center_smart(&mut self) {}

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

        if let Some(sender) = &self.cmd_out {
            // sender.send(ViewCommand::ZoomView(self.view_zoom));
            // We know the extents of the svg, so we just need to
            // calculate a new image size for the current zoom level.
        }
    }

    pub fn request_new_source_image(&mut self) {
        self.dirty = true
    }

    pub fn check_for_new_source_image(&mut self) {
        let MAX_SIZE = 2048;
        if self.dirty && self.timeout_for_source_image.is_none() {
            if let Some(extents) = self.source_image_extents {
                let cmd_extents = (
                    extents.left() as f64,
                    extents.top() as f64,
                    extents.width() as f64,
                    extents.height() as f64,
                );
                if let Some(sender) = &self.cmd_out {
                    // let scale = self.zoom() as f32 * 1. / self.ppp;
                    // println!("Scale would be: {:?}, but might clamp to 16.", scale);
                    // let scale = scale.max(16.);

                    /*
                    let resolution = (
                        (extents.width() * scale) as usize,
                        (extents.height() * scale) as usize,
                    );*/
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

                    // println!(
                    //     "Requesting an image of {:?} for mm size {:?}",
                    //     resolution,
                    //     // self.zoom(),
                    //     // scale,
                    //     extents
                    // );
                    if let Some(handle) = &self.source_image_handle {
                        let hs = handle.size();
                        // println!("Self::hs {:?}", hs);
                        if (hs[0] < MAX_SIZE && hs[1] < MAX_SIZE) {
                            eprintln!("Smaller than max size. Requesting.");
                            sender.send(ViewCommand::RequestSourceImage {
                                extents: cmd_extents,
                                resolution: resolution,
                            });
                            self.timeout_for_source_image =
                                Some(Instant::now() + Duration::from_secs(3));
                        } else if (hs[0] / 5 > resolution.0 / 4 || hs[1] / 5 > resolution.1 / 4) {
                            eprintln!(
                                "Requesting WAY smaller than current image to avoid jaggies."
                            );
                            sender.send(ViewCommand::RequestSourceImage {
                                extents: cmd_extents,
                                resolution: resolution,
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
            cmd_out.send(ViewCommand::Quit);
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
            paper_modal_open: false,
            pen_crib_open: false,
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
            show_center_tools: false,
            container_rect: None,
            edit_cmd: String::new(), //Just a default
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

        let received = if let Some(msg_in) = &self.state_in {
            match msg_in.try_recv() {
                Ok(msg) => msg,
                Err(_nomsg) => ApplicationStateChangeMsg::None,
            }
        } else {
            ApplicationStateChangeMsg::None
        };
        if received != ApplicationStateChangeMsg::None {
            println!("Received: {:?}", received);
        };
        match received {
            ApplicationStateChangeMsg::Dead => {
                exit(0);
            }
            ApplicationStateChangeMsg::Pong => {}
            ApplicationStateChangeMsg::None => {}
            ApplicationStateChangeMsg::ResetDisplay => todo!(),
            ApplicationStateChangeMsg::UpdateSourceImage {
                image: image,
                extents: (x, y, width, height),
            } => {
                if let Some(handle) = &mut self.source_image_handle {
                    handle.set(image, egui::TextureOptions::NEAREST);
                    self.source_image_extents = Some(Rect::from_min_size(
                        pos2(x as f32, y as f32),
                        vec2(width as f32, height as f32),
                    ));
                }
                // self.dirty = false;
                self.timeout_for_source_image = None;
            }
            ApplicationStateChangeMsg::UpdateMachineConfig(machine_config) => todo!(),
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
        }

        crate::ui::update_ui(self, ctx, frame);

        // This is how to go into continuous mode - uncomment this to see example of continuous mode
        // ctx.request_repaint();
    }
}
