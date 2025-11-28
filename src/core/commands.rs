use egui::ColorImage;
use std::path::PathBuf;

use crate::{
    core::{
        config::AppConfig,
        machine::MachineConfig,
        project::{Paper, PenDetail},
        sender::{PlotterResponse, PlotterState},
    },
    view_model::view_model_patch::ViewModelPatch,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub enum ViewCommand {
    #[default]
    Ping,
    // ZoomView(f64), // Measured in pixels per mm. The view will calculate exactly how big.
    RequestSourceImage {
        //     extents: (f64, f64, f64, f64),
        zoom: f64,
        // resolution: (usize, usize),
        rotation: Option<((f64, f64), f64)>,
    },
    RequestPlotPreviewImage {
        extents: (f64, f64, f64, f64),
        resolution: (usize, usize),
    },
    ImportSVG(PathBuf),
    SetOrigin(f64, f64),
    SetPaper(Paper),
    UpdateMachineConfig(MachineConfig),
    SetClipBoundary {
        min: (f64, f64),
        max: (f64, f64),
    },
    RotateSource {
        center: (f64, f64),
        degrees: f64,
    },
    Scale(f64),
    Post,
    StartPlot,
    PausePlot,
    CancelPlot,
    PenUp,
    PenDown,
    SendCommand(String),
    ConnectPlotter(String),
    DisconnectPlotter,
    Quit,
    ApplyPens(Vec<PenDetail>),
    ApplyPenToSelection(usize), // Tool ID.
    Undo,
    ResetProject,
    LoadProject(PathBuf),
    LoadPGF(PathBuf),
    SaveProject(Option<PathBuf>),
    TryPickAt(f64, f64),
    AddPickAt(f64, f64),
    TogglePickAt(f64, f64),
    PickByColorAt(f64, f64),
    SelectAll,
    ClearPick,
    UnGroup,
    Group,
    DeleteSelection,
    UpdateConfig(AppConfig),
    Translate(f64, f64),
    None,
}

#[derive(Debug, PartialEq, Default)]
#[allow(dead_code)]
pub enum ApplicationStateChangeMsg {
    #[default]
    Pong,
    NotifyConfig(AppConfig),
    UpdateSourceImage {
        image: ColorImage,             // The image to draw.
        extents: (f64, f64, f64, f64), //Min.x, Min.y, width, height
        rotation: Option<((f64, f64), f64)>,
    },
    TransformPreviewImage {
        image: ColorImage,
        extents: (f64, f64, f64, f64),
    },
    SourceChanged {
        extents: (f64, f64, f64, f64),
    },
    PlotPreviewChanged {
        extents: (f64, f64, f64, f64),
    },
    OriginChanged(f64, f64),

    UpdateMachineConfig(MachineConfig),
    ResetDisplay,
    Dead,
    ProgressMessage {
        message: String,
        percentage: usize,
    },
    PlotterState(PlotterState),
    PlotterResponse(PlotterResponse),
    FoundPorts(Vec<String>),
    PostComplete(usize),
    Error(String),
    UndoAvailable(bool),
    PaperChanged(Paper),
    PatchViewModel(ViewModelPatch),
    Picked(Option<Vec<usize>>),
    None,
}
