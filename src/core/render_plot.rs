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

fn machine_coords_to_model_coords(xy: (f64, f64), origin: (f64, f64)) -> (f64, f64) {
    let x = xy.0 + origin.0;
    let y = origin.1 - xy.1;
    (x, y)
}

pub(crate) fn render_plot_preview(
    project: &Project,
    gc_item: &Vec<GCode>,
    extents: (f64, f64, f64, f64),
    progress: (usize, usize),
    resolution: (usize, usize),
    state_change_out: &Sender<ApplicationStateChangeMsg>,
    cancel: &Receiver<()>,
) -> Result<ColorImage, anyhow::Error> {
    // println!(
    //     "Requested plot preview at progress {}/{}",
    //     progress.0, progress.1
    // );
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
    // canvas.draw_circle((mid.x as f32, mid.y as f32), 5., &paint);
    // canvas.draw_circle((extents.min().x as f32, extents.min().y as f32), 5., &paint);
    // println!("EXTENTS: {:?}  -  ", extents);
    for pg in &project.geometry {
        // paint.set_stroke_width(
        //     pg.stroke
        //         .clone()
        //         .unwrap_or(PenDetail::default())
        //         .stroke_width as f32
        //         / 2.,
        // );
        paint.set_stroke_width(0.);
        paint.set_alpha_f(0.5);
        paint.set_color(Color::BLACK);
        paint.set_path_effect(PathEffect::dash(&[0.2, 0.5], 0.));
        // println!("STROKE IS {}", paint.stroke_width());
        // let mut pb = PathBuilder::new();
        // let mut stroke = Stroke::default();
        // stroke.line_cap = LineCap::Round;
        // stroke.width = 0.1; // * scale;

        if let Geometry::MultiLineString(mls) = &pg.geometry {
            let line_count = mls.0.len();
            for (idx, line) in mls.0.clone().iter().enumerate() {
                let mut path = Path::new();
                //.enumerate() {
                let mut exit = false;
                loop {
                    match cancel.try_recv() {
                        Ok(_) => {
                            // println!("EXITING DUE TO CANCEL");
                            exit = true;
                        }
                        Err(_) => break,
                    }
                    if exit {
                        return Err(anyhow::anyhow!("Got a cancel on render."));
                    }
                }
                // let mut pb = PathBuilder::new();
                if let Some(p0) = line.0.first() {
                    // pb.move_to(p0.x as f32, p0.y as f32);
                    path.move_to((p0.x as f32, p0.y as f32));
                    for coord in line.0.iter().skip(1) {
                        path.line_to((coord.x as f32, coord.y as f32));
                    }
                }
                surface.canvas().draw_path(&path, &paint);
            }

            // let path = pb.finish();
            // if let Some(path) = path {
            //     pixmap.stroke_path(&path, &paint, &stroke, transform, None)
            // }
        }
    }

    let origin = project
        .origin()
        .expect("How did I get gcode with no origin?!");
    let mut px = 0.;
    let mut py = 0.;
    // println!("About to plot progress... {}/{}", progress.0, progress.1);
    // println!(
    //     "GCODE length is {} and progress is {}/{}",
    //     gc_item.len(),
    //     progress.0,
    //     progress.1
    // );
    // for (idx, gcode) in gcode.iter().take(progress.0).enumerate() {
    for (idx, line) in project
        .program()
        .unwrap_or_else(|| {
            // println!("NO PROGRAM?!?!?!?!");
            Box::new(Vec::new())
        })
        .iter()
        .take(progress.0)
        .enumerate()
    {
        // println!("GOT LINE: {}", line);
        let gcodes = gcode::parse(line);
        let mut path = Path::new();
        paint.set_stroke_width(1.);
        let xy = machine_coords_to_model_coords((px as f64, py as f64), origin);
        path.move_to((xy.0 as f32, xy.1 as f32));
        // println!("GCODE: {:?}", gcode);
        if idx < progress.1 {
            paint.set_path_effect(None);
        } else {
            paint.set_path_effect(PathEffect::dash(&[0.2, 0.5], 0.));
        }
        for gcode_item in gcodes {
            match gcode_item.mnemonic() {
                gcode::Mnemonic::General => {
                    px = gcode_item.value_for('X').unwrap_or(px);
                    py = gcode_item.value_for('Y').unwrap_or(py);
                    // println!("GXX -> {},{}", px, py);
                    let xy = machine_coords_to_model_coords((px as f64, py as f64), origin);
                    let xy = (xy.0 as f32, xy.1 as f32);

                    if gcode_item.major_number() == 0 {
                        paint.set_color(Color::RED);
                        path.line_to(xy);
                        // path.move_to(xy);
                    } else if gcode_item.major_number() == 1 {
                        paint.set_color(Color::BLUE);
                        path.line_to(xy);
                    }
                }
                gcode::Mnemonic::Miscellaneous => (),
                gcode::Mnemonic::ProgramNumber => (),
                gcode::Mnemonic::ToolChange => (),
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
    // println!("RESULT of alloc bmap: {:?}", result);
    let result = surface.read_pixels_to_bitmap(&bmap, (0, 0));
    // println!("RESULT of READ TO bmap: {:?}", result);
    let pixels = bmap.peek_pixels().unwrap();
    // let pixels = surface.peek_pixels().unwrap();
    // println!("COLORSPACE: {:?}", pixels.color_space());
    // println!("PIXELS COLORTYPE: {:?}", pixels.color_type());
    let cimg: ColorImage = ColorImage::from_rgba_premultiplied(
        [pixels.width() as usize, pixels.height() as usize],
        pixels.bytes().expect("Failed to get pixels!"),
    );
    // println!("RENDERED AND OUTPUT");
    // let cimg: ColorImage = ColorImage::filled([resolution.0, resolution.1], Color32::TRANSPARENT);
    Ok(cimg)
}
