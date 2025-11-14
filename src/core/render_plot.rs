use super::post::LastMove;
use egui::ColorImage;
// TODO: I really need to write my own that has stateful G/M codes to remember the
// previous move type, so that just coords can be used to reduce the outgoing bitrate
use gcode::GCode;
use geo::{Coord, Geometry, Rect};
use skia_safe::paint::Style;
use skia_safe::{AlphaType, Bitmap, Color, ImageInfo, Paint, Path, PathEffect, surfaces};
use std::sync::mpsc::{Receiver, Sender};

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::Project;

fn machine_coords_to_model_coords(xy: (f64, f64), origin: (f64, f64)) -> (f64, f64) {
    let x = xy.0 + origin.0;
    let y = origin.1 - xy.1;
    (x, y)
}

pub(crate) fn render_plot_preview(
    project: &Project,
    // gc_item: &Vec<GCode>,
    extents: (f64, f64, f64, f64),
    progress: (usize, usize),
    resolution: (usize, usize),
    _state_change_out: &Sender<ApplicationStateChangeMsg>,
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
    let (_xofs, _yofs) = (xofs as f32, yofs as f32); // In case it's negative, which happens sometimes.

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
        paint.set_stroke_width(0.);
        paint.set_alpha_f(0.5);
        paint.set_color(Color::BLACK);
        paint.set_path_effect(PathEffect::dash(&[0.2, 0.5], 0.));

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

    let origin = if let Some(origin) = project.origin() {
        origin
    } else {
        (0., 0.)
    };
    let mut px = 0.;
    let mut py = 0.;
    let mut last_move = LastMove::None;
    for (idx, line) in project
        .program()
        .unwrap_or_else(|| Box::new(Vec::new()))
        .iter()
        .take(progress.0)
        .enumerate()
    {
        // println!("GOT LINE: {}", line);
        let gcodes = gcode::parse(line);
        let mut path = Path::new();
        paint.set_stroke_width(0.25);
        let xy = machine_coords_to_model_coords((px as f64, py as f64), origin);
        path.move_to((xy.0 as f32, xy.1 as f32));
        // println!("GCODE: {:?}", gcode);
        if idx < progress.1 {
            paint.set_path_effect(None);
        } else {
            paint.set_path_effect(PathEffect::dash(&[0.2, 0.5], 0.));
        }
        for gcode_item in gcodes {
            // println!("GCODE: {:?}", gcode_item);
            match gcode_item.mnemonic() {
                gcode::Mnemonic::General => {
                    px = gcode_item.value_for('X').unwrap_or(px);
                    py = gcode_item.value_for('Y').unwrap_or(py);
                    let xy = machine_coords_to_model_coords((px as f64, py as f64), origin);
                    let xy = (xy.0 as f32, xy.1 as f32);

                    if gcode_item.major_number() == 0 {
                        paint.set_color(Color::RED);
                        path.line_to(xy);
                        last_move = LastMove::Move;
                        // path.move_to(xy);
                    } else if gcode_item.major_number() == 1 {
                        paint.set_color(Color::BLUE.with_a(128));
                        path.line_to(xy);
                        last_move = LastMove::Feed;
                    }
                }
                gcode::Mnemonic::Miscellaneous => (), //last_move = LastMove::None,
                gcode::Mnemonic::ProgramNumber => (), //last_move = LastMove::None,
                gcode::Mnemonic::ToolChange => (),    //last_move = LastMove::None,
            }
            surface.canvas().draw_path(&path, &paint);
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
