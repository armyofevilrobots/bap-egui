// use aoer_plotty_rs::geo_types::hatch::Hatches;
pub use super::BAPGeometry;
pub use super::Project;
use geo::{Rect, coord};

impl Project {
    pub fn extents(&self) -> Rect {
        self.extents.clone()
    }

    pub fn calc_extents_for_geometry(geometry: &Vec<BAPGeometry>) -> Rect {
        if geometry.len() == 0 {
            return Rect::new(coord! {x: -1., y: -1.}, coord! {x: 1., y: 1.});
        }
        let mut xmin = f64::MAX;
        let mut xmax = f64::MIN;
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        for geo in geometry {
            // Only update extents if the geometry is rational and non-empty.
            if let Some(tmp_extents) = geo.geometry.bounding_rect() {
                if tmp_extents.min().x < xmin {
                    xmin = tmp_extents.min().x;
                }
                if tmp_extents.min().y < ymin {
                    ymin = tmp_extents.min().y;
                }
                if tmp_extents.max().x > xmax {
                    xmax = tmp_extents.max().x;
                }
                if tmp_extents.max().y > ymax {
                    ymax = tmp_extents.max().y;
                }
            }
        }
        if xmax - xmin == 0. || ymax - ymin == 0. {
            xmin = -1.;
            ymin = -1.;
            xmax = 1.;
            ymax = 1.;
        }
        let extents = Rect::new(coord! {x: xmin, y:ymin}, coord! {x:xmax, y: ymax});
        // println!("Returning calculated extents of {:?}", &extents);
        extents
    }

    /// Returns top left, bottom right as 4 f64s.
    pub fn calc_extents(&self) -> Rect {
        Self::calc_extents_for_geometry(&self.plot_geometry)
    }

    pub fn regenerate_extents(&mut self) {
        self.extents = self.calc_extents();
    }
}
