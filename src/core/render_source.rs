use egui::ColorImage;
use geo::{Coord, Geometry, Rect};
use std::sync::mpsc::{Receiver, Sender};
use tiny_skia::{LineCap, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};
use usvg::Tree;

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::Project;

pub(crate) fn render_svg_preview(
    project: &Project,
    extents: (f64, f64, f64, f64),
    resolution: (usize, usize),
    state_change_out: &Sender<ApplicationStateChangeMsg>,
    cancel: &Receiver<()>,
) -> Result<ColorImage, anyhow::Error> {
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(0, 0, 0, 255);
    paint.blend_mode = tiny_skia::BlendMode::Source;
    paint.anti_alias = true;
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
    let mut pixmap =
        Pixmap::new(resolution.0 as u32, resolution.1 as u32).expect("Failed to create pixmap!");
    let (xofs, yofs) = extents.min().x_y();
    // let (_xofs, _yofs) = (xofs as f32, yofs as f32); // In case it's negative, which happens sometimes.

    let _stroke_width = (resolution.0 as f32 / extents.width() as f32) * 2.5;
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

    let geo_count = &project.geometry.len();
    let mut i = 0;
    // let mut pb = PathBuilder::new();
    let geo_len = project.geometry.len();

    for pg in &project.geometry {
        let mut pb = PathBuilder::new();
        let mut stroke = Stroke::default();
        stroke.line_cap = LineCap::Round;
        stroke.width = 0.1; // * scale;
        i += 1;
        state_change_out
            .send(ApplicationStateChangeMsg::ProgressMessage {
                message: "Scaling preview...".to_string(),
                percentage: 20 + (20 * i) / geo_count,
            })
            .expect("Failed to send back to core. Dead core?");
        if let Geometry::MultiLineString(mls) = &pg.geometry {
            let line_count = mls.0.len();
            for (idx, line) in mls.0.clone().iter().enumerate() {
                //.enumerate() {
                if idx % 10 == 0 {
                    let pc_prog =
                        (20 + (20 * i) / geo_count) + (20 * idx) / (geo_count * line_count);
                    state_change_out
                        .send(ApplicationStateChangeMsg::ProgressMessage {
                            message: "Scaling preview...".to_string(),
                            percentage: pc_prog,
                        })
                        .expect("Failed to send back to core. Dead core?");
                }
                match cancel.try_recv() {
                    Ok(_) => {
                        return Err(anyhow::anyhow!("Got a cancel on render."));
                    }
                    Err(_) => (),
                }
                // let mut pb = PathBuilder::new();
                if let Some(p0) = line.0.first() {
                    pb.move_to(p0.x as f32, p0.y as f32);
                    for coord in line.0.iter().skip(1) {
                        pb.line_to(coord.x as f32, coord.y as f32);
                    }
                }
            }
            let path = pb.finish();
            if let Some(path) = path {
                pixmap.stroke_path(&path, &paint, &stroke, transform, None)
            }
        }
    }
    match cancel.try_recv() {
        Ok(_) => return Err(anyhow::anyhow!("Got a cancel on render.")),
        Err(_) => (),
    }
    state_change_out
        .send(ApplicationStateChangeMsg::ProgressMessage {
            message: "Rendering preview...".to_string(),
            percentage: 90,
        })
        .expect("Failed to send progress. Closed socket?");
    match cancel.try_recv() {
        Ok(_) => return Err(anyhow::anyhow!("CANCEL")),
        Err(_) => (),
    }
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
