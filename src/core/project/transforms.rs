use super::BAPGeometry;
// use aoer_plotty_rs::geo_types::hatch::Hatches;
use geo::Point;
use nalgebra::{Affine2, Matrix3};
use std::collections::BTreeSet;

impl super::Project {
    pub fn rotate_geometry_around_point(
        &self,
        around: (f64, f64),
        angle: f64,
        picked: &Option<BTreeSet<u32>>,
    ) -> Vec<BAPGeometry> {
        let (xc, yc) = around;
        self.plot_geometry
            .iter()
            .enumerate()
            .map(|(idx, pg)| {
                let new_geo = if let Some(pick) = picked {
                    if pick.contains(&(idx as u32)) {
                        pg.geometry.rotate_around_point(angle, Point::new(xc, yc))
                    } else {
                        pg.geometry.clone()
                    }
                } else {
                    pg.geometry.rotate_around_point(angle, Point::new(xc, yc))
                };
                BAPGeometry {
                    name: format!("geometry {}", idx).to_string(),
                    geometry: new_geo,
                    pen_uuid: pg.pen_uuid,
                    keepdown_strategy: pg.keepdown_strategy,
                }
            })
            .collect()
    }

    #[allow(unused)]
    fn scale_native_units(units: &str) -> f64 {
        if units == "mm" {
            1.
        } else if units == "in" {
            25.4
        } else if units == "pt" {
            25.4 / 72.
        } else if units == "cm" {
            10.
        } else {
            25.4 / 96.
        }
    }

    /// Scales the entire set of geometries by some factor.
    pub fn scale_by_factor(&mut self, factor: f64) {
        let tx_affine2 = Affine2::<f64>::from_matrix_unchecked(Matrix3::new(
            factor, 0., 0., 0., factor, 0., 0., 0., 1.,
        ));
        for geo in self.plot_geometry.iter_mut() {
            *geo = geo.transformed(&tx_affine2);
        }
        self.regenerate_extents();
    }

    pub fn dims_from_dimattr(attr: &str) -> Option<(f64, &str)> {
        let units_re =
            regex::Regex::new(r"^(?P<val>[0-9]+\.?[0-9]*)\s*(?P<units>[a-zA-Z]*)").unwrap();
        if let Some(captures) = units_re.captures(attr) {
            if let Some(value) = captures.name("val") {
                if let Some(units) = captures.name("units") {
                    if let Ok(value) = value.as_str().parse::<f64>() {
                        return Some((value, units.as_str()));
                    }
                }
            }
        }
        return None;
    }

    pub fn translate_arbitrary_geo(
        geo: &Vec<BAPGeometry>,
        translation: (f64, f64),
        picked: &Option<BTreeSet<u32>>,
    ) -> Vec<BAPGeometry> {
        let mut geo_out = geo.clone();
        for (idx, geometry) in geo_out.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry
                        .geometry
                        .translate_mut(translation.0, translation.1);
                }
            } else {
                geometry
                    .geometry
                    .translate_mut(translation.0, translation.1);
            }
        }
        return geo_out;
    }

    /// Translates all geometry.
    pub fn translate_geometry_mut(
        &mut self,
        translation: (f64, f64),
        picked: &Option<BTreeSet<u32>>,
    ) {
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry
                        .geometry
                        .translate_mut(translation.0, translation.1);
                }
            } else {
                geometry
                    .geometry
                    .translate_mut(translation.0, translation.1);
            }
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
    }

    /// Scale all geometry around a given point.
    pub fn scale_geometry_around_point(
        geo: &Vec<BAPGeometry>,
        center: (f64, f64),
        scale: f64,
        picked: &Option<BTreeSet<u32>>,
    ) -> Vec<BAPGeometry> {
        let mut geo = geo.clone();
        for (idx, plotgeo) in geo.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    plotgeo.geometry.scale_around_point_mut(
                        scale,
                        scale,
                        Point::new(center.0, center.1),
                    );
                }
            } else {
                plotgeo.geometry.scale_around_point_mut(
                    scale,
                    scale,
                    Point::new(center.0, center.1),
                );
            }
        }
        geo
    }

    /// Scale all geometry around a given point.
    pub fn scale_geometry_around_point_mut(
        &mut self,
        center: (f64, f64),
        scale: f64,
        picked: &Option<BTreeSet<u32>>,
    ) {
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry.geometry.scale_around_point_mut(
                        scale,
                        scale,
                        Point::new(center.0, center.1),
                    );
                }
            } else {
                geometry.geometry.scale_around_point_mut(
                    scale,
                    scale,
                    Point::new(center.0, center.1),
                );
            }
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
    }

    /// Rotates all geometry around a given point.
    pub fn rotate_geometry_around_point_mut(
        &mut self,
        center: (f64, f64),
        degrees: f64,
        picked: &Option<BTreeSet<u32>>,
    ) {
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry
                        .geometry
                        .rotate_around_point_mut(degrees, Point::new(center.0, center.1));
                }
            } else {
                geometry
                    .geometry
                    .rotate_around_point_mut(degrees, Point::new(center.0, center.1));
            }
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
    }
}
