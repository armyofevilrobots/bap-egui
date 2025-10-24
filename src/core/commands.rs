use egui::{ColorImage, Vec2};
use std::path::PathBuf;

use crate::{machine::MachineConfig, sender::PlotterState};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ViewCommand {
    #[default]
    Ping,
    // ZoomView(f64), // Measured in pixels per mm. The view will calculate exactly how big.
    RequestSourceImage {
        extents: (f64, f64, f64, f64),
        resolution: (usize, usize),
    },
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
    SendCommand(String),
    ConnectPlotter(String),
    DisconnectPlotter,
    Quit,
    None,
}

#[derive(Debug, PartialEq, Default)]
pub enum ApplicationStateChangeMsg {
    #[default]
    Pong,
    UpdateSourceImage {
        image: ColorImage,             // The image to draw.
        extents: (f64, f64, f64, f64), //Min.x, Min.y, width, height
    },
    SourceChanged {
        extents: (f64, f64, f64, f64),
    },
    UpdateMachineConfig(MachineConfig),
    ResetDisplay,
    Dead,
    ProgressMessage {
        message: String,
        percentage: usize,
    },
    PlotterState(PlotterState),
    FoundPorts(Vec<String>),
    None,
}
