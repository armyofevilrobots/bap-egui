use std::collections::VecDeque;

use aoer_plotty_rs::plotter::pen::PenDetail;
use egui::Modifiers;
use egui::{Color32, Pos2, pos2};

use crate::core::config::AppConfig;
use crate::core::machine::MachineConfig;
use crate::core::project::Orientation;
use crate::core::project::PaperSize;
use crate::core::sender::PlotterState;
use crate::view_model::CommandContext;
use crate::view_model::RulerOrigin;

use super::BAPDisplayMode;
use super::BAPViewModel;

impl Default for BAPViewModel {
    fn default() -> Self {
        Self {
            toolbar_position: super::DockPosition::Left,
            display_mode: BAPDisplayMode::SVG,
            state_in: None,
            cmd_out: None,
            status_msg: None,
            progress: None,
            file_selector: None,
            source_image_handle: None,
            source_image_extents: None,
            timeout_for_source_image: None,
            cancel_render: None,
            look_at: Pos2 { x: 0., y: 0. },
            view_zoom: 4.,
            command_context: CommandContext::None,
            paper_orientation: Orientation::Portrait,
            // paper_modal_open: false,
            // pen_crib_open: false,
            // TODO: This should just be a paper record.
            paper_size: PaperSize::Letter,
            origin: pos2(0., 0.),
            paper_color: Color32::WHITE,
            center_coords: pos2(0., 0.),
            machine_config: MachineConfig::default(),
            show_machine_limits: true,
            show_paper: true,
            show_rulers: true,
            show_extents: true,
            ppp: 1.5,
            dirty: false,
            container_rect: None,
            edit_cmd: String::new(),
            serial_ports: Vec::new(), //Just a default
            current_port: "".to_string(),
            join_handle: None,
            move_increment: 5.,
            plotter_state: PlotterState::Disconnected,
            queued_toasts: VecDeque::new(),
            pen_crib: vec![
                Default::default(),
                PenDetail {
                    tool_id: 2,
                    name: "Red Pen".to_string(),
                    stroke_width: 1.0,
                    stroke_density: 1.0,
                    feed_rate: Some(2000.0),
                    color: csscolorparser::Color::from_rgba8(255, 0, 0, 255),
                    ..Default::default()
                },
                PenDetail {
                    tool_id: 3,
                    name: "Blue Pen".to_string(),
                    stroke_width: 0.25,
                    stroke_density: 0.5, // It's runny
                    feed_rate: Some(1000.0),
                    color: csscolorparser::Color::from_rgba8(0, 0, 255, 255),
                    ..Default::default()
                },
            ],
            undo_available: false,
            file_path: None,
            ruler_origin: RulerOrigin::Source,
            last_pointer_pos: None,
            picked: None,
            config: AppConfig::default(),
            visuals: (
                "Nord Dark".to_string(),
                crate::ui::themes::egui_nord::visuals(),
            ),
            modifiers: Modifiers::NONE,
        }
    }
}
