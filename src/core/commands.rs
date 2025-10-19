use egui::{ColorImage, Vec2};
use std::path::PathBuf;

use crate::machine::MachineConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ViewCommand {
    #[default]
    Ping,
    ZoomView(f64), // Measured in pixels per mm. The view will calculate exactly how big.
    ImportSVG(PathBuf),
    SetOrigin(f64, f64),
    UpdateMachineConfig(MachineConfig),
    SetClipBoundary {
        min: (f64, f64),
        max: (f64, f64),
    },
    RotateSource {
        center: (f64, f64),
        theta: f64,
    },
    Post,
    StartPlot,
    PausePlot,
    CancelPlot,
    Quit,
    None,
}

#[derive(Debug, PartialEq, Default)]
pub enum ApplicationStateChangeMsg {
    #[default]
    Pong,
    UpdateSVGImage {
        image: ColorImage, // The image to draw.
        min: (f64, f64),   // How big it is
        size: (f64, f64),
        zoom: f64, // What zoom level we drew this for
    },
    UpdateMachineConfig(MachineConfig),
    ResetDisplay,
    Dead,
    None,
}
