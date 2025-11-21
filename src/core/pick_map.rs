use geo::{Geometry, Rect};
use skia_safe::paint::Style;
use skia_safe::{AlphaType, Bitmap, BlendMode, Color, ImageInfo, Paint, Path, surfaces};
use std::sync::mpsc::Sender;
use std::u32;

use crate::core::commands::ApplicationStateChangeMsg;
use crate::core::project::{PenDetail, Project};
// The number of pick points per mm, so our image size will be mm * PICKS_PER_MM
pub const PICKS_PER_MM: usize = 4;

pub(crate) fn render_pick_map(
    project: &Project,
    _state_change_out: &Sender<ApplicationStateChangeMsg>,
) -> Result<(Vec<u32>, Rect), anyhow::Error> {
    let (extents, geo) = (project.extents(), project.geometry.clone());

    let resolution = (
        (PICKS_PER_MM as f64 * extents.width().ceil()) as usize,
        (PICKS_PER_MM as f64 * extents.height().ceil()) as usize,
    );

    let (xofs, yofs) = extents.min().x_y();

    let _stroke_width = (resolution.0 as f32 / extents.width() as f32) * 2.5;
    // TODO: This resolution needs to be scaled if we have rotated.
    let sx = resolution.0 as f32 / extents.width() as f32;
    let sy = resolution.1 as f32 / extents.height() as f32;
    let mut surface =
        surfaces::raster_n32_premul((resolution.0 as i32, resolution.1 as i32)).expect("surface");
    let mut paint = Paint::default();
    paint.set_color(Color::new(u32::MAX));
    paint.set_style(Style::Stroke);
    paint.set_anti_alias(false); // We want sharp edges and no "blurs"
    // paint.set_stroke_width(0.1);
    paint.set_stroke_cap(skia_safe::PaintCap::Round);
    paint.set_shader(None);
    let canvas = surface.canvas();
    canvas.clear(u32::MAX);
    // canvas.draw_circle((0., 0.), 25., &paint);
    canvas.translate((-xofs as f32 * sx, -yofs as f32 * sy));
    canvas.scale((sx, sy));
    let _mid = extents.center();
    paint.set_blend_mode(BlendMode::Src);
    for pg in &geo {
        let pen = pg.stroke.clone().unwrap_or(PenDetail::default());
        paint.set_stroke_width(pen.stroke_width as f32);
        paint.set_color(Color::new(pg.id as u32));

        if let Geometry::MultiLineString(mls) = &pg.geometry {
            //&pg.geometry {
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
    let mut pixels = bmap.peek_pixels().expect("Failed to peek pixel data.");
    let data = pixels.bytes_mut().expect("Failed to get back pixel data.");
    let u32_data = vec![0_u32; data.len() / 4];
    unsafe {
        let u8_ref = std::slice::from_raw_parts_mut((*data).as_mut_ptr(), data.len());
        std::ptr::copy(u8_ref.as_ptr(), u32_data.as_ptr() as *mut u8, data.len());
    }
    Ok((u32_data, extents))
}
