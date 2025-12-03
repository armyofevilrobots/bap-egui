use aoer_plotty_rs::plotter::pen::PenDetail;

use crate::view_model::view_model_patch::ViewModelPatch;

use super::ApplicationCore;
use super::ApplicationStateChangeMsg;
use super::PICKED_ROTATE_TIME;
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

impl ApplicationCore {
    pub fn apply_pen_to_selection(&mut self, pen_id: usize) {
        self.checkpoint();
        if let Some(picked) = &self.picked {
            let mut pen = PenDetail::default();
            for p in self.project.pens.clone() {
                if p.tool_id == pen_id {
                    pen = p.clone();
                }
            }

            for (idx, geo) in self.project.plot_geometry.iter_mut().enumerate() {
                if picked.contains(&(idx as u32)) {
                    geo.pen_uuid = pen.identity;
                }
            }

            self.state_change_out
                .send(ApplicationStateChangeMsg::PatchViewModel(
                    ViewModelPatch::from(self.project.clone()),
                ))
                .expect("Failed to send error to viewmodel.");
            self.rebuild_after_content_change();
            self.ctx.request_repaint();
        }
    }

    pub fn delete_selection(&mut self) {
        self.checkpoint();
        if let Some(picked) = &self.picked {
            for idx in (0..self.project.plot_geometry.len()).rev() {
                if picked.contains(&(idx as u32)) {
                    self.project.plot_geometry.remove(idx);
                }
            }
            self.state_change_out
                .send(ApplicationStateChangeMsg::PatchViewModel(
                    ViewModelPatch::from(self.project.clone()),
                ))
                .expect("Failed to send error to viewmodel.");
            self.clear_pick();
            self.rebuild_after_content_change();
            self.ctx.request_repaint();
        }
    }

    pub fn send_pick_changed(&self) {
        self.state_change_out
            .send(ApplicationStateChangeMsg::Picked(Some(
                self.picked
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|i| *i as usize)
                    .collect::<Vec<usize>>(),
            )))
            .expect("OMFG ViewModel is borked sending pick id");
    }

    pub fn select_by_color_at(&mut self, x: f64, y: f64) {
        // println!("!PICK COLOR @{},{}", x, y);
        if let Some(pick) = self.try_pick(x, y) {
            // println!("GOT PICK OF {}", pick);
            if let Some(geo) = self.project.plot_geometry.get(pick as usize) {
                // println!("Got picked geo: {:?}", geo);
                // if let Some(stroke_sel) = self.project.pen_by_uuid(geo.pen_uuid) {
                //&geo.stroke {
                // println!("Stroke sel: {:?}", stroke_sel);
                let mut new_picked = BTreeSet::new();
                for (idx, other_geo) in self.project.plot_geometry.iter().enumerate() {
                    // if let Some(other_stroke) = &other_geo.stroke {
                    // if other_stroke.tool_id == stroke_sel.tool_id {
                    //     // println!("Matched a stroke at {}", stroke_sel.tool_id);
                    //     new_picked.insert(idx as u32);
                    // }
                    // }
                    if other_geo.pen_uuid == geo.pen_uuid {
                        new_picked.insert(idx as u32);
                    }
                }
                if new_picked.len() > 0 {
                    self.picked = Some(new_picked);
                    self.send_pick_changed();
                }
                // }
            }
        }
    }

    pub fn add_pick_at(&mut self, x: f64, y: f64) {
        let picked = self.try_pick(x, y);
        if let Some(id) = picked {
            if self.picked.is_none() {
                self.picked = Some(BTreeSet::new());
            }

            self.picked.as_mut().unwrap().insert(id as u32);
            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(Some(
                    self.picked
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|i| *i as usize)
                        .collect::<Vec<usize>>(),
                )))
                .expect("OMFG ViewModel is borked sending pick id");
        } else {
            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(None))
                .unwrap_or_else(|_err| {
                    if self.shutdown {
                        eprintln!("Cannot update pick image while shutting down...")
                    } else {
                        eprintln!("Cannot send pick image because ViewModel has hung up.")
                    }
                });
            // .expect("OMFG ViewModel is borked sending pick id");
        }
        self.last_rendered =
            Instant::now() + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
    }

    pub fn try_pick_at(&mut self, x: f64, y: f64) {
        let picked = self.try_pick(x, y);
        if let Some(id) = picked {
            if self.picked.is_none() {
                self.picked = Some(BTreeSet::new());
            }

            self.picked.as_mut().unwrap().insert(id as u32);
            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(Some(
                    self.picked
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|i| *i as usize)
                        .collect::<Vec<usize>>(),
                )))
                .expect("OMFG ViewModel is borked sending pick id");
        } else {
            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(None))
                .unwrap_or_else(|_err| {
                    if self.shutdown {
                        eprintln!("Cannot update pick image while shutting down...")
                    } else {
                        eprintln!("Cannot send pick image because ViewModel has hung up.")
                    }
                });
            // .expect("OMFG ViewModel is borked sending pick id");
            self.picked = None
        }
        self.last_rendered =
            Instant::now() + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
    }

    pub fn clear_pick(&mut self) {
        self.state_change_out
            .send(ApplicationStateChangeMsg::Picked(None))
            .expect("OMFG ViewModel is borked sending pick id");
        self.picked = None
    }

    pub fn toggle_pick_at(&mut self, x: f64, y: f64) {
        let picked = self.try_pick(x, y);
        if let Some(id) = picked {
            //
            if self.picked.is_none() {
                self.picked = Some(BTreeSet::new());
            }

            if self.picked.as_mut().unwrap().contains(&id) {
                self.picked.as_mut().unwrap().remove(&id);
            } else {
                self.picked.as_mut().unwrap().insert(id.clone());
            }
            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(Some(
                    self.picked
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|i| *i as usize)
                        .collect::<Vec<usize>>(),
                )))
                .expect("OMFG ViewModel is borked sending pick id");
        }
        self.last_rendered =
            Instant::now() + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
    }

    pub fn select_all(&mut self) {
        self.picked = Some(BTreeSet::from_iter(
            (0..self.project.plot_geometry.len()).map(|i| i as u32),
        ));
        self.last_rendered =
            Instant::now() + Duration::from_millis((PICKED_ROTATE_TIME * 1000.) as u64);
    }
}
