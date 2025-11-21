use egui::ColorImage;
use geo::{Coord, Geometry, Rect};
use skia_safe::paint::Style;
use skia_safe::{AlphaType, Bitmap, Color, ImageInfo, Paint, Path, PathEffect, surfaces};
use std::ops::Rem;
use std::sync::mpsc::{Receiver, Sender};
use usvg::tiny_skia_path::Scalar;

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::{PenDetail, Project};
use tiny_skia::{LineCap, PathBuilder, Pixmap, Stroke, Transform};

const MAX_TEXTURE_SIZE: usize = 8192; // Maximum size in any dimension of the preview images.
// This can never be more than 16384. That's the max
// size of the underlying EGUI framework.

pub(crate) fn render_svg_preview(
    project: &Project,
    // extents: (f64, f64, f64, f64),
    // resolution: (usize, usize),
    zoom: f64,
    rotate: Option<((f64, f64), f64)>,
    picked: Option<u32>,
    phase: f64,
    _state_change_out: &Sender<ApplicationStateChangeMsg>,
    cancel: &Receiver<()>,
) -> Result<(ColorImage, Rect), anyhow::Error> {
    let (extents, geo) = if let Some(((xc, yc), rot)) = &rotate {
        let geo = project.rotate_geometry_around_point((*xc, *yc), *rot);
        (Project::calc_extents_for_geometry(&geo), geo)
    } else {
        (project.extents(), project.geometry.clone())
    };

    let mut resolution = (
        (zoom * extents.width().ceil()) as usize,
        (zoom * extents.height().ceil()) as usize,
    );

    // Cut the render size for fast rotation
    if rotate.is_some() {
        resolution = (resolution.0 / 2, resolution.1 / 2);
    }

    if resolution.0 > MAX_TEXTURE_SIZE {
        let ratio = resolution.0 as f32 / MAX_TEXTURE_SIZE as f32;
        resolution = (
            (resolution.0 as f32 / ratio) as usize,
            (resolution.1 as f32 / ratio) as usize,
        );
    }
    if resolution.1 > MAX_TEXTURE_SIZE {
        let ratio = resolution.1 as f32 / MAX_TEXTURE_SIZE as f32;
        resolution = (
            (resolution.0 as f32 / ratio) as usize,
            (resolution.1 as f32 / ratio) as usize,
        );
    }

    let (xofs, yofs) = extents.min().x_y();

    let _stroke_width = (resolution.0 as f32 / extents.width() as f32) * 2.5;
    // TODO: This resolution needs to be scaled if we have rotated.
    let sx = resolution.0 as f32 / extents.width() as f32;
    let sy = resolution.1 as f32 / extents.height() as f32;
    let mut surface =
        surfaces::raster_n32_premul((resolution.0 as i32, resolution.1 as i32)).expect("surface");
    let mut paint = Paint::default();
    paint.set_color(Color::BLUE);
    paint.set_style(Style::Stroke);
    paint.set_anti_alias(true);
    // paint.set_stroke_width(0.1);
    paint.set_shader(None);
    let canvas = surface.canvas();
    // canvas.draw_circle((0., 0.), 25., &paint);
    canvas.translate((-xofs as f32 * sx, -yofs as f32 * sy));
    canvas.scale((sx, sy));
    let _mid = extents.center();
    for pg in &geo {
        let pen = pg.stroke.clone().unwrap_or(PenDetail::default());
        paint.set_path_effect(None);
        paint.set_stroke_cap(skia_safe::PaintCap::Round);
        paint.set_stroke_width(pen.stroke_width as f32);
        if let Some(id) = picked {
            if id == pg.id as u32 {
                let dash = 16.0 / sx;
                let phase = phase.rem((dash * 4.) as f64) as f32;
                println!("Matched pick with phase {}", phase);
                println!("Dash is {} sx is {}", dash, sx);
                paint.set_path_effect(PathEffect::dash(&[dash, dash, dash, dash], phase));
                paint.set_stroke_cap(skia_safe::PaintCap::Square);
                paint.set_stroke_width(1. / sx as f32);
            }
        }
        paint.set_alpha_f(pen.stroke_density as f32);
        // let color_code = pen.color.clone(); //csscolorparser::parse(pen.color.as_str()).unwrap_or_default();
        let [r, g, b, a] = pen.color.to_rgba8();

        paint.set_color(Color::from_argb(a, r, g, b));
        // paint.set_path_effect(None);

        if let Geometry::MultiLineString(mls) = &pg.geometry {
            //&pg.geometry {
            let _line_count = mls.0.len();
            for (_idx, line) in mls.0.clone().iter().enumerate() {
                let mut path = Path::new();
                if let Ok(_msg) = cancel.try_recv() {
                    return Err(anyhow::anyhow!("Got a cancel on render."));
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
    let mut bmap = Bitmap::new();
    let _result = bmap.set_info(
        &ImageInfo::new(
            surface.image_info().dimensions().clone(),
            skia_safe::ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        ),
        None,
    );
    bmap.alloc_pixels();
    let _result = surface.read_pixels_to_bitmap(&bmap, (0, 0));
    let pixels = bmap.peek_pixels().unwrap();
    let cimg: ColorImage = ColorImage::from_rgba_premultiplied(
        [pixels.width() as usize, pixels.height() as usize],
        pixels.bytes().expect("Failed to get pixels!"),
    );
    Ok((cimg, extents))
}
