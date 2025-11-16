use crate::core::project::PenDetail;

use super::project::Project;
use anyhow::Result as AnyResult;
use anyhow::anyhow;
use aoer_plotty_rs::optimizer::*;
use geo::Coord;
#[allow(deprecated)]
use geo::EuclideanDistance;
use geo::{Geometry, LineString, MultiLineString};
use nalgebra::{Affine2, Matrix3};
use tera::Context;

#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(unused)]
pub enum LastMove {
    Move,
    Feed,
    None,
}

pub fn post(project: &Project) -> AnyResult<Vec<String>> {
    let machine = project.machine().ok_or(anyhow!("Invalid machine"))?;
    let post_template = &machine.post_template()?;
    let mut pen_up = false;
    let mut distance_down = 0.0f64; // Used to ensure we do an extra pen down periodically?
    // #[allow(unused)]
    // let mut last_move = LastMove::None;

    let mut program: Vec<String> = Vec::new();
    program.extend(
        post_template
            .render("prelude", &Context::new())?
            .split("\n")
            .map(|s| s.to_string()),
    );
    if let Some(height) = machine.skim() {
        let mut context = Context::new();
        context.insert("skim", &height);
        program.extend(
            post_template
                .render("penup_skim", &context)?
                .split("\n")
                .map(|s| s.to_string()),
        );
        pen_up = true;
    }
    let (mut last_x, mut last_y) = (-9999999., -99999999.);

    let scalex = 1.;
    let scaley = -1.;

    let (tx, ty) = if let Some((ox, oy)) = &project.origin {
        if let Some(_machine) = &project.machine() {
            (-ox, oy)
        } else {
            return Err(anyhow!("Project machine limits are not configured"));
        }
    } else {
        return Err(anyhow!("Project extents are not configured"));
    };

    let tx_affine2 = Affine2::<f64>::from_matrix_unchecked(Matrix3::new(
        scalex.clone(),
        0.,
        tx.clone(),
        0.,
        scaley.clone(),
        ty.clone(),
        0.,
        0.,
        1.,
    ));

    let mut last_tool: usize = 0;
    for geometry in &project.geometry {
        let geo_lines = geometry
            .transformed(&tx_affine2)
            .geometry
            .to_multi_line_strings();
        let opt = Optimizer::new(
            machine.keepdown().unwrap_or(1.0),
            OptimizationStrategy::Greedy,
        );
        let pen = geometry.stroke.clone().unwrap_or(PenDetail::default());
        let feedrate = pen.feed_rate.unwrap_or(machine.feedrate());
        let geo_lines = opt.optimize(&geo_lines);
        if pen.tool_id != last_tool {
            last_tool = pen.tool_id;
            let mut context = Context::new();
            context.insert("tool_id", &pen.tool_id);
            program.extend(
                post_template
                    .render("penup", &context)?
                    .split("\n")
                    .map(|s| s.to_string()),
            );
            program.extend(
                post_template
                    .render("toolchange", &context)?
                    .split("\n")
                    .map(|s| s.to_string()),
            );
        }
        for line in geo_lines {
            let mut context = Context::new();
            if let Some(height) = machine.skim() {
                context.insert("skim", &height);
            };

            // TODO: This should definitely be using the keepdown strategy in the project..
            if machine.keepdown().is_some()
                && !pen_up
                && ((&line[0].x - last_x).powi(2) + (&line[0].y - last_y).powi(2)).sqrt()
                    < machine.keepdown().unwrap()
            {
                // Then we're doing a keepdown.
            } else if !pen_up {
                distance_down = 0.0;
                program.extend(
                    post_template
                        .render("penup_skim", &context)?
                        .split("\n")
                        .map(|s| s.to_string()),
                );
                pen_up = true;
            }

            let mut context = Context::new();
            context.insert("xmm", &line[0].x);
            context.insert("ymm", &line[0].y);
            program.extend(
                post_template
                    .render("moveto", &context)?
                    .split("\n")
                    .map(|s| s.to_string()),
            );
            // last_move = LastMove::Move;

            // Only do a pendown if we actually did a penup.
            // if machine.keepdown().is_some()
            //     && ((&line[0].x - last_x).powi(2) + (&line[0].y - last_y).powi(2)).sqrt()
            //         < machine.keepdown().unwrap()
            // {
            let pen_width = match &geometry.stroke {
                Some(stroke) => stroke.stroke_width,
                None => 0.5,
            };

            // TODO: This should definitely be used further down.
            let keepdown = ((&line[0].x - last_x).powi(2) + (&line[0].y - last_y).powi(2)).sqrt()
                < geometry.keepdown_strategy.threshold(pen_width);
            if !keepdown || pen_up {
                program.extend(
                    post_template
                        .render(
                            if let Some(_) = machine.skim() {
                                "pendown_skim"
                            } else {
                                "pendown"
                            },
                            &Context::new(),
                        )?
                        .split("\n")
                        .map(|s| s.to_string()),
                );
                pen_up = false;
            }

            for point in &line.0[1..] {
                #[allow(deprecated)]
                if pen_up == false {
                    distance_down += point.euclidean_distance(&Coord {
                        x: last_x,
                        y: last_y,
                    });
                } else {
                    distance_down = 0.0;
                }
                (last_x, last_y) = (point.x.clone(), point.y.clone());
                let mut context = Context::new();
                context.insert("xmm", &point.x);
                context.insert("ymm", &point.y);
                // context.insert("feedrate", &machine.feedrate());
                context.insert("feedrate", &feedrate);
                // if last_move == LastMove::Feed {
                //     program.extend(
                //         post_template
                //             .render("coords", &context)?
                //             .split("\n")
                //             .map(|s| s.to_string()),
                //     );
                // } else {
                program.extend(
                    post_template
                        .render("lineto", &context)?
                        .split("\n")
                        .map(|s| s.to_string()),
                );
                // last_move = LastMove::Feed;
                // }
                // This should really be configurable.
                if !pen_up && distance_down > 1500.0 {
                    distance_down = 0.;
                    program.extend(
                        post_template
                            .render("pendrop", &Context::new())?
                            .split("\n")
                            .map(|s| s.to_string()),
                    );
                }
            }
        }
    }
    program.extend(
        post_template
            .render("epilog", &Context::new())?
            .split("\n")
            .map(|s| s.to_string()),
    );

    Ok(program)
}

/// Converts geometry into MultiLineString
/// Lines/MultiLine are just passed through, whereas polygons
/// and rects are converted into their Perimeters
pub trait GeometryToMultiLineString {
    fn to_multi_line_strings(&self) -> MultiLineString<f64>;
}

impl GeometryToMultiLineString for Geometry<f64> {
    fn to_multi_line_strings(&self) -> MultiLineString<f64> {
        let mut out = MultiLineString::new(vec![]);
        match self {
            Geometry::Point(_) => todo!(),
            Geometry::Line(line) => out.0.push(LineString::from(line.clone())),
            Geometry::LineString(linestring) => out.0.push(linestring.clone()),
            Geometry::Polygon(poly) => {
                out.0.push(poly.exterior().clone());
                for interior in poly.interiors() {
                    out.0.push(interior.clone())
                }
            }
            Geometry::MultiPoint(_) => todo!(),
            Geometry::MultiLineString(mls) => {
                let mut mls = mls.clone();
                out.0.append(&mut mls.0);
            }
            Geometry::MultiPolygon(mp) => {
                for poly in mp {
                    out.0.push(poly.exterior().clone());
                    for interior in poly.interiors() {
                        out.0.push(interior.clone())
                    }
                }
            }
            Geometry::GeometryCollection(gc) => {
                for geo in gc {
                    let tmp_geometry = Geometry::from(geo.clone());
                    out.0.append(&mut tmp_geometry.to_multi_line_strings().0);
                }
            }
            Geometry::Rect(rekt) => {
                let poly = rekt.to_polygon();
                out.0.push(poly.exterior().clone());
                for interior in poly.interiors() {
                    out.0.push(interior.clone())
                }
            }
            Geometry::Triangle(tri) => {
                let poly = tri.to_polygon();
                out.0.push(poly.exterior().clone());
                for interior in poly.interiors() {
                    out.0.push(interior.clone())
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::project::{PenDetail, svg_to_geometries};
    use std::include_bytes;
    // use usvg::{Options, Tree};

    #[test]
    fn flatten_svg_geom() {
        let svg_data = include_bytes!("../../resources/test_groups_simple.svg");
        let mut opt = usvg::Options::default();
        opt.dpi = 25.4;
        if let Ok(rtree) = usvg::Tree::from_data(svg_data, &opt) {
            let geometry = svg_to_geometries(&rtree, 1., 1., true, &vec![PenDetail::default()]);
            for geo in geometry {
                let _lines = geo.geometry.to_multi_line_strings();
                // println!("Lines are: {:?}", lines);
            }
        }
    }

    #[test]
    fn test_post() {}
}
