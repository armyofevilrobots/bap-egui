use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Pos2, Rect, TextureHandle, Vec2, pos2, vec2};
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::core::commands::{ApplicationStateChangeMsg, ViewCommand};

use crate::core::machine::MachineConfig;
use crate::core::project::{Orientation, PaperSize, PenDetail};
use crate::sender::{PlotterResponse, PlotterState};

#[derive(PartialEq, Clone, Debug)]
pub enum CommandContext {
    Origin,
    PaperChooser,
    PenCrib,
    PenEdit(usize),   // The pen index in Vec<Pens>
    PenDelete(usize), // Delete the pen at IDX via modal confirmation
    Clip(Option<Pos2>, Option<Pos2>),
    Scale(f64),
    None,
}
