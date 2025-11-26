use egui::Pos2;

use crate::core::commands::ViewCommand;

use super::BAPViewModel;

impl BAPViewModel {
    pub fn set_picked(&mut self, picked: Option<Vec<usize>>) {
        self.picked = picked;
    }

    pub fn picked(&self) -> Option<Vec<usize>> {
        self.picked.clone()
    }

    pub fn pick_clear(&self) {
        self.yolo_view_command(ViewCommand::ClearPick);
    }

    pub fn pick_at_point(&self, point: Pos2) {
        self.yolo_view_command(ViewCommand::TryPickAt(point.x as f64, point.y as f64));
    }

    pub fn toggle_pick_at_point(&self, point: Pos2) {
        self.yolo_view_command(ViewCommand::TogglePickAt(point.x as f64, point.y as f64));
    }

    pub fn add_pick_at_point(&self, point: Pos2) {
        self.yolo_view_command(ViewCommand::AddPickAt(point.x as f64, point.y as f64));
    }

    pub fn pick_all(&self) {
        self.yolo_view_command(ViewCommand::SelectAll);
    }

    #[allow(unused)]
    pub fn select_by_color_pick(&self, point: Pos2) {
        self.yolo_view_command(ViewCommand::SelectByColorPick(
            point.x as f64,
            point.y as f64,
        ))
    }
}
