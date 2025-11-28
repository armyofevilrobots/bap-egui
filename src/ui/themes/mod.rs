use crate::view_model::BAPViewModel;
use eframe::egui;
use egui::{ComboBox, Id};
use std::collections::HashMap;

use egui::Visuals;

pub mod egui_nord;

pub fn themes() -> HashMap<String, Visuals> {
    HashMap::from([
        ("Nord Dark".to_string(), egui_nord::visuals()),
        ("EGUI Light".to_string(), egui::style::Visuals::light()),
        (default_theme(), egui::style::Visuals::dark()),
    ])
}

pub fn default_theme() -> String {
    "EGUI Dark".to_string()
}

pub(crate) fn theme_window(model: &mut BAPViewModel, ctx: &egui::Context) {
    egui::Modal::new(Id::new("ThemeSelector")).show(ctx, |ui| {
        ui.set_width(250.);
        ui.heading("Select Theme");
        let mut theme_names: Vec<String> = themes().keys().map(|k| k.clone()).collect();
        theme_names.sort();
        let mut current_theme = model.visuals().0;
        ComboBox::from_label("")
            .selected_text(current_theme.clone())
            .show_ui(ui, |ui| {
                for theme_name in theme_names.clone() {
                    if ui
                        .selectable_value(
                            &mut current_theme,
                            theme_name.clone(),
                            theme_name.clone(),
                        )
                        .clicked()
                    {
                        model.set_visuals(
                            current_theme.clone(),
                            themes().get(&current_theme).unwrap().clone(),
                        );
                        model.update_core_config_from_changes();
                    };
                }
            });
    });
}
