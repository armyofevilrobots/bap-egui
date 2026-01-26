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
pub enum SelectionType {
    Hatches,
    #[default]
    Strokes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum MatTarget {
    // Order is top right bottom left
    Machine(f64, f64, f64, f64),
    Paper(f64, f64, f64, f64),
}

impl Default for MatTarget {
    fn default() -> Self {
        MatTarget::Paper(10., 10., 10., 10.)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub enum ViewCommand {
    #[default]
    Ping,
    MergePens,
    ReorderToDestination(usize),
    OrderByPenId,
    SetGCode(String),
    RequestSourceImage {
        zoom: f64,
        rotation: Option<((f64, f64), f64)>,
        translation: Option<(f64, f64)>,
        scale_around: Option<((f64, f64), f64)>,
    },
    RequestPlotPreviewImage {
        extents: (f64, f64, f64, f64),
        resolution: (usize, usize),
    },
    ImportSVG(PathBuf),
    SetOrigin(f64, f64),
    SetPaper(Paper),
    UpdateMachineConfig(MachineConfig),
    LoadMachineConfig(PathBuf),
    SaveMachineConfig(PathBuf),
    SetClipBoundary {
        min: (f64, f64),
        max: (f64, f64),
    },
    RotateSource {
        center: (f64, f64),
        degrees: f64,
    },
    ScaleAround {
        center: (f64, f64),
        factor: f64,
    },
    Scale(f64),
    // Scale so the whole drawing fits with AT LEAST this margin on printable area/paper
    ScaleMatTo(MatTarget),
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
    TogglePickByIndex(usize),
    PickByType(SelectionType),
    ForcePick(Vec<usize>),
    InvertPick,
    SelectAll,
    ClearPick,
    UnGroup,
    Group,
    DeleteSelection,
    UpdateConfig(AppConfig),
    Translate(f64, f64),
    RenameLayer {
        id: usize,
        name: String,
    },
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
    GCode(Option<String>),
    None,
}
