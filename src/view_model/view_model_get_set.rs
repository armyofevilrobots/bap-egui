use std::{sync::mpsc::Sender, thread::JoinHandle};

use aoer_plotty_rs::plotter::pen::PenDetail;
use egui::{Modifiers, Pos2, Rect, TextureHandle, Visuals};
use egui_toast::Toast;

use crate::{
    core::{
        commands::ViewCommand,
        config::{AppConfig, DockPosition, RulerOrigin},
        machine::MachineConfig,
        sender::PlotterState,
    },
    view_model::BAPGeoLayer,
};

use super::{BAPDisplayMode, BAPViewModel};

impl BAPViewModel {
    pub fn name() -> &'static str {
        "Bot-a-Plot"
    }

    pub fn inhibit_space_command(&self) -> bool {
        self.inhibit_space_command
    }

    pub fn set_inhibit_space_command(&mut self, inhibit: bool) {
        self.inhibit_space_command = inhibit
    }

    pub fn show_layers(&self) -> bool {
        self.show_layers
    }

    pub fn set_show_layers(&mut self, show: bool) {
        self.show_layers = show
    }

    pub fn update_layer_name(&mut self, idx: usize, name: String) {
        self.yolo_view_command(ViewCommand::RenameLayer { id: idx, name });
    }

    pub fn geo_layers_mut(&mut self) -> &mut Vec<BAPGeoLayer> {
        &mut self.geo_layers
    }

    pub fn geo_layers(&self) -> &Vec<BAPGeoLayer> {
        &self.geo_layers
    }

    pub fn set_toolbar_width(&mut self, width: f32) {
        self.toolbar_width = width;
    }

    pub fn toolbar_width(&self) -> f32 {
        self.toolbar_width
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn gcode(&self) -> &String {
        &self.gcode
    }

    pub fn gcode_mut(&mut self) -> &mut String {
        &mut self.gcode
    }

    pub fn set_gcode(&mut self, gcode: String) {
        self.gcode = gcode;
    }

    pub fn modifiers(&self) -> egui::Modifiers {
        self.modifiers.clone()
    }

    pub fn set_modifiers(&mut self, modifiers: &Modifiers) {
        self.modifiers = modifiers.clone()
    }

    pub fn visuals(&self) -> (String, Visuals) {
        self.visuals.clone()
    }

    pub fn set_visuals(&mut self, viz_name: String, visuals: Visuals) {
        self.visuals = (viz_name, visuals);
    }

    pub fn set_origin(&mut self, origin: Pos2, create_history: bool) {
        if let Some(cmd_out) = &self.cmd_out {
            self.origin = origin;
            if create_history {
                cmd_out
                    .send(ViewCommand::SetOrigin(origin.x as f64, origin.y as f64))
                    .expect("Failed to send ORIGIN command?");
            }
        }
    }

    pub fn set_join_handle(&mut self, handle: JoinHandle<()>) {
        self.join_handle = Some(handle);
    }

    pub fn geo_layer_position(&self) -> DockPosition {
        self.geo_layer_position.clone()
    }

    pub fn set_geo_layer_position(&mut self, geo_layer_position: &DockPosition) {
        self.geo_layer_position = geo_layer_position.clone();
    }

    pub fn toolbar_position(&self) -> DockPosition {
        self.toolbar_position.clone()
    }

    pub fn set_toolbar_position(&mut self, toolbar_position: &DockPosition) {
        self.toolbar_position = toolbar_position.clone();
    }

    pub fn ruler_origin(&self) -> RulerOrigin {
        self.ruler_origin.clone()
    }

    pub fn set_ruler_origin(&mut self, origin: &RulerOrigin) {
        self.ruler_origin = origin.clone();
    }

    pub fn set_cancel_render(&mut self, cr: Sender<()>) {
        self.cancel_render = Some(cr);
    }

    #[allow(unused)]
    pub fn undo_available(&self) -> bool {
        self.undo_available.clone()
    }

    #[allow(unused)]
    pub fn set_pen_crib(&mut self, crib: Vec<PenDetail>) {
        self.pen_crib = crib;
    }

    pub fn pen_crib_mut(&mut self) -> &mut Vec<PenDetail> {
        &mut self.pen_crib
    }

    pub fn pen_crib(&self) -> Vec<PenDetail> {
        self.pen_crib.clone()
    }

    pub fn next_toast(&mut self) -> Option<Toast> {
        self.queued_toasts.pop_front()
    }

    pub fn plotter_state(&self) -> PlotterState {
        self.plotter_state.clone()
    }

    pub fn set_move_increment(&mut self, increment: f32) {
        self.move_increment = increment;
    }

    pub fn move_increment(&self) -> f32 {
        self.move_increment.clone()
    }

    pub fn current_port(&self) -> String {
        self.current_port.clone()
    }

    pub fn set_current_port(&mut self, port: String) {
        self.current_port = port;
    }

    pub fn serial_ports(&self) -> Vec<String> {
        self.serial_ports.clone()
    }

    #[allow(unused)]
    pub fn container_rect(&self) -> Option<Rect> {
        self.container_rect.clone()
    }

    pub fn set_container_rect(&mut self, rect: Rect) {
        self.container_rect = Some(rect);
    }

    pub fn set_edit_cmd(&mut self, cmd: String) {
        self.edit_cmd = cmd.clone();
    }

    pub fn edit_cmd(&self) -> String {
        self.edit_cmd.clone()
    }

    pub fn set_show_paper(&mut self, show_paper: bool) {
        self.show_paper = show_paper
    }

    pub fn set_show_rulers(&mut self, show_rulers: bool) {
        self.show_rulers = show_rulers
    }

    pub fn set_show_extents(&mut self, show_extents: bool) {
        self.show_extents = show_extents
    }

    pub fn set_show_machine_limits(&mut self, show_machine_limits: bool) {
        self.show_machine_limits = show_machine_limits
    }

    pub fn show_rulers(&self) -> bool {
        self.show_rulers.clone()
    }

    pub fn show_paper(&self) -> bool {
        self.show_paper.clone()
    }

    pub fn show_extents(&self) -> bool {
        self.show_extents.clone()
    }

    pub fn show_machine_limits(&self) -> bool {
        self.show_machine_limits.clone()
    }

    pub fn machine_config_mut(&mut self) -> &mut MachineConfig {
        &mut self.machine_config
    }

    pub fn machine_config(&self) -> MachineConfig {
        self.machine_config.clone()
    }

    pub fn set_center_coords(&mut self, coords: Pos2) {
        self.center_coords = coords.clone();
    }

    #[allow(unused)]
    pub fn center_coords(&self) -> Pos2 {
        self.center_coords.clone()
    }

    pub fn look_at(&self) -> Pos2 {
        self.look_at.clone()
    }

    pub fn set_look_at(&mut self, at: Pos2) {
        self.look_at = at;
    }

    pub fn origin(&self) -> Pos2 {
        self.origin.clone()
    }

    pub fn source_image_handle(&self) -> Option<Box<TextureHandle>> {
        self.source_image_handle.clone()
    }

    pub fn set_source_image_handle(&mut self, image_handle: Box<TextureHandle>) {
        self.source_image_handle = Some(image_handle);
    }

    pub fn progress(&self) -> Option<(String, usize)> {
        self.progress.clone()
    }

    pub fn source_image_extents(&self) -> Option<Rect> {
        self.source_image_extents.clone()
    }

    #[allow(unused)]
    pub fn set_status_msg(&mut self, msg: &Option<String>) {
        self.status_msg = msg.clone();
    }

    pub fn status_msg(&self) -> Option<String> {
        self.status_msg.clone()
    }

    pub fn zoom(&self) -> f64 {
        self.view_zoom
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.view_zoom = zoom.min(200.).max(1.);

        self.cancel_render();

        if let Some(_sender) = &self.cmd_out {
            // We know the extents of the svg, so we just need to
            // calculate a new image size for the current zoom level.
            self.request_new_source_image();
        }
    }

    pub fn display_mode(&self) -> BAPDisplayMode {
        self.display_mode.clone()
    }

    pub fn set_display_mode(&mut self, mode: BAPDisplayMode) {
        self.dirty = true;
        self.display_mode = mode;
    }
    #[allow(unused)]
    pub fn set_ppp(&mut self, ppp: f32) {
        self.ppp = ppp;
        // TODO: Reload the svg preview.
    }

    pub fn ppp(&self) -> f32 {
        self.ppp
    }
}
