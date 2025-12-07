use crate::core::post::GeometryToMultiLineString;

// use aoer_plotty_rs::geo_types::hatch::Hatches;
use super::GeometryKind;
pub use aoer_plotty_rs::context::pgf_file::*;
use geo::{Coord, Geometry, MultiLineString, Point, Rotate, Scale, Translate};
use nalgebra::Affine2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BAPGeometry {
    pub pen_uuid: Uuid,
    #[serde(default)]
    pub name: String,
    pub geometry: GeometryKind,
    pub keepdown_strategy: KeepdownStrategy,
}

impl BAPGeometry {
    pub fn transformed(&self, tx: &Affine2<f64>) -> BAPGeometry {
        BAPGeometry {
            pen_uuid: self.pen_uuid,
            geometry: self.geometry.transformed(tx),
            keepdown_strategy: self.keepdown_strategy,
            name: self.name.clone(),
        }
    }

    #[allow(unused)]
    pub fn rotate_around_point_mut(&mut self, degrees: f64, around: impl Into<Point<f64>>) {
        self.geometry_mut()
            .rotate_around_point_mut(degrees, around.into());
    }

    #[allow(unused)]
    pub fn scale_around_point_mut(&mut self, xs: f64, ys: f64, around: impl Into<Coord<f64>>) {
        self.geometry_mut().scale_around_point_mut(xs, ys, around);
    }

    #[allow(unused)]
    pub fn translate_mut(&mut self, x: f64, y: f64) {
        self.geometry_mut().translate_mut(x, y);
    }

    pub fn lines(&self) -> MultiLineString {
        match &self.geometry {
            GeometryKind::Stroke(geometry) => geometry.to_multi_line_strings(),
            GeometryKind::Hatch(geometry) => geometry.to_multi_line_strings(),
        }
    }
    pub fn geometry(&self) -> &Geometry {
        self.geometry.geometry()
    }

    pub fn geometry_mut(&mut self) -> &mut Geometry {
        self.geometry.geometry_mut()
    }
}
