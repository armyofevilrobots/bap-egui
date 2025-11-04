use egui::{Color32, ColorImage};
use gcode::GCode;
use geo::{Coord, Geometry, Rect};
use skia_safe::paint::Style;
use skia_safe::{
    AlphaType, Bitmap, Color, Data, EncodedImageFormat, ImageInfo, Paint, PaintStyle, Path,
    PathEffect, Surface, surfaces,
};
use std::mem;
use std::sync::mpsc::{Receiver, Sender};
use tiny_skia::Shader;
use usvg::Tree;

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::{PenDetail, Project};
use tiny_skia::{LineCap, PathBuilder, Pixmap, Stroke, StrokeDash, Transform};

pub(crate) fn render_svg_preview(
    project: &Project,
    extents: (f64, f64, f64, f64),
    resolution: (usize, usize),
    state_change_out: &Sender<ApplicationStateChangeMsg>,
    cancel: &Receiver<()>,
) -> Result<ColorImage, anyhow::Error> {
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
    let (xofs, yofs) = extents.min().x_y();
    // let (_xofs, _yofs) = (xofs as f32, yofs as f32); // In case it's negative, which happens sometimes.

    let _stroke_width = (resolution.0 as f32 / extents.width() as f32) * 2.5;
    let sx = resolution.0 as f32 / extents.width() as f32;
    let sy = resolution.1 as f32 / extents.height() as f32;
    let mut surface =
        surfaces::raster_n32_premul((resolution.0 as i32, resolution.1 as i32)).expect("surface");
    let mut paint = Paint::default();
    paint.set_color(Color::BLUE);
    paint.set_style(Style::Stroke);
    paint.set_anti_alias(true);
    // paint.set_stroke_width(0.1);
    paint.set_stroke_cap(skia_safe::PaintCap::Round);
    paint.set_shader(None);
    let canvas = surface.canvas();
    // canvas.draw_circle((0., 0.), 25., &paint);
    canvas.translate((-xofs as f32 * sx, -yofs as f32 * sy));
    canvas.scale((sx, sy));
    let mid = extents.center();
    for pg in &project.geometry {
        let pen = pg.stroke.clone().unwrap_or(PenDetail::default());
        paint.set_stroke_width(pen.stroke_width as f32);
        paint.set_alpha_f(pen.stroke_density as f32);
        let color_code = csscolorparser::parse(pen.color.as_str()).unwrap_or_default();
        let [r, g, b, a] = color_code.to_rgba8();

        paint.set_color(Color::from_argb(a, r, g, b));
        paint.set_path_effect(None);

        if let Geometry::MultiLineString(mls) = &pg.geometry {
            let line_count = mls.0.len();
            for (idx, line) in mls.0.clone().iter().enumerate() {
                let mut path = Path::new();
                let mut exit = false;
                loop {
                    match cancel.try_recv() {
                        Ok(_) => {
                            exit = true;
                        }
                        Err(_) => break,
                    }
                    if exit {
                        return Err(anyhow::anyhow!("Got a cancel on render."));
                    }
                }
                if let Some(p0) = line.0.first() {
                    path.move_to((p0.x as f32, p0.y as f32));
                    for coord in line.0.iter().skip(1) {
                        path.line_to((coord.x as f32, coord.y as f32));
                    }
                }
                surface.canvas().draw_path(&path, &paint);
            }
        }
    }
    let mut context = surface.direct_context();
    let mut bmap = Bitmap::new();
    let dims = surface.image_info().dimensions().clone();
    let result = bmap.set_info(
        &ImageInfo::new(
            surface.image_info().dimensions().clone(),
            skia_safe::ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        ),
        None,
    );
    bmap.alloc_pixels();
    let result = surface.read_pixels_to_bitmap(&bmap, (0, 0));
    let pixels = bmap.peek_pixels().unwrap();
    let cimg: ColorImage = ColorImage::from_rgba_premultiplied(
        [pixels.width() as usize, pixels.height() as usize],
        pixels.bytes().expect("Failed to get pixels!"),
    );
    Ok(cimg)
}

pub(crate) fn _old_tiny_skia_render_svg_preview(
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
