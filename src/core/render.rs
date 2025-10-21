use egui::{ColorImage, Context};
use egui_extras::Size;
use geo::{BoundingRect, Coord, Geometry, GeometryCollection, Point, Rect, Scale, Translate};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use tiny_skia::{LineCap, Path, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};
use usvg::{Options, Tree};

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::Project;

pub(crate) fn render_svg_preview(
    project: &Project,
    extents: (f64, f64, f64, f64),
    resolution: (usize, usize),
    state_change_out: &Sender<ApplicationStateChangeMsg>,
) -> Result<ColorImage, anyhow::Error> {
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(0, 0, 0, 255);
    paint.blend_mode = tiny_skia::BlendMode::Source;
    // paint.anti_alias = true;

    let extents = Rect::new(
        Coord {
            x: extents.0,
            y: extents.1,
        },
        Coord {
            x: extents.2 + extents.0,
            y: extents.3 + extents.1,
        },
    );
    println!("Extents incoming are {:?}", extents);
    let mut pixmap =
        Pixmap::new(resolution.0 as u32, resolution.1 as u32).expect("Failed to create pixmap!");
    let (xofs, yofs) = extents.min().x_y();
    let (xofs, yofs) = (xofs as f32, yofs as f32); // In case it's negative, which happens sometimes.

    let stroke_width = (resolution.0 as f32 / extents.width() as f32) * 2.5;
    let sx = resolution.0 as f32 / extents.width() as f32;
    let sy = resolution.1 as f32 / extents.height() as f32;

    let transform = Transform::from_row(
        sx,
        0.,
        0.,
        sy,
        -extents.min().x as f32 * sx,
        -extents.min().y as f32 * sy,
    );

    let line_count = &project.geometry.len();
    let mut i = 0;
    let mut pb = PathBuilder::new();
    let mut stroke = Stroke::default();
    stroke.line_cap = LineCap::Round;
    stroke.width = 0.1; // * scale;
    for pg in &project.geometry {
        i += 1;
        state_change_out.send(ApplicationStateChangeMsg::ProgressMessage {
            message: "Scaling preview...".to_string(),
            percentage: (20 * i) / line_count,
        });
        if let Geometry::MultiLineString(mls) = &pg.geometry {
            for line in &mls.0 {
                // let mut pb = PathBuilder::new();
                if let Some(p0) = line.0.first() {
                    pb.move_to(p0.x as f32, p0.y as f32);
                    for coord in line.0.iter().skip(1) {
                        pb.line_to(coord.x as f32, coord.y as f32);
                    }
                }
                // let path = pb.finish(); // .expect("Failed to finish line.");

                // stroke.dash = StrokeDash::new(vec![10.0, 10.0], 0.0);
                // let path = pb.finish(); // .expect("Failed to finish line.");
                // if let Some(path) = path {
                //     pixmap.stroke_path(&path, &paint, &stroke, transform, None)
                // }
            }
        }
    }
    let path = pb.finish(); // .expect("Failed to finish line.");
    if let Some(path) = path {
        pixmap.stroke_path(&path, &paint, &stroke, transform, None)
    }
    state_change_out
        .send(ApplicationStateChangeMsg::ProgressMessage {
            message: "Rendering preview...".to_string(),
            percentage: 90,
        })
        .expect("Failed to send progress. Closed socket?");
    let cimg: ColorImage = ColorImage::from_rgba_premultiplied(
        [pixmap.width() as usize, pixmap.height() as usize],
        pixmap.data(),
    );
    state_change_out
        .send(ApplicationStateChangeMsg::ProgressMessage {
            message: "Rendering preview...".to_string(),
            percentage: 95,
        })
        .expect("Failed to send progress. Closed socket?");
    state_change_out
        .send(ApplicationStateChangeMsg::ProgressMessage {
            message: "Done preview.".to_string(),
            percentage: 100,
        })
        .expect("Failed to send state change message.");
    Ok(cimg)
}

pub(crate) fn _render_svg_preview(tree: &Tree) -> Result<ColorImage, anyhow::Error> {
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(128, 32, 32, 255);
    paint.anti_alias = true;
    let path = {
        let mut pb = PathBuilder::new();
        const RADIUS: f32 = 100.0;
        const CENTER: f32 = 250.0;
        pb.move_to(CENTER + RADIUS, CENTER);
        for i in 1..8 {
            let a = 2.6927937 * i as f32;
            pb.line_to(CENTER + RADIUS * a.cos(), CENTER + RADIUS * a.sin());
        }
        pb.finish().unwrap()
    };

    let mut stroke = Stroke::default();
    stroke.width = 6.0;
    stroke.line_cap = LineCap::Round;
    stroke.dash = StrokeDash::new(vec![10.0, 10.0], 0.0);

    let mut pixmap = Pixmap::new(500, 500).unwrap();
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    let cimg: ColorImage = ColorImage::from_rgba_premultiplied(
        [pixmap.width() as usize, pixmap.height() as usize],
        pixmap.data(),
    );
    Ok(cimg)
}
