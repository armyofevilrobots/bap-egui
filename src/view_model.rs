use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use egui::{Color32, Pos2, Rect, TextureHandle, Vec2, pos2, vec2};

use crate::core::commands::{ApplicationStateChangeMsg, ViewCommand};

use crate::core::project::{Orientation, PaperSize};
use crate::machine::MachineConfig;

pub const PIXELS_PER_MM: f32 = 10.0;

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
    pub svg_img_handle: Option<Box<TextureHandle>>,
    pub svg_img_dims: Option<Rect>, // How big is the svg it represents x/y mm
    pub svg_img_zoom: Option<f64>, // How was this image zoomed when created? (for determining re-render requests)
    pub look_at: Pos2,             // What coordinate is currently at the center of the screen
    pub center_coords: Pos2,       // Where in the window (cursor) is the center of the view
    pub view_zoom: f64,            // What is our coordinate/zoom multiplier
    pub command_context: CommandContext,
    pub paper_orientation: Orientation,
    pub paper_size: PaperSize,
    pub paper_modal_open: bool,
    pub pen_crib_open: bool,
    pub origin: Pos2,
    pub machine_config: MachineConfig,
    pub paper_color: Color32,
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

    pub fn mm_to_frame_coords<T>(&self, mm: T) -> Pos2
    where
        T: IsPos2Able,
    {
        let mm = mm.into_pos2(); // First just a raw position
        let tmp = mm - self.look_at.to_vec2(); // Then we push to where we're actually looking.
        let tmp = tmp / PIXELS_PER_MM; // Then we adjust for the native pixel/mm density
        let tmp = tmp * self.view_zoom as f32; // Then we do the zoom adjustment
        let tmp = self.center_coords.into_pos2() + tmp.to_vec2(); // Then translate to center
        // (self.center_coords.into_pos2() + (mm.to_vec2() * (self.view_zoom as f32 / PIXELS_PER_MM))
        //     - self.look_at.to_vec2())
        tmp
    }

    pub fn frame_coords_to_mm<T>(&self, frame_coords: T) -> Pos2
    where
        T: IsPos2Able,
    {
        (((frame_coords.into_pos2() - self.center_coords + self.look_at.to_vec2()) * PIXELS_PER_MM)
            / self.view_zoom as f32)
            .to_pos2()
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
            svg_img_handle: None,
            svg_img_dims: None,
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
            svg_img_zoom: None,
            machine_config: MachineConfig::default(), //Just a default
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
            ApplicationStateChangeMsg::UpdateSVGImage {
                image,
                min: (left, top),
                size: (width, height),
                zoom,
            } => {
                if let Some(handle) = &mut self.svg_img_handle {
                    handle.set(image, egui::TextureOptions::NEAREST);
                    self.svg_img_dims = Some(Rect::from_min_size(
                        pos2(left as f32, top as f32),
                        vec2(width as f32, height as f32),
                    ));
                    self.svg_img_zoom = Some(zoom);
                }
            }
            ApplicationStateChangeMsg::UpdateMachineConfig(machine_config) => todo!(),
        }

        crate::ui::update_ui(self, ctx, frame);

        // This is how to go into continuous mode - uncomment this to see example of continuous mode
        // ctx.request_repaint();
    }
}
