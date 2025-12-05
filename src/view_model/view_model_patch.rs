use std::path::PathBuf;

use egui::{Color32, ColorImage};
use uuid::Uuid;

use crate::core::{
    machine::MachineConfig,
    project::{Paper, PenDetail, Project},
};

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct ViewModelPatch {
    pub pens: Option<Vec<PenDetail>>,
    pub paper: Option<Paper>,
    pub origin: Option<Option<(f64, f64)>>, // Target/center of the viewport
    pub extents: Option<(f64, f64, f64, f64)>,
    pub machine_config: Option<Option<MachineConfig>>,
    pub program: Option<Option<Box<Vec<String>>>>,
    pub file_path: Option<Option<PathBuf>>,
    pub geo_layers: Option<Vec<(String, ColorImage, Uuid)>>, //Option<Vec<(String, Box<ColorImage>, Uuid)>>,
}

impl From<Project> for ViewModelPatch {
    fn from(project: Project) -> Self {
        let extents = project.extents();
        // println!("Patching with extents: {:?}", extents);
        Self {
            pens: Some(project.pens.clone()),
            paper: Some(project.paper.clone()),
            origin: Some(project.origin.clone()),
            extents: Some((
                extents.min().x,
                extents.min().y,
                extents.max().x,
                extents.max().y,
            )),
            machine_config: Some(project.machine()),
            program: Some(project.program()),
            file_path: Some(project.file_path),
            geo_layers: Some(
                project
                    .plot_geometry
                    .iter()
                    .enumerate()
                    .map(|(_idx, item)| {
                        (
                            item.name.clone(),
                            ColorImage::filled([32, 32], Color32::LIGHT_GRAY),
                            item.pen_uuid.clone(),
                        )
                    })
                    .collect(),
            ),
        }
    }
}
