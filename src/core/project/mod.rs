use crate::core::machine::MachineConfig;
use anyhow::{Result, anyhow};
use aoer_plotty_rs::context::operation::OPLayer;
// use aoer_plotty_rs::geo_types::hatch::Hatches;
pub use super::paper::*;
pub use aoer_plotty_rs::context::pgf_file::*;
pub use aoer_plotty_rs::plotter::pen::PenDetail;
use geo::{Point, Rect, coord};
use nalgebra::{Affine2, Matrix3};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

pub(crate) mod bap_geometry;
pub(crate) mod extents;
pub(crate) mod geometry_kind;
pub(crate) mod import;
// pub(crate) mod project;
pub(crate) mod reorder;
pub(crate) mod transforms;
pub use bap_geometry::BAPGeometry;
pub use geometry_kind::GeometryKind;
// pub use project::*;

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectV0 {
    svg: Option<String>,
    pub geometry: Vec<PlotGeometry>,
    // pub geometry: HashMap<Uuid, PlotGeometry>,
    pub layers: HashMap<String, OPLayer>,
    pub pens: Vec<PenDetail>,
    pub paper: Paper,
    pub origin: Option<(f64, f64)>, // Target/center of the viewport
    extents: Rect,
    machine: Option<MachineConfig>,
    program: Option<Box<Vec<String>>>,
    pub do_keepdown: bool,
    pub file_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    #[serde(default)]
    version: usize,
    svg: Option<String>,
    #[serde(default)]
    pub plot_geometry: Vec<BAPGeometry>,
    #[serde(skip_serializing)]
    #[serde(rename = "geometry")]
    #[serde(default)]
    pub old_geometry: Vec<PlotGeometry>,
    pub pens: Vec<PenDetail>,
    pub paper: Paper,
    pub origin: Option<(f64, f64)>, // Target/center of the viewport
    pub extents: Rect,
    machine: Option<MachineConfig>,
    program: Option<Box<Vec<String>>>,
    pub do_keepdown: bool,
    pub file_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectGuessVersion {
    #[serde(default)]
    version: usize,
}

impl Project {
    pub fn new() -> Self {
        Project {
            version: 2,
            svg: None,
            plot_geometry: vec![],
            old_geometry: vec![],
            pens: Vec::new(),
            paper: Paper {
                weight_gsm: 180.,
                rgb: (1.0, 1.0, 1.0),
                size: PaperSize::Letter,
                orientation: Orientation::Portrait,
            },
            extents: Rect::new(coord! {x: -1., y: -1.}, coord! {x: 1., y: 1.}),
            origin: None,
            machine: Some(MachineConfig::default()),
            program: None,
            do_keepdown: true,
            file_path: None,
        }
    }

    pub fn pen_by_uuid(&self, uuid: Uuid) -> Option<PenDetail> {
        for pen in &self.pens {
            if pen.identity == uuid {
                return Some(pen.clone());
            }
        }
        None
    }

    pub fn save(&self) -> Result<PathBuf> {
        match &self.file_path {
            Some(path) => Ok(self.save_to_path(path)?),
            None => Err(anyhow!("No path set, couldn't save to existing path.")),
        }
    }

    /// Saves to a destination path. Makes a new temp file and moves
    /// it to the destination after writing it.
    pub fn save_to_path(&self, path: &PathBuf) -> Result<PathBuf> {
        let mut path = path.clone(); //std::fs::canonicalize(path)?;
        let mut dest_path = path.clone();
        dest_path.set_extension(OsString::from_str("bap2")?);
        // We save, then move, to ensure we don't accidentally delete if something bad happens.
        let tmptime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        path.set_extension(OsString::from_str(format!("bap.{tmptime}.tmp").as_str())?);

        // Write the file
        {
            let writer = std::fs::File::create(&path)?;
            let writer = Box::new(BufWriter::new(writer));
            //
            // Turns out writer_pretty is hella slow too.
            // Or... Actually it was the lack of a bufwriter. Wups.
            ron::Options::default().to_io_writer_pretty(writer, self, PrettyConfig::default())?;
        } // Falls out of scope, closes file, we hope.
        std::fs::rename(&path, &dest_path)?;
        // println!("Renamed.");
        Ok(dest_path)
    }

    pub fn guess_file_version(path: &PathBuf) -> Result<usize> {
        let project_rdr = std::fs::File::open(path.clone())?;
        let gv = ron::de::from_reader::<File, ProjectGuessVersion>(project_rdr)?;
        Ok(gv.version)
    }

    pub fn find_matching_pen(&self, pen: &PenDetail) -> Option<PenDetail> {
        for my_pen in &self.pens {
            if my_pen.color == pen.color
                && my_pen.stroke_width == pen.stroke_width
                && my_pen.stroke_density == pen.stroke_density
            {
                return Some(my_pen.clone());
            }
        }
        None
    }

    pub fn merge_matching_pens(&mut self) {
        let mut remove_pens: Vec<(Uuid, Uuid)> = vec![]; //old_uuid, one_we_keep
        for geo in &self.plot_geometry {
            let old_uuid = geo.pen_uuid;
            if let Some(geopen) = self.pen_by_uuid(old_uuid) {
                if let Some(matchpen) = self.find_matching_pen(&geopen) {
                    // We have a matching pen.
                    if matchpen.identity == old_uuid {
                        continue;
                    } else {
                        remove_pens.push((old_uuid, matchpen.identity));
                    }
                }
            }
        }

        // println!("Would remove 3ens: {:#?}", remove_pens);
        for (remove, replace) in remove_pens.clone() {
            for geo in &mut self.plot_geometry {
                if geo.pen_uuid == remove {
                    geo.pen_uuid = replace;
                }
            }
            for (idx, pen) in self.pens.clone().iter().enumerate() {
                if pen.identity == remove {
                    self.pens.remove(idx);
                }
            }
        }

        // Finally, cull unused pens.
        let mut used_pens: BTreeSet<Uuid> = BTreeSet::new();
        for geo in &self.plot_geometry {
            used_pens.insert(geo.pen_uuid);
        }
        for (idx, pen) in self.pens.clone().iter().enumerate().rev() {
            if !used_pens.contains(&pen.identity) {
                // println!("Removing unused pen {}", pen.identity);
                self.pens.remove(idx);
            }
        }
    }

    pub fn upgrade(&mut self) {
        if self.version == 0 {
            self.version = 2;
            for pen in &mut self.pens {
                if pen.identity == Uuid::nil() {
                    pen.identity = Uuid::new_v4();
                }
            }

            for (idx, old_geo) in self.old_geometry.clone().iter().enumerate() {
                // println!("Found pen in geo: {:#?}", old_geo.stroke);
                let mut found_pen = if let Some(stroke) = old_geo.stroke.clone() {
                    if let Some(found_pen) = self.find_matching_pen(&stroke) {
                        found_pen
                    } else {
                        let mut tmp_pen = old_geo
                            .stroke
                            .clone()
                            .unwrap_or(self.pens.first().unwrap_or(&PenDetail::default()).clone());
                        if tmp_pen.identity.is_nil() {
                            tmp_pen.identity = Uuid::new_v4();
                        }
                        self.pens.push(tmp_pen.clone());
                        tmp_pen
                    }
                } else {
                    self.pens.first().unwrap_or(&PenDetail::default()).clone()
                };
                if found_pen.identity.is_nil() {
                    found_pen.identity = Uuid::new_v4();
                }
                self.plot_geometry.push({
                    BAPGeometry {
                        pen_uuid: found_pen.identity,
                        name: format!("geometry {}", idx).to_string(),
                        geometry: GeometryKind::Stroke(old_geo.geometry.clone()),
                        keepdown_strategy: old_geo.keepdown_strategy,
                    }
                })
            }
        }
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if let Ok(path) = std::fs::canonicalize(path) {
            let project_rdr = std::fs::File::open(path.clone())?;
            // if let Ok(prj) = ron::de::from_reader(project_rdr) {
            //     return Ok(prj);
            // }
            return match ron::de::from_reader::<File, Self>(project_rdr) {
                Ok(mut prj) => {
                    prj.file_path = Some(path);
                    // prj.reindex_geometry();
                    prj.calc_extents();
                    // println!("Calced extents are: {:?}", prj.extents());
                    // println!("Loaded pens from disk: {:?}", &prj.pens);
                    prj.upgrade();

                    Ok(prj)
                }
                Err(err) => {
                    // eprintln!("Failed to load due to: {:?}", &err);
                    let version = Self::guess_file_version(&path)?;
                    eprintln!("Guess file version: {}", version);

                    Err(anyhow!(format!("Error was: {:?}", &err)))
                }
            };
        };
        Err(anyhow!(format!("Invalid project path {:?}", path)))
    }

    pub fn set_program(&mut self, program: Option<Box<Vec<String>>>) {
        self.program = program
    }

    pub fn program(&self) -> Option<Box<Vec<String>>> {
        self.program.clone()
    }

    pub fn machine(&self) -> Option<MachineConfig> {
        self.machine.clone()
    }

    #[allow(unused)]
    pub fn set_machine(&mut self, machine: Option<MachineConfig>) {
        self.machine = machine.clone()
    }

    #[allow(unused)]
    pub fn valid_project(&self) -> bool {
        !(self.svg == None)
    }

    pub fn origin(&self) -> Option<(f64, f64)> {
        self.origin.clone()
    }

    pub fn set_origin(&mut self, origin: &Option<(f64, f64)>) {
        // println!("Setting origin.");
        self.origin = origin.clone();
    }

    #[allow(unused)]
    pub fn svg(&self) -> Option<String> {
        self.svg.clone()
    }

    pub fn update_pen_details(&mut self, pen_crib: &Vec<PenDetail>) {
        self.pens = pen_crib.clone();
        for idx in 0..self.plot_geometry.len() {
            if self.pen_by_uuid(self.plot_geometry[idx].pen_uuid).is_none() {
                if let Some(pen) = self.pens.first() {
                    self.plot_geometry[idx].pen_uuid = pen.identity;
                } else {
                    let new_pen = PenDetail {
                        identity: Uuid::new_v4(),
                        ..PenDetail::default()
                    };
                    self.pens.push(new_pen.clone());
                    self.plot_geometry[idx].pen_uuid = new_pen.identity;
                }
            }
        }
    }

    /// Loads a machine config from disk
    pub fn load_machine(&mut self, path: &PathBuf) -> Result<()> {
        if let Ok(path) = std::fs::canonicalize(path) {
            let machine: MachineConfig = MachineConfig::load_from_path(&path)?;
            self.machine = Some(machine);
        }
        Ok(())
    }

    /// Saves a machine config to disk
    pub fn save_machine(&mut self, path: &PathBuf) -> Result<()> {
        if let Some(machine) = &self.machine {
            machine.save_to_path(&path)?;
        }
        Ok(())
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Paper {
    #[allow(unused)]
    pub fn dimensions(&self) -> (f64, f64) {
        let dims = self.size.dimensions();
        match self.orientation {
            Orientation::Portrait => dims,
            Orientation::Landscape => (dims.1, dims.0),
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    pub fn test_import_svg_scale_default() {
        let mut project = Project::default();
        project.import_svg(
            &PathBuf::from_str("resources/plotter_sign_better.svg").unwrap(),
            true,
            true,
        );
    }

    #[test]
    pub fn test_import_svg_scale_mm() {
        let mut project = Project::default();
        project.import_svg(
            &PathBuf::from_str("resources/test_groups_simple.svg").unwrap(),
            true,
            true,
        );
    }

    #[test]
    pub fn test_load_svg_scale_mm() {
        if let Ok((_rtree, scale_x, scale_y)) =
            Project::load_svg(&PathBuf::from_str("resources/test_groups_simple.svg").unwrap())
        {
            println!("SX,SY: {},{}", scale_x, scale_y);
        } else {
            assert!(false)
        }
    }

    #[test]
    pub fn test_load_svg_scale_default() {
        if let Ok((_rtree, scale_x, scale_y)) = super::super::Project::load_svg(
            &PathBuf::from_str("resources/plotter_sign_better.svg").unwrap(),
        ) {
            println!("SX,SY: {},{}", scale_x, scale_y);
        } else {
            assert!(false)
        }
    }
}
