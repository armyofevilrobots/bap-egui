use crate::view_model::BAPViewModel;
use catppuccin_egui::{FRAPPE, LATTE, MACCHIATO, MOCHA, set_style_theme};
use eframe::egui;
use egui::{ComboBox, Id, Layout};
// use std::collections::HashMap;
use indexmap::IndexMap;

use egui::Visuals;

pub mod egui_nord;

pub fn themes() -> IndexMap<String, Visuals> {
    let mut cat_latte = egui::Style::default();
    set_style_theme(&mut cat_latte, LATTE);
    let mut cat_mocha = egui::Style::default();
    set_style_theme(&mut cat_mocha, MOCHA);
    let mut cat_macchiato = egui::Style::default();
    set_style_theme(&mut cat_macchiato, MACCHIATO);
    let mut cat_frappe = egui::Style::default();
    set_style_theme(&mut cat_frappe, FRAPPE);

    IndexMap::from([
        ("Nord Dark".to_string(), egui_nord::visuals()),
        ("EGUI Light".to_string(), egui::style::Visuals::light()),
        ("Catppuccin Latte".to_string(), cat_latte.visuals),
        ("Catppuccin Mocha".to_string(), cat_mocha.visuals),
        ("Catppuccin Macchiato".to_string(), cat_macchiato.visuals),
        ("Catppuccin Frappe".to_string(), cat_frappe.visuals),
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

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Done").clicked() {
                model.cancel_command_context(false);
            }
        });
    });
}
