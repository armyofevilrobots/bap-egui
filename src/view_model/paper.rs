use egui::{Color32, Pos2, Rect, Vec2, pos2, vec2};

use super::BAPViewModel;
use crate::core::{
    commands::ViewCommand,
    project::{Orientation, Paper, PaperSize},
};

impl BAPViewModel {
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

    pub fn paper_size(&self) -> PaperSize {
        self.paper_size.clone()
    }

    pub fn paper_orientation(&self) -> Orientation {
        self.paper_orientation.clone()
    }

    pub fn paper_color(&self) -> Color32 {
        self.paper_color.clone()
    }

    pub fn set_paper_color(&mut self, color: &Color32, create_history: bool) {
        // println!("Setting paper color to {:?}", color.clone());
        self.paper_color = color.clone();
        if let Some(cmd_out) = &self.cmd_out
            && create_history
        {
            let paper_out = ViewCommand::SetPaper(Paper {
                weight_gsm: 120.,
                rgb: (
                    color.r() as f64 / 255.0,
                    color.g() as f64 / 255.0,
                    color.b() as f64 / 255.0,
                ),
                size: self.paper_size.clone(),
                orientation: self.paper_orientation.clone(),
            });
            // println!("COLOR paper out {:?}", paper_out);
            cmd_out
                .send(paper_out)
                .expect("Failed to send SetPaper command?");
        };
        self.request_new_source_image();
    }

    pub fn set_paper_size(&mut self, paper_size: &PaperSize, create_history: bool) {
        self.paper_size = paper_size.clone();
        if let Some(cmd_out) = &self.cmd_out
            && create_history
        {
            let color = self.paper_color.clone();
            let paper_out = ViewCommand::SetPaper(Paper {
                weight_gsm: 120.,
                rgb: (
                    color.r() as f64 / 255.0,
                    color.g() as f64 / 255.0,
                    color.b() as f64 / 255.0,
                ),
                size: self.paper_size.clone(),
                orientation: self.paper_orientation.clone(),
            });
            // println!("SIZE Paper out: {:?}", paper_out);
            cmd_out
                .send(paper_out)
                .expect("Failed to send SetPaper command?");
        };
    }

    pub fn set_paper_orientation(&mut self, orientation: &Orientation, create_history: bool) {
        // println!("Setting paper orientation: {:?}", orientation);
        self.paper_orientation = orientation.clone();
        if let Some(cmd_out) = &self.cmd_out
            && create_history
        {
            let color = self.paper_color.clone();
            let paper_out = ViewCommand::SetPaper(Paper {
                weight_gsm: 120.,
                rgb: (
                    color.r() as f64 / 255.0,
                    color.g() as f64 / 255.0,
                    color.b() as f64 / 255.0,
                ),
                size: self.paper_size.clone(),
                orientation: self.paper_orientation.clone(),
            });
            // println!("ORIENTATION Paper out: {:?}", paper_out);
            cmd_out
                .send(paper_out)
                .expect("Failed to send SetPaper command?");
        };
    }
}
