use super::commands::ApplicationStateChangeMsg;
use geo::{Geometry, MultiLineString};

use super::ApplicationCore;
use crate::{
    core::{
        post::GeometryToMultiLineString,
        project::{BAPGeometry, GeometryKind},
    },
    view_model::view_model_patch::ViewModelPatch,
};

impl ApplicationCore {
    pub fn apply_ungroup(&mut self) {
        // println!("Would have ungrouped {:?}", self.picked);
        if let Some(picked) = &self.picked {
            // Make copies of all the stuff we're breaking up.
            let geo_items: Vec<BAPGeometry> = picked
                .iter()
                .filter_map(|idx| self.project.plot_geometry.get(*idx as usize))
                .map(|item| item.clone())
                .collect();
            // Then remove them from the geometry list. We reverse the order
            // to prevent shrinking and removing the wrong shit.
            for idx in picked.iter().rev() {
                self.project.plot_geometry.remove(*idx as usize);
            }

            for geo in geo_items {
                let new_geokind = match geo.geometry {
                    GeometryKind::Stroke(geoms) => GeometryKind::Stroke(Geometry::MultiLineString(
                        geoms.to_multi_line_strings(),
                    )),
                    GeometryKind::Hatch(geoms) => GeometryKind::Hatch(Geometry::MultiLineString(
                        geoms.to_multi_line_strings(),
                    )),
                };
                self.project.plot_geometry.push(BAPGeometry {
                    name: geo.name,
                    pen_uuid: geo.pen_uuid,
                    geometry: new_geokind,
                    keepdown_strategy: geo.keepdown_strategy,
                })
                // let geo = geo.clone();
                // match geo.geometry() {
                // Geometry::MultiLineString(mls) => {
                //     for linestring in mls.0 {
                //         self.project.plot_geometry.push(BAPGeometry { pen_uuid: (), geometry: () } {
                //             geometry: Geometry::MultiLineString(MultiLineString::new(vec![
                //                 linestring,
                //             ])),
                //             // id: u32::MAX as u64,
                //             stroke: geo.stroke.clone(),
                //             keepdown_strategy: geo.keepdown_strategy.clone(),
                //         });
                // }
                // }
                // _ => (),
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
            let picked_items: Vec<BAPGeometry> = picked
                .iter()
                .filter_map(|idx| self.project.plot_geometry.get(*idx as usize))
                .map(|item| item.clone())
                .collect();
            // Then remove them from the geometry list. We reverse the order
            // to prevent shrinking and removing the wrong shit.
            for idx in picked.iter().rev() {
                self.project.plot_geometry.remove(*idx as usize);
            }

            let mut new_mls: MultiLineString<f64> = MultiLineString::new(Vec::new());
            // TODO: We should find the first stroke and hatch, and group down to TWO geometries, one of each
            let tmp_geo = picked_items.first().unwrap().clone();

            for geo in picked_items {
                let geo = geo.clone();
                new_mls.0.extend(geo.geometry().to_multi_line_strings());
                // match geo.geometry() {
                //     Geometry::MultiLineString(mls) => {
                //         new_mls.0.extend(mls.0);
                //     }
                //     _ => (),
                // }
            }
            self.project.plot_geometry.push(BAPGeometry {
                name: tmp_geo.name,
                pen_uuid: tmp_geo.pen_uuid,
                geometry: GeometryKind::Stroke(Geometry::MultiLineString(new_mls)),
                keepdown_strategy: tmp_geo.keepdown_strategy,
            });
            // {
            //     geometry: geometry::multilinestring(new_mls),
            //     // id: u32::max as u64,
            //     stroke: tmp_geo.stroke.clone(),
            //     keepdown_strategy: tmp_geo.keepdown_strategy.clone(),
            // });

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
