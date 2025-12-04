use crate::core::machine::MachineConfig;
use crate::core::post::GeometryToMultiLineString;
use anyhow::{Result, anyhow};
use aoer_plotty_rs::context::operation::OPLayer;
use aoer_plotty_rs::geo_types::matrix::TransformGeometry;
// use aoer_plotty_rs::geo_types::hatch::Hatches;
pub use super::paper::*;
pub use aoer_plotty_rs::context::pgf_file::*;
pub use aoer_plotty_rs::plotter::pen::PenDetail;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::{
    Coord, Geometry, LineString, MultiLineString, Point, Rect, Rotate, Scale, Translate, coord,
};
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
use usvg::{Tree, WriteOptions};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GeometryKind {
    Stroke(Geometry),
    Hatch(Geometry),
}

impl GeometryKind {
    pub fn bounding_rect(&self) -> Option<Rect> {
        self.geometry().bounding_rect()
    }

    pub fn transformed(&self, tx: &Affine2<f64>) -> GeometryKind {
        let inner_geo = self.geometry().transformed(tx);
        match self {
            GeometryKind::Hatch(_old_geo) => GeometryKind::Hatch(inner_geo),
            GeometryKind::Stroke(_old_geo) => GeometryKind::Stroke(inner_geo),
        }
    }

    pub fn rotate_around_point(&self, degrees: f64, around: impl Into<Point<f64>>) -> GeometryKind {
        let mut inner_geo = self.geometry().clone();
        inner_geo.rotate_around_point_mut(degrees, around.into());
        match self {
            GeometryKind::Hatch(_old_geo) => GeometryKind::Hatch(inner_geo),
            GeometryKind::Stroke(_old_geo) => GeometryKind::Stroke(inner_geo),
        }
    }

    pub fn geometry(&self) -> &Geometry {
        match self {
            GeometryKind::Stroke(geo) => geo,
            GeometryKind::Hatch(geo) => geo,
        }
    }

    pub fn geometry_mut(&mut self) -> &mut Geometry {
        match self {
            GeometryKind::Stroke(geo) => geo,
            GeometryKind::Hatch(geo) => geo,
        }
    }

    pub fn rotate_around_point_mut(&mut self, degrees: f64, around: impl Into<Point<f64>>) {
        self.geometry_mut()
            .rotate_around_point_mut(degrees, around.into());
    }

    pub fn scale_around_point_mut(
        &mut self,
        xscale: f64,
        yscale: f64,
        around: impl Into<Coord<f64>>,
    ) {
        self.geometry_mut()
            .scale_around_point_mut(xscale, yscale, around)
    }

    pub fn translate_mut(&mut self, x: f64, y: f64) {
        match self {
            GeometryKind::Stroke(geometry) => geometry.translate_mut(x, y),
            GeometryKind::Hatch(geometry) => geometry.translate_mut(x, y),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BAPGeometry {
    pub pen_uuid: Uuid,
    pub geometry: GeometryKind,
    pub keepdown_strategy: KeepdownStrategy,
}

impl BAPGeometry {
    pub fn transformed(&self, tx: &Affine2<f64>) -> BAPGeometry {
        BAPGeometry {
            pen_uuid: self.pen_uuid,
            geometry: self.geometry.transformed(tx),
            keepdown_strategy: self.keepdown_strategy,
        }
    }

    #[allow(unused)]
    pub fn rotate_around_point_mut(&mut self, degrees: f64, around: impl Into<Point<f64>>) {
        self.geometry_mut()
            .rotate_around_point_mut(degrees, around.into());
    }

    #[allow(unused)]
    pub fn scale_around_point_mut(&mut self, xs: f64, ys: f64, around: impl Into<Coord<f64>>) {
        self.geometry_mut().scale_around_point_mut(xs, ys, around);
    }

    #[allow(unused)]
    pub fn translate_mut(&mut self, x: f64, y: f64) {
        self.geometry_mut().translate_mut(x, y);
    }

    pub fn lines(&self) -> MultiLineString {
        match &self.geometry {
            GeometryKind::Stroke(geometry) => geometry.to_multi_line_strings(),
            GeometryKind::Hatch(geometry) => geometry.to_multi_line_strings(),
        }
    }
    pub fn geometry(&self) -> &Geometry {
        self.geometry.geometry()
    }

    pub fn geometry_mut(&mut self) -> &mut Geometry {
        self.geometry.geometry_mut()
    }
}

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
    extents: Rect,
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

    pub fn translate_arbitrary_geo(
        geo: &Vec<BAPGeometry>,
        translation: (f64, f64),
        picked: &Option<BTreeSet<u32>>,
    ) -> Vec<BAPGeometry> {
        let mut geo_out = geo.clone();
        for (idx, geometry) in geo_out.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry
                        .geometry
                        .translate_mut(translation.0, translation.1);
                }
            } else {
                geometry
                    .geometry
                    .translate_mut(translation.0, translation.1);
            }
        }
        return geo_out;
    }

    /// Translates all geometry.
    pub fn translate_geometry_mut(
        &mut self,
        translation: (f64, f64),
        picked: &Option<BTreeSet<u32>>,
    ) {
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry
                        .geometry
                        .translate_mut(translation.0, translation.1);
                }
            } else {
                geometry
                    .geometry
                    .translate_mut(translation.0, translation.1);
            }
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
    }

    /// Scale all geometry around a given point.
    pub fn scale_geometry_around_point(
        geo: &Vec<BAPGeometry>,
        center: (f64, f64),
        scale: f64,
        picked: &Option<BTreeSet<u32>>,
    ) -> Vec<BAPGeometry> {
        let mut geo = geo.clone();
        for (idx, plotgeo) in geo.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    plotgeo.geometry.scale_around_point_mut(
                        scale,
                        scale,
                        Point::new(center.0, center.1),
                    );
                }
            } else {
                plotgeo.geometry.scale_around_point_mut(
                    scale,
                    scale,
                    Point::new(center.0, center.1),
                );
            }
        }
        geo
    }

    /// Scale all geometry around a given point.
    pub fn scale_geometry_around_point_mut(
        &mut self,
        center: (f64, f64),
        scale: f64,
        picked: &Option<BTreeSet<u32>>,
    ) {
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry.geometry.scale_around_point_mut(
                        scale,
                        scale,
                        Point::new(center.0, center.1),
                    );
                }
            } else {
                geometry.geometry.scale_around_point_mut(
                    scale,
                    scale,
                    Point::new(center.0, center.1),
                );
            }
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
    }

    /// Rotates all geometry around a given point.
    pub fn rotate_geometry_around_point_mut(
        &mut self,
        center: (f64, f64),
        degrees: f64,
        picked: &Option<BTreeSet<u32>>,
    ) {
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            if let Some(picks) = picked {
                if picks.contains(&(idx as u32)) {
                    geometry
                        .geometry
                        .rotate_around_point_mut(degrees, Point::new(center.0, center.1));
                }
            } else {
                geometry
                    .geometry
                    .rotate_around_point_mut(degrees, Point::new(center.0, center.1));
            }
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
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

            for old_geo in &self.old_geometry {
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

    pub fn extents(&self) -> Rect {
        self.extents.clone()
    }

    pub fn rotate_geometry_around_point(
        &self,
        around: (f64, f64),
        angle: f64,
        picked: &Option<BTreeSet<u32>>,
    ) -> Vec<BAPGeometry> {
        let (xc, yc) = around;
        self.plot_geometry
            .iter()
            .enumerate()
            .map(|(idx, pg)| {
                let new_geo = if let Some(pick) = picked {
                    if pick.contains(&(idx as u32)) {
                        pg.geometry.rotate_around_point(angle, Point::new(xc, yc))
                    } else {
                        pg.geometry.clone()
                    }
                } else {
                    pg.geometry.rotate_around_point(angle, Point::new(xc, yc))
                };
                BAPGeometry {
                    geometry: new_geo,
                    pen_uuid: pg.pen_uuid,
                    keepdown_strategy: pg.keepdown_strategy,
                }
            })
            .collect()
    }

    pub fn calc_extents_for_geometry(geometry: &Vec<BAPGeometry>) -> Rect {
        if geometry.len() == 0 {
            return Rect::new(coord! {x: -1., y: -1.}, coord! {x: 1., y: 1.});
        }
        let mut xmin = f64::MAX;
        let mut xmax = f64::MIN;
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        for geo in geometry {
            // Only update extents if the geometry is rational and non-empty.
            if let Some(tmp_extents) = geo.geometry.bounding_rect() {
                if tmp_extents.min().x < xmin {
                    xmin = tmp_extents.min().x;
                }
                if tmp_extents.min().y < ymin {
                    ymin = tmp_extents.min().y;
                }
                if tmp_extents.max().x > xmax {
                    xmax = tmp_extents.max().x;
                }
                if tmp_extents.max().y > ymax {
                    ymax = tmp_extents.max().y;
                }
            }
        }
        if xmax - xmin == 0. || ymax - ymin == 0. {
            xmin = -1.;
            ymin = -1.;
            xmax = 1.;
            ymax = 1.;
        }
        let extents = Rect::new(coord! {x: xmin, y:ymin}, coord! {x:xmax, y: ymax});
        // println!("Returning calculated extents of {:?}", &extents);
        extents
    }

    /// Returns top left, bottom right as 4 f64s.
    pub fn calc_extents(&self) -> Rect {
        Self::calc_extents_for_geometry(&self.plot_geometry)
    }

    pub fn regenerate_extents(&mut self) {
        self.extents = self.calc_extents();
    }

    #[allow(unused)]
    fn scale_native_units(units: &str) -> f64 {
        if units == "mm" {
            1.
        } else if units == "in" {
            25.4
        } else if units == "pt" {
            25.4 / 72.
        } else if units == "cm" {
            10.
        } else {
            25.4 / 96.
        }
    }

    /// Scales the entire set of geometries by some factor.
    pub fn scale_by_factor(&mut self, factor: f64) {
        let tx_affine2 = Affine2::<f64>::from_matrix_unchecked(Matrix3::new(
            factor, 0., 0., 0., factor, 0., 0., 0., 1.,
        ));
        for geo in self.plot_geometry.iter_mut() {
            *geo = geo.transformed(&tx_affine2);
        }
        self.regenerate_extents();
    }

    fn dims_from_dimattr(attr: &str) -> Option<(f64, &str)> {
        let units_re =
            regex::Regex::new(r"^(?P<val>[0-9]+\.?[0-9]*)\s*(?P<units>[a-zA-Z]*)").unwrap();
        if let Some(captures) = units_re.captures(attr) {
            if let Some(value) = captures.name("val") {
                if let Some(units) = captures.name("units") {
                    if let Ok(value) = value.as_str().parse::<f64>() {
                        return Some((value, units.as_str()));
                    }
                }
            }
        }
        return None;
    }

    fn load_svg(path: &PathBuf) -> Result<(usvg::Tree, f64, f64)> {
        let path = path.clone();
        let mut opt = usvg::Options::default();
        // opt.dpi = 25.4;
        opt.dpi = 96.;
        let path = std::fs::canonicalize(path)?;
        let svg_data = std::fs::read(path)?;
        // let mut scale_x: f64 = 25.4 / 96.;
        // let mut scale_y: f64 = 25.4 / 96.;
        // let mut scale_x = 1.;
        // let mut scale_y = 1.;
        // We parse it twice. Inefficient AF, but an easy way to get the BBox
        if let Ok(xmltree) = usvg::roxmltree::Document::parse(
            String::from_utf8(svg_data.clone())
                .unwrap_or("".to_string())
                .as_str(),
        ) {
            // println!("PARSED! {:?}", xmltree.root());
            let rtree = usvg::Tree::from_xmltree(&xmltree, &opt)?;
            let _rsize = rtree.size().clone();
            // let rsize = rtree.view_box.rect.clone();
            for child in xmltree
                .root()
                .children()
                .filter(|item| item.has_tag_name("svg"))
            {
                if let Some(width) = child.attribute("width") {
                    // println!("WIDTH IS: {:?} and rsize is {:?}", width, rsize);
                    if let Some((_value, _units)) = Self::dims_from_dimattr(width) {
                        // println!("Values, Units: {},{}", &value, &units);
                        // scale_x = (value / rsize.width() as f64) * Self::scale_native_units(units);
                        // println!("ScaleX is now: {}", scale_x);
                    }
                }
                // if let Some(height) = child.attribute("height") {
                //     // if let Some((value, units)) = Self::dims_from_dimattr(height) {
                //     // println!("Values, Units: {},{}", &value, &units);
                //     // scale_y = (value / rsize.height() as f64) * Self::scale_native_units(units);
                //     // println!("ScaleY is now: {}", scale_y);
                //     // }
                // }
                // println!("Child: {:?}", child)
            }
            // Ok((rtree, scale_x, scale_y))
            Ok((rtree, 1., 1.))
        } else {
            Err(anyhow!("No SVG parsed."))
        }
    }

    pub fn update_pen_details(&mut self, pen_crib: &Vec<PenDetail>) {
        self.pens = pen_crib.clone();
        /*
        // println!("Updating pens with {:?}", &pen_crib);
        println!("Got new pens: {:?}", pen_crib);
        let mut orig_geopens_map: HashMap<usize, Uuid> = HashMap::new(); //geometry_id->Pen UUID
        let mut new_pensgeo_map: HashMap<Uuid, usize> = HashMap::new(); //PenUUID->PenIDX
        self.pens = pen_crib.clone();
        for (idx, pen) in self.pens.iter().enumerate() {
            new_pensgeo_map.insert(pen.identity.clone(), idx);
        }

        let default_pen = match pen_crib.get(0) {
            Some(pen_detail) => pen_detail.clone(),
            None => PenDetail::default(),
        };
        for (idx, geometry) in self.plot_geometry.iter_mut().enumerate() {
            let pen_uuid = geometry
                .stroke
                .as_ref()
                .unwrap_or(&default_pen)
                .identity
                .clone();
            orig_geopens_map.insert(idx, pen_uuid);
            let new_pen_idx = new_pensgeo_map.get(&pen_uuid).unwrap_or(&0).clone();
            let new_stroke = Some(self.pens.get(new_pen_idx).unwrap_or(&default_pen).clone());
            geometry.stroke = new_stroke;
        }
        */
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
    /// Loads a pregenerated plot geo set (Plotter Geometry Format)
    pub fn load_pgf(&mut self, path: &PathBuf, import_pens: bool) -> Result<()> {
        self.plot_geometry = vec![];
        if let Ok(path) = std::fs::canonicalize(path) {
            let pgf: PGF = PGF::from_file(&path)?;
            if !import_pens {
                if self.pens.is_empty() {
                    let dpen = PenDetail::default();
                    self.pens.push(dpen.clone());
                }
            }
            for geometry in &mut pgf.geometries().clone() {
                if import_pens {
                    if let Some(stroke) = &mut geometry.stroke {
                        if stroke.identity.is_nil() {
                            stroke.identity = Uuid::new_v4();
                        }
                        if !self.pens.contains(&stroke) {
                            self.pens.push(stroke.clone());
                        }
                    }
                } else {
                    geometry.stroke = Some(self.pens.first().unwrap().clone())
                };
                self.plot_geometry.push(BAPGeometry {
                    geometry: GeometryKind::Stroke(geometry.geometry.clone()),
                    pen_uuid: match &geometry.stroke {
                        Some(pen) => pen.identity,
                        None => Uuid::new_v4(),
                    },
                    keepdown_strategy: geometry.keepdown_strategy,
                });
            }
            self.regenerate_extents();
        }
        Ok(())
    }

    pub fn import_svg(&mut self, path: &PathBuf, keepdown: bool, generate_pens: bool) {
        if let Ok(path) = std::fs::canonicalize(path) {
            if let Ok((rtree, scale_x, scale_y)) = Self::load_svg(&path) {
                let svg_string = rtree.to_string(&usvg::WriteOptions::default());
                // println!("{:?}", rtree.root());
                self.svg = Some(svg_string);
                let tmp_geometry =
                    svg_to_geometries(&rtree, scale_x, scale_y, keepdown, &mut self.pens);
                if !generate_pens {
                    if self.pens.is_empty() {
                        let dpen = PenDetail::default();
                        self.pens.push(dpen.clone());
                    }
                }
                self.plot_geometry = tmp_geometry
                    .iter()
                    .map(|geo| {
                        let mut tmp_id = geo.stroke.clone().unwrap().identity;
                        if tmp_id.is_nil() {
                            tmp_id = Uuid::new_v4();
                        };
                        if !generate_pens {
                            tmp_id = self.pens.get(0).unwrap().identity;
                        }
                        BAPGeometry {
                            pen_uuid: tmp_id,
                            geometry: GeometryKind::Stroke(geo.geometry.clone()),
                            keepdown_strategy: geo.keepdown_strategy,
                        }
                    })
                    .collect();
                // self.extents = self.calc_extents();
                self.regenerate_extents();
            }
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

pub fn svg_to_geometries(
    tree: &Tree,
    scale_x: f64,
    scale_y: f64,
    keepdown: bool,
    pens: &mut Vec<PenDetail>,
) -> Vec<PlotGeometry> {
    // println!("I received pens: {:?}", pens);
    let mut geometries: Vec<PlotGeometry> = vec![];
    // let mut multilines: MultiLineString<f64> = MultiLineString::new(vec![]);

    // TODO: We should look at parsing WITH preprocessing if it makes things more reliable
    if let Ok(out) =
        svg2polylines::parse_with_meta(&*tree.to_string(&WriteOptions::default()), 0.1, false)
    {
        // let mut pens: Vec<PenDetail> = vec![];
        for (idx, (linestring, meta)) in out.iter().enumerate() {
            let tmp_ls = LineString::new(
                linestring
                    .iter()
                    .map(|x| coord! {x: scale_x * x.x as f64, y: scale_y * x.y as f64})
                    .collect(),
            );
            // println!("Got geometry of: {:?}", tmp_ls);
            let this_pen = PenDetail {
                tool_id: 1 + pens.len(),
                name: format!("svg-auto-{}", idx + 1),
                stroke_width: 1.,
                stroke_density: 1.,
                feed_rate: None,
                color: match &meta.stroke {
                    Some(stroke_string) => csscolorparser::parse(stroke_string.as_str())
                        .unwrap_or(csscolorparser::parse("black").unwrap()),
                    None => csscolorparser::parse("black").unwrap(),
                },
                ..Default::default()
            };
            // println!("This pen: {:?}", this_pen);
            let mut pen_idx = usize::MAX;
            for (idx, pen) in pens.iter().enumerate() {
                if (pen.color == this_pen.color)
                    && (pen.stroke_width == this_pen.stroke_width)
                    && (pen.stroke_density == this_pen.stroke_density)
                {
                    pen_idx = idx
                }
            }
            let pen_out = if pen_idx != usize::MAX {
                pens[pen_idx].clone()
            } else {
                pens.push(this_pen.clone());
                this_pen
            };

            geometries.push(PlotGeometry {
                geometry: Geometry::MultiLineString(MultiLineString::new(vec![tmp_ls])),
                stroke: Some(pen_out),
                keepdown_strategy: if keepdown {
                    KeepdownStrategy::PenWidthAuto
                } else {
                    KeepdownStrategy::None
                },
                // meta: HashMap::new(),
            });
        }
    }
    // println!("Total geo size is: {:?}", multilines.bounding_rect());

    // println!("After import, pens are: {:?}", pens);
    geometries
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
        if let Ok((_rtree, scale_x, scale_y)) =
            Project::load_svg(&PathBuf::from_str("resources/plotter_sign_better.svg").unwrap())
        {
            println!("SX,SY: {},{}", scale_x, scale_y);
        } else {
            assert!(false)
        }
    }
}
