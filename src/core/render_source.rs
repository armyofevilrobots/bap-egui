use egui::ColorImage;
use geo::Rect;
use skia_safe::paint::Style;
use skia_safe::{AlphaType, Bitmap, Color, ImageInfo, Paint, Path, PathEffect, surfaces};
use std::collections::BTreeSet;
use std::ops::Rem;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Instant, SystemTime};

use super::ApplicationCore;
use super::commands::ApplicationStateChangeMsg;
use super::project::{PenDetail, Project};

const MAX_TEXTURE_SIZE: usize = 8192; // Maximum size in any dimension of the preview images.
// This can never be more than 16384. That's the max
// size of the underlying EGUI framework.
impl ApplicationCore {
    pub fn handle_request_source_image(
        &mut self,
        zoom: f64,
        rotation: Option<((f64, f64), f64)>,
        translation: Option<(f64, f64)>,
        scale_around: Option<((f64, f64), f64)>,
    ) {
        let phase = (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .expect("Should never fail to calc unix seconds.");
        let phase = phase.as_secs_f64();
        let (cimg, extents_out) = match render_source_preview(
            &self.project,
            zoom,
            rotation.clone(),
            translation.clone(),
            scale_around.clone(),
            self.picked.clone(),
            phase,
            &self.state_change_out,
            &self.cancel_render,
        ) {
            Ok((cimg, xo)) => {
                // eprintln!("Rendered CIMG of {:?}", cimg.size);
                (
                    Some(cimg),
                    (xo.min().x, xo.min().y, xo.width(), xo.height()),
                )
            }
            Err(_err) => {
                // eprintln!("Error rendering source image: {:?}", err);
                let min_x = self.project.extents().min().x;
                let min_y = self.project.extents().min().y;

                (
                    None,
                    (
                        min_x,
                        min_y,
                        self.project.extents().width(),
                        self.project.extents().height(),
                    ),
                )
            }
        };

        if let Some(cimg) = cimg {
            self.last_render = Some((cimg.clone(), extents_out.clone()));
            self.last_rendered = Instant::now();
            self.state_change_out
                .send(ApplicationStateChangeMsg::UpdateSourceImage {
                    image: cimg,
                    extents: extents_out,
                    rotation: rotation.clone(),
                })
                .unwrap_or_else(|_err| {
                    self.shutdown = true;
                    eprintln!("Failed to send message from bap core. Shutting down.");
                });
        }
        self.ctx.request_repaint();
    }
}

pub(crate) fn render_source_preview(
    project: &Project,
    // extents: (f64, f64, f64, f64),
    // resolution: (usize, usize),
    zoom: f64,
    rotate: Option<((f64, f64), f64)>,
    translate: Option<(f64, f64)>,
    scale_around: Option<((f64, f64), f64)>,
    picked: Option<BTreeSet<u32>>,
    phase: f64,
    _state_change_out: &Sender<ApplicationStateChangeMsg>,
    cancel: &Receiver<()>,
) -> Result<(ColorImage, Rect), anyhow::Error> {
    let (extents, geo) = if let Some(((xc, yc), rot)) = &rotate {
        let geo = project.rotate_geometry_around_point((*xc, *yc), *rot, &picked);
        (Project::calc_extents_for_geometry(&geo), geo)
    } else {
        (project.extents(), project.plot_geometry.clone())
    };
    let (extents, geo) = if let Some((dx, dy)) = &translate {
        let new_geo = Project::translate_arbitrary_geo(&geo, (*dx, *dy), &picked);
        (Project::calc_extents_for_geometry(&new_geo), new_geo)
    } else {
        (extents, geo)
    };
    let (extents, geo) = if let Some(((dx, dy), factor)) = &scale_around {
        let new_geo = Project::scale_geometry_around_point(&geo, (*dx, *dy), *factor, &picked);
        (Project::calc_extents_for_geometry(&new_geo), new_geo)
    } else {
        (extents, geo)
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
    for (id, pg) in geo.clone().iter().enumerate() {
        // let pen = pg.stroke.clone().unwrap_or(PenDetail::default());
        let pen = project
            .pen_by_uuid(pg.pen_uuid)
            .unwrap_or(PenDetail::default());
        paint.set_path_effect(None);
        paint.set_stroke_cap(skia_safe::PaintCap::Round);
        paint.set_stroke_width(pen.stroke_width as f32);
        if let Some(pickset) = &picked {
            // let id = pg.id as u32;
            if pickset.contains(&(id as u32)) {
                let dash = 16.0 / sx;
                let phase = phase.rem((dash * 4.) as f64) as f32;
                // println!("Matched pick with phase {}", phase);
                // println!("Dash is {} sx is {}", dash, sx);
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

        let mls = &pg.lines();
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
