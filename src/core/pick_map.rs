use geo::{Geometry, Rect};
use skia_safe::paint::Style;
use skia_safe::{AlphaType, Bitmap, BlendMode, Color, ImageInfo, Paint, Path, surfaces};
use std::collections::BTreeSet;
use std::sync::mpsc::Sender;
use std::u32;

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::{PenDetail, Project};
// The number of pick points per mm, so our image size will be mm * PICKS_PER_MM
pub const PICKS_PER_MM: usize = 4;

pub const PICK_EXPANSION_MAP: [(i32, i32); 25] = [
    (0, 0),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1), // First circle
    (2, 0),  // Second circle
    (2, 1),
    (2, 2),
    (1, 2),
    (0, 2),
    (-1, 2),
    (-2, 2),
    (-2, 1),
    (-2, 0),
    (-2, -1),
    (-2, -2),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -2),
    (2, -1), // End second circle
];

impl super::ApplicationCore {
    pub fn try_pick(&self, x: f64, y: f64) -> Option<u32> {
        if let Some((pick_img, extents)) = &self.pick_image {
            if x < extents.0
                || y < extents.1
                || y > (extents.1 + extents.3)
                || x > (extents.0 + extents.2)
            {
                // println!("Outside of extents.");
                return None;
            }

            // println!("Click MM are {},{}", x, y);
            let xpx = (x - extents.0).ceil() as usize * PICKS_PER_MM;
            let ypx = (y - extents.1).ceil() as usize * PICKS_PER_MM;
            let xspan = extents.2.ceil() as usize * PICKS_PER_MM;
            // println!("Extents are: {:?}", extents);
            // println!("XPX and YPX are {},{}", xpx, ypx);
            let pxextents = (
                extents.0 * PICKS_PER_MM as f64,
                extents.1 * PICKS_PER_MM as f64,
                extents.2 * PICKS_PER_MM as f64,
                extents.3 * PICKS_PER_MM as f64,
            );
            for (xofs, yofs) in PICK_EXPANSION_MAP.clone() {
                let xpick = xpx as i32 + xofs;
                let ypick = ypx as i32 + yofs;
                // println!("Trying pick at {},{}", xpick, ypick);
                if !(0..(pxextents.2 as i32)).contains(&xpick) {
                    // println!("Out of extents: X{}->{:?}", xpick, &extents);
                    continue;
                }
                if !(0..(pxextents.3 as i32)).contains(&ypick) {
                    // println!("Out of extents: Y{}->{:?}", ypick, &extents);
                    continue;
                }
                if let Some(id) = pick_img.get((xpick + ypick * xspan as i32) as usize) {
                    if *id == u32::MAX {
                        continue;
                    }
                    // println!("Picked a color of {:?}", id);
                    if self.project.plot_geometry.get(*id as usize).is_some() {
                        return Some(*id);
                        // return Some(geo.id.clone() as u32);
                    }
                }
            }
        }
        return None;
    }
    // pub(crate) fn try_pick(x: f64, y: f64) -> u32 {}
}

pub(crate) fn render_pick_map(
    project: &Project,
    _state_change_out: &Sender<ApplicationStateChangeMsg>,
) -> Result<(Vec<u32>, Rect), anyhow::Error> {
    let (extents, geo) = (project.extents(), project.plot_geometry.clone());

    let resolution = (
        (PICKS_PER_MM as f64 * extents.width().ceil()) as usize,
        (PICKS_PER_MM as f64 * extents.height().ceil()) as usize,
    );

    let (xofs, yofs) = extents.min().x_y();
    let sx = resolution.0 as f32 / extents.width() as f32;
    let sy = resolution.1 as f32 / extents.height() as f32;
    // println!("Extents are: {},{}", extents.width(), extents.height());
    // println!("SX IS {}", sx);
    let mut surface =
        surfaces::raster_n32_premul((resolution.0 as i32, resolution.1 as i32)).expect("surface");
    let canvas = surface.canvas();
    canvas.clear(u32::MAX);
    canvas.translate((-xofs as f32 * sx, -yofs as f32 * sy));
    canvas.scale((sx, sy));
    let mut paint = Paint::default();
    paint
        .set_style(Style::Stroke)
        .set_anti_alias(false) // We want sharp edges and no "blurs"
        .set_stroke_cap(skia_safe::PaintCap::Round)
        .set_blend_mode(BlendMode::Src)
        .set_alpha(255)
        .set_color(Color::new(u32::MAX));
    for (id, pg) in geo.iter().enumerate() {
        let _mid = extents.center();

        let pen = project
            .pen_by_uuid(pg.pen_uuid)
            .unwrap_or(PenDetail::default());
        // let pen = pg.stroke.clone().unwrap_or(PenDetail::default());
        // let pen_uuid = pg.pen_uuid;
        paint.set_stroke_width((pen.stroke_width as f32 * PICKS_PER_MM as f32) / 4.);
        let geo_color = Color::new(id as u32 | 0xff000000);
        paint.set_color(geo_color);

        // if let Geometry::MultiLineString(mls) = &pg.geometry {
        let mls = pg.lines();
        let _line_count = mls.0.len();
        for (_idx, line) in mls.0.clone().iter().enumerate() {
            let mut path = Path::new();
            if let Some(p0) = line.0.first() {
                path.move_to((p0.x as f32, p0.y as f32));
                for coord in line.0.iter().skip(1) {
                    path.line_to((coord.x as f32, coord.y as f32));
                }
            }
            surface.canvas().draw_path(&path, &paint);
        }
        // }
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
    let mut pixels = bmap.peek_pixels().expect("Failed to peek pixel data.");
    let data = pixels.bytes_mut().expect("Failed to get back pixel data.");
    let u32_data: Vec<u32> = data
        .chunks_exact(4)
        .map(|chunk| {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let _a = chunk[3];
            let out = b as u32 + g as u32 * 256 + r as u32 * 65536;
            if out != 0xffffff { out } else { u32::MAX }
        })
        .collect();

    let mut found: BTreeSet<u32> = BTreeSet::new();
    for pixel in &u32_data {
        found.insert(*pixel);
    }
    Ok((u32_data, extents))
}

#[cfg(test)]
pub mod test {}
