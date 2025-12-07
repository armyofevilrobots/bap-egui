use aoer_plotty_rs::geo_types::matrix::TransformGeometry;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::{Coord, Geometry, Point, Rect, Rotate, Scale, Translate};
use nalgebra::Affine2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GeometryKind {
    Stroke(Geometry),
    Hatch(Geometry),
}

impl GeometryKind {
    pub fn bounding_rect(&self) -> Option<Rect> {
        self.geometry().bounding_rect()
    }

    pub fn transformed(&self, tx: &Affine2<f64>) -> GeometryKind {
        let inner_geo = self.geometry().transformed(tx);
        match self {
            GeometryKind::Hatch(_old_geo) => GeometryKind::Hatch(inner_geo),
            GeometryKind::Stroke(_old_geo) => GeometryKind::Stroke(inner_geo),
        }
    }

    pub fn rotate_around_point(&self, degrees: f64, around: impl Into<Point<f64>>) -> GeometryKind {
        let mut inner_geo = self.geometry().clone();
        inner_geo.rotate_around_point_mut(degrees, around.into());
        match self {
            GeometryKind::Hatch(_old_geo) => GeometryKind::Hatch(inner_geo),
            GeometryKind::Stroke(_old_geo) => GeometryKind::Stroke(inner_geo),
        }
    }

    pub fn geometry(&self) -> &Geometry {
        match self {
            GeometryKind::Stroke(geo) => geo,
            GeometryKind::Hatch(geo) => geo,
        }
    }

    pub fn geometry_mut(&mut self) -> &mut Geometry {
        match self {
            GeometryKind::Stroke(geo) => geo,
            GeometryKind::Hatch(geo) => geo,
        }
    }

    pub fn rotate_around_point_mut(&mut self, degrees: f64, around: impl Into<Point<f64>>) {
        self.geometry_mut()
            .rotate_around_point_mut(degrees, around.into());
    }

    pub fn scale_around_point_mut(
        &mut self,
        xscale: f64,
        yscale: f64,
        around: impl Into<Coord<f64>>,
    ) {
        self.geometry_mut()
            .scale_around_point_mut(xscale, yscale, around)
    }

    pub fn translate_mut(&mut self, x: f64, y: f64) {
        match self {
            GeometryKind::Stroke(geometry) => geometry.translate_mut(x, y),
            GeometryKind::Hatch(geometry) => geometry.translate_mut(x, y),
        }
    }
}
