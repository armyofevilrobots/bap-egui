use std::path::PathBuf;

use crate::core::{
    machine::MachineConfig,
    project::{Orientation, Paper, PenDetail, Project},
};

use super::BAPViewModel;

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct ViewModelPatch {
    pub pens: Option<Vec<PenDetail>>,
    pub paper: Option<Paper>,
    pub origin: Option<(f64, f64)>, // Target/center of the viewport
    pub extents: Option<(f64, f64, f64, f64)>,
    pub machine_config: Option<MachineConfig>,
    pub program: Option<Box<Vec<String>>>,
    pub file_path: Option<PathBuf>,
}

impl From<Project> for ViewModelPatch {
    fn from(project: Project) -> Self {
        let extents = project.extents();
        Self {
            pens: Some(project.pens.clone()),
            paper: Some(project.paper.clone()),
            origin: project.origin.clone(),
            extents: Some((
                extents.min().x,
                extents.min().y,
                extents.max().x,
                extents.max().y,
            )),
            machine_config: project.machine(),
            program: project.program(),
            file_path: project.file_path,
        }
    }
}
