use std::sync::Arc;
use std::thread;
use std::thread::spawn;

use eframe::egui;
use egui::{Color32, ColorImage, Rect, Vec2, include_image, pos2, vec2};

pub(crate) mod core;
pub(crate) mod machine;
pub(crate) mod sender;
pub(crate) mod ui;
pub(crate) mod view_model;

use crate::view_model::BAPViewModel;

use core::ApplicationCore;
use uuid::Uuid;

fn main() -> eframe::Result<()> {
    let logo_img_data =
        eframe::icon_data::from_png_bytes(include_bytes!("../resources/images/aoer_logo.png"))
            .expect("The icon data must be valid");

    let mut native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((600.0, 400.0))
            .with_min_inner_size(Vec2::new(600., 400.)),
        vsync: false,
        ..eframe::NativeOptions::default()
    };
    native_options.viewport.icon = Some(Arc::new(logo_img_data));

    eframe::run_native(
        BAPViewModel::name(),
        native_options,
        Box::new(|ctx| {
            let ctx_app = ctx.egui_ctx.clone();
            let (mut application, cmd_out, state_in) = ApplicationCore::new(ctx_app);
            let mut model = BAPViewModel::default();
            model.state_in = Some(state_in);
            model.cmd_out = Some(cmd_out);
            model.origin = pos2(0., 279.);

            // We need some kind of placeholder due to the API. How bout a secret pixel?
            let tmp_svg_image = ColorImage::filled([3, 3], Color32::TRANSPARENT);
            let tex = ctx.egui_ctx.load_texture(
                format!("{}", Uuid::new_v4().as_u128()),
                tmp_svg_image,
                egui::TextureOptions::NEAREST,
            );
            model.source_image_handle = Some(Box::new(tex));
            model.source_image_extents = None;

            let handle = thread::spawn(move || application.run());
            model.set_join_handle(handle);
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            Ok(Box::new(model))
        }),
    )
}
