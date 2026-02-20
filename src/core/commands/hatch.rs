use aoer_plotty_rs::prelude::LineHatch;
use std::{fmt::Debug, sync::Arc};

use aoer_plotty_rs::prelude::HatchPattern;
#[derive(Clone, Debug)]
pub struct HatchConfig {
    hatch: Arc<dyn HatchPattern>,
    angle: f64,
    scale: f64,
    pen: f64,
}

impl PartialEq for HatchConfig {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self.hatch) == format!("{:?}", other.hatch)
            && self.angle == other.angle
            && self.scale == other.scale
            && self.pen == other.pen
    }
}

impl Default for HatchConfig {
    fn default() -> Self {
        Self {
            hatch: LineHatch::create(),
            angle: 45.,
            scale: 1.,
            pen: 1.,
        }
    }
}
