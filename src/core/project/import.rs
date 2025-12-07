use anyhow::{Result, anyhow};
pub use aoer_plotty_rs::context::pgf_file::*;
pub use aoer_plotty_rs::plotter::pen::PenDetail;
use geo::{Geometry, LineString, MultiLineString, coord};
use std::path::PathBuf;
use usvg::{Tree, WriteOptions};
use uuid::Uuid;

use crate::core::project::{BAPGeometry, GeometryKind};

pub fn svg_to_geometries(
    tree: &Tree,
    scale_x: f64,
    scale_y: f64,
    keepdown: bool,
    pens: &mut Vec<PenDetail>,
) -> Vec<PlotGeometry> {
    let mut geometries: Vec<PlotGeometry> = vec![];
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
    geometries
}

impl super::Project {
    pub fn load_svg(path: &PathBuf) -> Result<(usvg::Tree, f64, f64)> {
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
            for (idx, geometry) in &mut pgf.geometries().clone().iter_mut().enumerate() {
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
                    name: format!("geometry {}", idx).to_string(),
                    geometry: GeometryKind::Stroke(geometry.geometry.clone()),
                    pen_uuid: match &geometry.stroke {
                        Some(pen) => pen.identity,
                        None => Uuid::new_v4(),
                    },
                    keepdown_strategy: geometry.keepdown_strategy,
                });
            }
            if import_pens {
                // for idx in 0..self.pens.len() {
                //     self.pens[idx].tool_id = idx + 1;
                // }
                self.merge_matching_pens();
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
                    .enumerate()
                    .map(|(idx, geo)| {
                        let mut tmp_id = geo.stroke.clone().unwrap().identity;
                        if tmp_id.is_nil() {
                            tmp_id = Uuid::new_v4();
                        };
                        if !generate_pens {
                            tmp_id = self.pens.get(0).unwrap().identity;
                        }
                        BAPGeometry {
                            name: format!("geometry {}", idx).to_string(),
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
