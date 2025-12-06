use std::{collections::HashMap, path::PathBuf};

use egui::ColorImage;
use uuid::Uuid;

use crate::core::{
    machine::MachineConfig,
    project::{Paper, PenDetail, Project},
    render_preview::render_layer_preview,
};

#[derive(Default, Clone, PartialEq)]
pub(crate) struct ViewModelPatch {
    pub pens: Option<Vec<PenDetail>>,
    pub paper: Option<Paper>,
    pub origin: Option<Option<(f64, f64)>>, // Target/center of the viewport
    pub extents: Option<(f64, f64, f64, f64)>,
    pub machine_config: Option<Option<MachineConfig>>,
    pub program: Option<Option<Box<Vec<String>>>>,
    pub file_path: Option<Option<PathBuf>>,
    pub geo_layers: Option<Vec<(String, Box<ColorImage>, Uuid)>>, //Option<Vec<(String, Box<ColorImage>, Uuid)>>,
}

impl std::fmt::Debug for ViewModelPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tmp_layers = match &self.geo_layers {
            Some(layers) => &format!("{} layers", layers.len()).to_string(),
            None => &format!("No layers").to_string(),
        };
        f.debug_struct("ViewModelPatch")
            .field("pens", &self.pens)
            .field("paper", &self.paper)
            .field("origin", &self.origin)
            .field("extents", &self.extents)
            .field("machine_config", &self.machine_config)
            .field("program", &self.program)
            .field("file_path", &self.file_path)
            .field("geo_layers", &tmp_layers)
            .finish()
    }
}

impl From<Project> for ViewModelPatch {
    fn from(project: Project) -> Self {
        let extents = project.extents();
        // println!("Patching with extents: {:?}", extents);
        let pens = project.pens.clone();
        let pen_map: HashMap<Uuid, PenDetail> =
            HashMap::from_iter(pens.iter().map(|pen| (pen.identity, pen.clone())));
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
                        let rendered = render_layer_preview(
                            item,
                            // &project.pen_by_uuid(item.pen_uuid).clone().unwrap(),
                            pen_map.get(&item.pen_uuid).unwrap_or(&PenDetail::default()),
                            &[32, 32],
                        )
                        .unwrap();

                        (
                            item.name.clone(),
                            Box::new(rendered),
                            // ColorImage::filled([32, 32], Color32::LIGHT_GRAY),
                            // .unwrap_or(ColorImage::filled([32, 32], Color32::LIGHT_GRAY)),
                            item.pen_uuid.clone(),
                        )
                    })
                    .collect(),
            ),
        }
    }
}
