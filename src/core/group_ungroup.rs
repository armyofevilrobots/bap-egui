use aoer_plotty_rs::context::pgf_file::PlotGeometry;

use super::commands::ApplicationStateChangeMsg;
use geo::{Geometry, MultiLineString};

use super::ApplicationCore;
use crate::view_model::view_model_patch::ViewModelPatch;

impl ApplicationCore {
    pub fn apply_ungroup(&mut self) {
        // println!("Would have ungrouped {:?}", self.picked);
        if let Some(picked) = &self.picked {
            // Make copies of all the stuff we're breaking up.
            let geo_items: Vec<PlotGeometry> = picked
                .iter()
                .filter_map(|idx| self.project.geometry.get(*idx as usize))
                .map(|item| item.clone())
                .collect();
            // Then remove them from the geometry list. We reverse the order
            // to prevent shrinking and removing the wrong shit.
            for idx in picked.iter().rev() {
                self.project.geometry.remove(*idx as usize);
            }

            for geo in geo_items {
                let geo = geo.clone();
                match geo.geometry {
                    Geometry::MultiLineString(mls) => {
                        for linestring in mls.0 {
                            self.project.geometry.push(PlotGeometry {
                                geometry: Geometry::MultiLineString(MultiLineString::new(vec![
                                    linestring,
                                ])),
                                // id: u32::MAX as u64,
                                stroke: geo.stroke.clone(),
                                keepdown_strategy: geo.keepdown_strategy.clone(),
                            });
                        }
                    }
                    _ => (),
                }
            }
            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(None))
                .expect("OMFG ViewModel is borked sending pick id");
            self.picked = None;
            self.pick_image = None;
            self.rebuild_after_content_change();
            self.state_change_out
                .send(ApplicationStateChangeMsg::PatchViewModel(
                    ViewModelPatch::from(self.project.clone()),
                ))
                .expect("Failed to send error to viewmodel.");
            self.ctx.request_repaint();
        }
    }

    pub fn apply_group(&mut self) {
        if let Some(picked) = &self.picked {
            if picked.len() < 2 {
                return;
            }
            // Make copies of all the stuff we're breaking up.
            let picked_items: Vec<PlotGeometry> = picked
                .iter()
                .filter_map(|idx| self.project.geometry.get(*idx as usize))
                .map(|item| item.clone())
                .collect();
            // Then remove them from the geometry list. We reverse the order
            // to prevent shrinking and removing the wrong shit.
            for idx in picked.iter().rev() {
                self.project.geometry.remove(*idx as usize);
            }

            let mut new_mls: MultiLineString<f64> = MultiLineString::new(Vec::new());
            let tmp_geo = picked_items.first().unwrap().clone();

            for geo in picked_items {
                let geo = geo.clone();
                match geo.geometry {
                    Geometry::MultiLineString(mls) => {
                        new_mls.0.extend(mls.0);
                    }
                    _ => (),
                }
            }
            self.project.geometry.push(PlotGeometry {
                geometry: Geometry::MultiLineString(new_mls),
                // id: u32::MAX as u64,
                stroke: tmp_geo.stroke.clone(),
                keepdown_strategy: tmp_geo.keepdown_strategy.clone(),
            });

            self.state_change_out
                .send(ApplicationStateChangeMsg::Picked(None))
                .expect("OMFG ViewModel is borked sending pick id");
            self.picked = None;
            self.pick_image = None;
            self.rebuild_after_content_change();
            self.state_change_out
                .send(ApplicationStateChangeMsg::PatchViewModel(
                    ViewModelPatch::from(self.project.clone()),
                ))
                .expect("Failed to send error to viewmodel.");
            self.ctx.request_repaint();
        }
    }
}
