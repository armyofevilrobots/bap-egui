use anyhow::{Result, anyhow};
use aoer_plotty_rs::plotter::pen::PenDetail;
use egui::{Color32, ColorImage};
use geo::{BoundingRect, Coord};
use skia_safe::paint::Style;
use skia_safe::{AlphaType, Bitmap, Color, ImageInfo, Paint, Path, surfaces};

use crate::core::project::BAPGeometry;

pub(crate) fn render_layer_preview(
    geo: &BAPGeometry,
    pen: &PenDetail,
    resolution: &[usize; 2],
) -> Result<ColorImage> {
    let lines = geo.lines();
    let cimg = ColorImage::filled(resolution.clone(), Color32::TRANSPARENT);
    if let Some(extents) = lines.bounding_rect() {
        let scale =
            (resolution[0] as f64 / extents.width()).min(resolution[1] as f64 / extents.width());
        // TODO: Calculate additional offset for non-square images.
        let middle = Coord {
            x: (extents.width() / 2.) + extents.min().x,
            y: extents.height() / 2. + extents.min().y,
        };
        let ofs = Coord {
            x: resolution[0] as f64 / 2. - (middle.x * scale),
            y: resolution[1] as f64 / 2. - (middle.y * scale),
        };

        let mut surface = surfaces::raster_n32_premul((resolution[0] as i32, resolution[1] as i32))
            .ok_or_else(|| anyhow!("Couldn't create raster surface for preview."))?;
        let mut paint = Paint::default();
        paint.set_color(Color::BLUE);
        paint.set_style(Style::Stroke);
        paint.set_anti_alias(true);
        paint.set_stroke_width(1.);
        paint.set_shader(None);
        let canvas = surface.canvas();
        // canvas.draw_circle((0., 0.), 25., &paint);
        // canvas.translate((-ofs.x as f32 * scale as f32, -ofs.y as f32 * scale as f32));
        canvas.translate((ofs.x as f32, ofs.y as f32));
        canvas.scale((scale as f32, scale as f32));
        // let pen = pg.stroke.clone().unwrap_or(PenDetail::default());
        paint.set_path_effect(None);
        paint.set_stroke_cap(skia_safe::PaintCap::Round);
        paint.set_stroke_width(pen.stroke_width as f32 * 5.);
        paint.set_alpha_f(pen.stroke_density as f32);
        // let color_code = pen.color.clone(); //csscolorparser::parse(pen.color.as_str()).unwrap_or_default();
        let [r, g, b, a] = pen.color.to_rgba8();
        paint.set_color(Color::from_argb(a, r, g, b));
        let mut path = Path::new();

        for (_line_idx, line) in lines.iter().enumerate() {
            if let Some(p0) = line.0.first() {
                path.move_to((p0.x as f32, p0.y as f32));
                for coord in line.0.iter().skip(1) {
                    path.line_to((coord.x as f32, coord.y as f32));
                }
            }
        }
        surface.canvas().draw_path(&path, &paint);
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

        // let image = surface.image_snapshot();
        // let mut context = surface.direct_context();
        // let imgout = image
        //     .encode(context.as_mut(), EncodedImageFormat::PNG, None)
        //     .unwrap();
        // let mut file = File::create(format!("layer-preview-{}.png", TIMESTAMP)).unwrap();
        // let bytes = imgout.as_bytes();
        // file.write_all(bytes).unwrap();

        bmap.alloc_pixels();
        let _result = surface.read_pixels_to_bitmap(&bmap, (0, 0));
        let pixels = bmap
            .peek_pixels()
            .ok_or_else(|| anyhow!("Couldn't peek pixels"))?;
        let cimg: ColorImage = ColorImage::from_rgba_premultiplied(
            [pixels.width() as usize, pixels.height() as usize],
            pixels
                .bytes()
                .ok_or_else(|| anyhow!("Failed to get pixels.bytes()"))?,
        );
        Ok(cimg)
    } else {
        // No geometry. Just a blank image.
        Ok(cimg)
    }
}
