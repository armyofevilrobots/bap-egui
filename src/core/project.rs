use crate::core::machine::MachineConfig;
use anyhow::{Result, anyhow};
use aoer_plotty_rs::context::operation::OPLayer;
use gcode::GCode;
// use aoer_plotty_rs::geo_types::hatch::Hatches;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::prelude::MapCoords;
use geo::{Coord, Geometry, LineString, MultiLineString, Point, Rect, Rotate, coord};
use nalgebra::{Affine2, Matrix3, Point2};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fmt::Display;
use std::io::BufWriter;
use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};
use usvg::{Tree, WriteOptions};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum KeepdownStrategy {
    None,
    #[default]
    PenWidthAuto,
    PenWidthMultiple(f64),
    Static(f64),
}

impl KeepdownStrategy {
    pub fn threshold(&self, penwidth: f64) -> f64 {
        match self {
            KeepdownStrategy::None => 0.1,
            KeepdownStrategy::PenWidthAuto => 1.414f64 * penwidth,
            KeepdownStrategy::PenWidthMultiple(mul) => mul * penwidth,
            KeepdownStrategy::Static(val) => *val,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PenDetail {
    #[serde(default)]
    pub tool_id: usize,
    #[serde(default)]
    pub name: String,
    pub stroke_width: f64,
    pub stroke_density: f64,
    pub feed_rate: Option<f64>, //mm/min
    pub color: String,
}

impl Default for PenDetail {
    fn default() -> Self {
        Self {
            tool_id: 1,
            name: "Default Pen".to_string(),
            stroke_width: 0.5,
            stroke_density: 0.5,
            feed_rate: None,
            color: "#000000".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HatchDetail {
    pub hatch_pattern: String, //Arc<Box<dyn HatchPattern>>,
    pub pen: Option<PenDetail>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Orientation {
    Landscape,
    Portrait,
}
impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::Landscape => write!(f, "Landscape"),
            Orientation::Portrait => write!(f, "Portrait"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PaperSize {
    Letter,
    Ansi_A,
    Ansi_B,
    Ansi_C,
    Ansi_D,
    US_Legal,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    ISO216,
    USBusinessCard,
    EuroBusinessCard,
    Custom(f64, f64),
}

impl PaperSize {
    /// Get the mm measurements for this paper size, as a tuple of f64s.
    pub fn dims(&self) -> (f64, f64) {
        match self {
            PaperSize::Letter => (216., 279.),
            PaperSize::Ansi_A => (216., 279.),
            PaperSize::Ansi_B => (279., 432.),
            PaperSize::Ansi_C => (432., 559.),
            PaperSize::Ansi_D => (559., 864.),
            PaperSize::US_Legal => (216., 356.),
            PaperSize::A0 => (841., 1189.),
            PaperSize::A1 => (594., 841.),
            PaperSize::A2 => (420., 594.),
            PaperSize::A3 => (297., 420.),
            PaperSize::A4 => (210., 297.),
            PaperSize::A5 => (148., 210.),
            PaperSize::A6 => (105., 148.),
            PaperSize::A7 => (74., 105.),
            PaperSize::ISO216 => (74., 52.),
            PaperSize::USBusinessCard => (88.9, 50.8),
            PaperSize::EuroBusinessCard => (85., 55.),
            PaperSize::Custom(x, y) => (*x, *y),
        }
    }

    pub fn all() -> Vec<PaperSize> {
        vec![
            PaperSize::Letter,
            PaperSize::Ansi_A,
            PaperSize::Ansi_B,
            PaperSize::Ansi_C,
            PaperSize::Ansi_D,
            PaperSize::US_Legal,
            PaperSize::A0,
            PaperSize::A1,
            PaperSize::A2,
            PaperSize::A3,
            PaperSize::A4,
            PaperSize::A5,
            PaperSize::A6,
            PaperSize::A7,
            PaperSize::ISO216,
            PaperSize::USBusinessCard,
            PaperSize::EuroBusinessCard,
            PaperSize::Custom(200., 200.),
        ]
    }
}

impl Display for PaperSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmtarg = match self {
            Self::Letter => "US Letter".to_string(),
            Self::Ansi_A => "Ansi A".to_string(),
            Self::Ansi_B => "Ansi B".to_string(),
            Self::Ansi_C => "Ansi C".to_string(),
            Self::Ansi_D => "Ansi D".to_string(),
            Self::A0 => "A0".to_string(),
            Self::A1 => "A1".to_string(),
            Self::A2 => "A2".to_string(),
            Self::A3 => "A3".to_string(),
            Self::A4 => "A4".to_string(),
            Self::A5 => "A5".to_string(),
            Self::A6 => "A6".to_string(),
            Self::A7 => "A7".to_string(),
            Self::Custom(a, b) => format!("Custom({:.2}x{:.2})", a, b),
            Self::US_Legal => "US Legal".to_string(),
            PaperSize::ISO216 => "ISO216".to_string(),
            PaperSize::USBusinessCard => "US Business Card".to_string(),
            PaperSize::EuroBusinessCard => "Euro Business Card".to_string(),
        };
        write!(f, "{}", fmtarg)
    }
}

impl PaperSize {
    pub fn dimensions(&self) -> (f64, f64) {
        match self {
            PaperSize::Letter => (216., 279.),
            PaperSize::Ansi_A => (216., 279.),
            PaperSize::Ansi_B => (279., 432.),
            PaperSize::Ansi_C => (432., 559.),
            PaperSize::Ansi_D => (559., 864.),
            PaperSize::US_Legal => (216., 356.),
            PaperSize::A0 => (841., 1189.),
            PaperSize::A1 => (594., 841.),
            PaperSize::A2 => (420., 594.),
            PaperSize::A3 => (297., 420.),
            PaperSize::A4 => (210., 297.),
            PaperSize::A5 => (148., 210.),
            PaperSize::A6 => (105., 148.),
            PaperSize::A7 => (74., 105.),
            PaperSize::ISO216 => (74., 52.),
            PaperSize::USBusinessCard => (88.9, 50.8),
            PaperSize::EuroBusinessCard => (85., 55.),
            PaperSize::Custom(x, y) => (*x, *y),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Paper {
    pub weight_gsm: f64,
    pub rgb: (f64, f64, f64), // For display purposes only.
    pub size: PaperSize,
    pub orientation: Orientation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlotGeometry {
    pub geometry: Geometry,
    pub hatch: Option<HatchDetail>,
    pub stroke: Option<PenDetail>,
    #[serde(default)]
    pub keepdown_strategy: KeepdownStrategy,
    pub meta: HashMap<String, String>,
}

impl Default for PlotGeometry {
    fn default() -> Self {
        Self {
            geometry: Geometry::GeometryCollection(geo::GeometryCollection::new_from(vec![])),
            hatch: Default::default(),
            stroke: Default::default(),
            keepdown_strategy: Default::default(),
            meta: Default::default(),
        }
    }
}

impl PlotGeometry {
    pub fn transformed(&self, transformation: &Affine2<f64>) -> PlotGeometry {
        let mut new_geo = self.clone();
        new_geo.geometry = new_geo.geometry.map_coords(|coord| {
            let out = transformation * Point2::new(coord.x, coord.y);
            Coord { x: out.x, y: out.y }
        });
        new_geo
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    svg: Option<String>,
    pub geometry: Vec<PlotGeometry>,
    pub layers: HashMap<String, OPLayer>,
    pub pens: Vec<PenDetail>,
    pub paper: Paper,
    pub dirty: bool,
    pub origin: Option<(f64, f64)>, // Target/center of the viewport
    pub zoom: f64,                  // zoom level
    extents: Rect,
    machine: Option<MachineConfig>,
    program: Option<Box<Vec<String>>>,
    pub do_keepdown: bool,
}

impl Paper {
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
            svg: None,
            geometry: vec![],
            layers: HashMap::new(),
            pens: Vec::new(),
            paper: Paper {
                weight_gsm: 180.,
                rgb: (1.0, 1.0, 1.0),
                size: PaperSize::Letter,
                orientation: Orientation::Portrait,
            },
            extents: Rect::new(coord! {x: -1., y: -1.}, coord! {x: 1., y: 1.}),
            dirty: false,
            origin: None,
            machine: Some(MachineConfig::default()),
            zoom: 1.,
            program: None,
            do_keepdown: true,
        }
    }

    /// Rotates all geometry around a given point.
    pub fn rotate_geometry_around_point(&mut self, center: (f64, f64), degrees: f64) {
        for geometry in &mut self.geometry {
            geometry
                .geometry
                .rotate_around_point_mut(degrees, Point::new(center.0, center.1));
        }
        // println!("ROTATED. Now redoing extents etc.");
        self.regenerate_extents();
    }

    /// Saves to a destination path. Makes a new temp file and moves
    /// it to the destination after writing it.
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // println!("Saving...");
        let mut path = path.clone(); //std::fs::canonicalize(path)?;
        // println!("Canonicalized.");
        let mut dest_path = path.clone();
        dest_path.set_extension(OsString::from_str("bap")?);
        path.set_extension(OsString::from_str("bap.tmp")?);
        // println!("Saving tmp to {:?} and final to {:?}", &path, &dest_path);

        // Write the file
        {
            let writer = std::fs::File::create(&path)?;
            let writer = Box::new(BufWriter::new(writer));
            // println!("Writer...");
            ron::ser::to_writer_pretty(writer, self, PrettyConfig::default())?;
            // println!("Written...");
        } // Falls out of scope, closes file, we hope.
        // writer.sync_all()?;
        // println!("Synced...");
        std::fs::rename(&path, &dest_path)?;
        // println!("Renamed.");
        Ok(())
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if let Ok(path) = std::fs::canonicalize(path) {
            let project_rdr = std::fs::File::open(path.clone())?;
            // if let Ok(prj) = ron::de::from_reader(project_rdr) {
            //     return Ok(prj);
            // }
            return match ron::de::from_reader(project_rdr) {
                Ok(prj) => Ok(prj),
                Err(err) => {
                    eprintln!("Failed to load due to: {:?}", &err);
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

    pub fn set_machine(&mut self, machine: Option<MachineConfig>) {
        self.machine = machine.clone()
    }

    pub fn valid_project(&self) -> bool {
        !(self.svg == None)
    }

    pub fn origin(&self) -> Option<(f64, f64)> {
        self.origin.clone()
    }

    pub fn set_origin(&mut self, origin: &Option<(f64, f64)>) {
        println!("Setting origin.");
        self.origin = origin.clone();
    }

    pub fn svg(&self) -> Option<String> {
        self.svg.clone()
    }

    pub fn extents(&self) -> Rect {
        self.extents.clone()
    }

    /// Returns top left, bottom right as 4 f64s.
    pub fn calc_extents(&self) -> Rect {
        if self.geometry.len() == 0 {
            return Rect::new(coord! {x: -1., y: -1.}, coord! {x: 1., y: 1.});
        }
        let mut xmin = f64::MAX;
        let mut xmax = f64::MIN;
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        for geo in &self.geometry {
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

    pub fn regenerate_extents(&mut self) {
        self.extents = self.calc_extents();
    }

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
        for geo in self.geometry.iter_mut() {
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
            let rsize = rtree.size().clone();
            // let rsize = rtree.view_box.rect.clone();
            for child in xmltree
                .root()
                .children()
                .filter(|item| item.has_tag_name("svg"))
            {
                if let Some(width) = child.attribute("width") {
                    // println!("WIDTH IS: {:?} and rsize is {:?}", width, rsize);
                    if let Some((value, units)) = Self::dims_from_dimattr(width) {
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
        // println!("Updating pens with {:?}", &pen_crib);
        self.pens = pen_crib.clone();
        let default_pen = match pen_crib.get(0) {
            Some(pen_detail) => pen_detail.clone(),
            None => PenDetail::default(),
        };
        for (idx, geometry) in self.geometry.iter_mut().enumerate() {
            let new_stroke_pen = if geometry.stroke.is_some() {
                if let Some(current_stroke_pen) = geometry.stroke.clone() {
                    match pen_crib.get(current_stroke_pen.tool_id - 1) {
                        // Pens IDs are counted from 1, not zero
                        Some(pen) => pen.clone(),
                        None => default_pen.clone(),
                    }
                } else {
                    default_pen.clone()
                }
            } else {
                default_pen.clone()
            };
            // println!(
            //     "Replacing stroke pen {:?} with pen {:?}",
            //     geometry.stroke, new_stroke_pen
            // );
            geometry.stroke = Some(new_stroke_pen);

            if geometry.hatch.is_some() {
                let new_hatch_pen = if let Some(hatch_detail) = geometry.hatch.clone() {
                    if let Some(current_hatch_pen) = hatch_detail.pen {
                        match pen_crib.get(current_hatch_pen.tool_id - 1) {
                            Some(pen) => pen.clone(),
                            None => default_pen.clone(),
                        }
                    } else {
                        default_pen.clone()
                    }
                } else {
                    default_pen.clone()
                };
                // println!(
                //     "Updating hatch pen {:?} with pen {:?}",
                //     geometry.hatch, new_hatch_pen
                // );
                geometry.hatch.as_mut().unwrap().pen = Some(new_hatch_pen);
            };
        }
    }

    pub fn import_svg(&mut self, path: &PathBuf, keepdown: bool) {
        // Paper should already be set?
        // self.paper.rgb = (1., 1., 1.);
        if let Ok(path) = std::fs::canonicalize(path) {
            if let Ok((rtree, scale_x, scale_y)) = Self::load_svg(&path) {
                let svg_string = rtree.to_string(&usvg::WriteOptions::default());
                // println!("{:?}", rtree.root());
                self.svg = Some(svg_string);
                self.geometry = svg_to_geometries(&rtree, scale_x, scale_y, keepdown, &self.pens);
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
    pens: &Vec<PenDetail>,
) -> Vec<PlotGeometry> {
    // println!("I received pens: {:?}", pens);
    let mut geometries: Vec<PlotGeometry> = vec![];
    let mut multilines: MultiLineString<f64> = MultiLineString::new(vec![]);
    // TODO: We should look at parsing WITH preprocessing if it makes things more reliable
    if let Ok(out) = svg2polylines::parse(&*tree.to_string(&WriteOptions::default()), 0.1, false) {
        for linestring in out {
            let tmp_ls = LineString::new(
                linestring
                    .iter()
                    .map(|x| coord! {x: scale_x * x.x as f64, y: scale_y * x.y as f64})
                    .collect(),
            );
            // println!("Parsed geo size is: {:?}", tmp_ls.bounding_rect());
            multilines.0.push(tmp_ls);
        }
    }
    // println!("Total geo size is: {:?}", multilines.bounding_rect());
    geometries.push(PlotGeometry {
        geometry: Geometry::MultiLineString(multilines),
        hatch: Some(HatchDetail {
            hatch_pattern: "".to_string(), //Hatches::none(),
            // TODO: This should in the future be copied from pen settings.
            pen: Some(
                pens.first()
                    .unwrap_or(&PenDetail {
                        stroke_width: 0.5,
                        stroke_density: 1.0,
                        feed_rate: None,
                        color: "black".to_string(),
                        tool_id: 1,
                        name: "PEN1".to_string(),
                    })
                    .clone(),
            ),
        }),
        stroke: Some(
            pens.first()
                .unwrap_or(&PenDetail {
                    stroke_width: 0.5,
                    stroke_density: 1.0,
                    feed_rate: None,
                    color: "black".to_string(),
                    tool_id: 1,
                    name: "PEN1".to_string(),
                })
                .clone(),
        ),
        keepdown_strategy: if keepdown {
            KeepdownStrategy::PenWidthAuto
        } else {
            KeepdownStrategy::None
        },
        meta: HashMap::new(),
    });

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
        );
    }

    #[test]
    pub fn test_import_svg_scale_mm() {
        let mut project = Project::default();
        project.import_svg(
            &PathBuf::from_str("resources/test_groups_simple.svg").unwrap(),
            true,
        );
    }

    #[test]
    pub fn test_load_svg_scale_mm() {
        if let Ok((rtree, scale_x, scale_y)) =
            Project::load_svg(&PathBuf::from_str("resources/test_groups_simple.svg").unwrap())
        {
            println!("SX,SY: {},{}", scale_x, scale_y);
        } else {
            assert!(false)
        }
    }

    #[test]
    pub fn test_load_svg_scale_default() {
        if let Ok((rtree, scale_x, scale_y)) =
            Project::load_svg(&PathBuf::from_str("resources/plotter_sign_better.svg").unwrap())
        {
            println!("SX,SY: {},{}", scale_x, scale_y);
        } else {
            assert!(false)
        }
    }
}
