use eframe::egui;
use egui::{Button, Color32, Image, ImageSource, Response, Ui, Vec2};

pub(crate) fn tool_button<'a>(
    ui: &mut Ui,
    img_source: impl Into<ImageSource<'a>>,
    tooltip: Option<String>,
    enabled: bool,
) -> Response {
    let mut img = Image::new(img_source)
        .fit_to_exact_size(Vec2::new(24., 24.))
        .maintain_aspect_ratio(true);
    if !ui.visuals().dark_mode {
        img = img.tint(Color32::from_black_alpha(128));
    }
    let button = egui::Button::image(img)
        .min_size(Vec2::new(32., 32.))
        .corner_radius(5.);

    let response = ui.add_enabled(enabled, button);

    if let Some(text) = tooltip {
        response.on_hover_text(text)
    } else {
        response
    }
}

pub(crate) fn toggle_button<'a>(
    ui: &mut Ui,
    value: &mut bool,
    img_source: impl Into<ImageSource<'a>>,
    tooltip: Option<String>,
    enabled: bool,
) -> Response {
    let mut img = Image::new(img_source)
        .fit_to_exact_size(Vec2::new(16., 16.))
        .maintain_aspect_ratio(true);
    if !ui.visuals().dark_mode {
        img = img.tint(Color32::from_black_alpha(128));
    }
    let button = Button::selectable(*value, img)
        .frame_when_inactive(true)
        .min_size(Vec2::new(20., 20.))
        .corner_radius(2.);

    let response = ui.add_enabled(enabled, button);
    if response.clicked() {
        *value = !*value;
    }
    if let Some(text) = tooltip {
        response.on_hover_text(text)
    } else {
        response
    }
}
