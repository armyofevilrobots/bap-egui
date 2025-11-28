use super::BAPViewModel;
use crate::{
    core::{commands::ViewCommand, config::AppConfig},
    ui::themes::egui_nord::visuals,
};

impl BAPViewModel {
    pub fn update_ui_from_config(&mut self, config: AppConfig) {
        let (name, theme) = match crate::ui::themes::themes().get(&config.ui_config.theme) {
            Some(theme) => (config.ui_config.theme.clone(), theme.clone()),
            None => (
                crate::ui::themes::default_theme(),
                crate::ui::themes::themes()
                    .get(&crate::ui::themes::default_theme())
                    .unwrap()
                    .clone(),
            ),
        };
        self.toolbar_position = config.ui_config.tool_dock_position.clone();
        self.ruler_origin = config.ui_config.ruler_origin.clone();
        self.show_extents = config.ui_config.show_extents;
        self.show_rulers = config.ui_config.show_rulers;
        self.show_paper = config.ui_config.show_paper;
        self.show_machine_limits = config.ui_config.show_limits;
        self.visuals = (name, theme);
        self.config = config.clone();
    }
    /// This will send a new config package to the core and will
    /// pull that new config from the current UI state.
    pub fn update_core_config_from_changes(&mut self) {
        self.config.ui_config.tool_dock_position = self.toolbar_position.clone();
        self.config.ui_config.ruler_origin = self.ruler_origin.clone();
        self.config.ui_config.show_extents = self.show_extents;
        self.config.ui_config.show_rulers = self.show_rulers;
        self.config.ui_config.show_paper = self.show_paper;
        self.config.ui_config.show_limits = self.show_machine_limits;
        self.config.ui_config.theme = self.visuals().0.clone();
        self.yolo_view_command(ViewCommand::UpdateConfig(self.config.clone()));
    }
}
