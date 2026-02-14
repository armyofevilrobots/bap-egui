use std::fmt::{Debug, Display};

use super::space_commands::{SPACE_CMDS, SpaceCommandBranch};
use aoer_plotty_rs::plotter::pen::PenDetail;
use eframe::egui;
use egui::{Key, Pos2};
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::{
    core::{commands::MatTarget, config::AppConfig, machine::MachineConfig},
    view_model::BAPViewModel,
};
// use crate::view_model::project_ops::project_ops;

#[derive(PartialEq, Clone, Debug)]
pub enum CommandContext {
    Origin,
    PaperChooser,
    MachineEdit(Option<MachineConfig>),
    PenCrib,
    PenEdit(usize, PenDetail), // The pen index in Vec<Pens>
    PenDelete(usize),          // Delete the pen at IDX via modal confirmation
    #[allow(unused)]
    Clip(Option<Pos2>, Option<Pos2>),
    Rotate(Option<Pos2>, Option<Pos2>, Option<Pos2>), // center, reference, angle
    Scale(f64),
    Space(Vec<Key>),
    SelectColorAt(Option<Pos2>),
    SelectTheme,
    Translate(Option<Pos2>),
    ScaleAround(Option<Pos2>, Option<Pos2>), // Center, reference
    EditGcode(Option<String>),               // Saves original gcode.
    Configure(Option<AppConfig>),
    MatToTarget(MatTarget),
    None,
}

impl Display for CommandContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandContext::Origin => write!(f, "Origin"),
            CommandContext::PaperChooser => write!(f, "PaperChooser"),
            CommandContext::MachineEdit(arg0) => f.debug_tuple("MachineEdit").field(arg0).finish(),
            CommandContext::PenCrib => write!(f, "PenCrib"),
            CommandContext::PenEdit(arg0, arg1) => {
                f.debug_tuple("PenEdit").field(arg0).field(arg1).finish()
            }
            CommandContext::PenDelete(arg0) => f.debug_tuple("PenDelete").field(arg0).finish(),
            CommandContext::Clip(arg0, arg1) => {
                f.debug_tuple("Clip").field(arg0).field(arg1).finish()
            }
            CommandContext::Rotate(arg0, arg1, arg2) => f
                .debug_tuple("Rotate")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            CommandContext::Scale(arg0) => f.debug_tuple("Scale").field(arg0).finish(),
            CommandContext::Space(keys) => write!(
                f,
                "{}",
                keys.iter()
                    .map(|k| format!("[{}]", k.symbol_or_name()))
                    .collect::<Vec<String>>()
                    .join(">")
            ),
            CommandContext::None => write!(f, "None"),
            CommandContext::SelectColorAt(pos2) => write!(
                f,
                "SelectColorAt({})",
                match pos2 {
                    Some(Pos2 { x, y }) => format!("{},{}", x, y),
                    None => "...".to_string(),
                }
            ),
            CommandContext::SelectTheme => write!(f, "Select Theme"),
            CommandContext::Translate(_start) => write!(f, "Translate"),
            CommandContext::ScaleAround(xy, _opt_ref) => {
                write!(f, "ScaleAround({:?})", xy)
            }
            CommandContext::EditGcode(_) => write!(f, "Edit GCode"),
            CommandContext::Configure(_) => write!(f, "Configuration"),
            CommandContext::MatToTarget(mat_target) => write!(f, "Arrange matted: {}", mat_target),
        }
    }
}

pub enum SpaceCommandStatus {
    Ongoing,
    Invalid,
    Dispatched(String),
}

impl CommandContext {
    /// Validates that a given space command is either done or
    /// viable to continue, and dispatches if possible.
    /// Returns a bool to tell the parent to either
    pub fn dispatch_space_cmd(model: &mut BAPViewModel, keys: &Vec<Key>) -> SpaceCommandStatus {
        // println!("Dispatching command: {}", model.command_context);
        let mut tree = &*SPACE_CMDS.lock(); //.expect("Failed to lock space commands!");
        let _allkeys = keys.clone();
        let mut keys = keys.clone();
        let mut cmd_display = String::new();
        keys.reverse();
        loop {
            let key = keys.pop();
            let mut next: Option<&SpaceCommandBranch> = None;
            match tree {
                SpaceCommandBranch::Branch(cmds) => {
                    let ccmds = cmds;
                    ccmds.iter().for_each(|(ckey, cmd)| {
                        if Some(*ckey) == key {
                            cmd_display = cmd_display.clone()
                                + format!("{}[{}] ", ckey.symbol_or_name(), cmd.0).as_str();
                            next = Some(&cmd.1);
                        }
                    });
                }
                SpaceCommandBranch::Leaf(_name, cmd, _opt_enabled_fn) => {
                    cmd(model);
                    return SpaceCommandStatus::Dispatched(cmd_display);
                }
                SpaceCommandBranch::Stub(_name) => {
                    break;
                }
            };
            // println!("KEY: {:?} NEXT: {:?}", key, next);
            if key.is_some() && next.is_none() {
                return SpaceCommandStatus::Invalid;
            } else if next.is_none() {
                return SpaceCommandStatus::Ongoing;
            } else if next.is_some() {
                tree = next.unwrap();
            }
        }
        return SpaceCommandStatus::Ongoing;
    }
}

impl BAPViewModel {
    pub fn cancel_command_context(&mut self, was_cancel: bool) {
        if was_cancel {
            if self.command_context != CommandContext::None {
                self.queued_toasts.push_back(Toast {
                    kind: ToastKind::Info,
                    text: format!("Exited command context {}", self.command_context).into(),
                    options: ToastOptions::default()
                        .duration_in_seconds(3.0)
                        .show_progress(true),
                    ..Default::default()
                });
            }
        };

        self.command_context = match &self.command_context {
            CommandContext::Origin => CommandContext::None,
            CommandContext::PaperChooser => CommandContext::None,
            CommandContext::MachineEdit(machine_config) => {
                if was_cancel {
                    self.machine_config = match machine_config {
                        Some(cfg) => cfg.clone(),
                        None => self.machine_config.clone(),
                    };
                };
                CommandContext::None
            }
            CommandContext::PenCrib => CommandContext::None,
            CommandContext::PenEdit(_, _pen_detail) => CommandContext::PenCrib,
            CommandContext::PenDelete(_) => CommandContext::PenCrib,
            CommandContext::Clip(_pos2, _pos3) => CommandContext::None,
            CommandContext::Rotate(pos2, pos3, pos4) => {
                if was_cancel {
                    if let Some(_p4) = pos4 {
                        CommandContext::Rotate(pos2.clone(), pos3.clone(), None)
                    } else if let Some(_p3) = pos3 {
                        CommandContext::Rotate(pos2.clone(), None, None)
                    } else {
                        CommandContext::None
                    }
                } else {
                    CommandContext::None
                }
            }
            CommandContext::Scale(_) => CommandContext::None,
            CommandContext::Space(items) => {
                if was_cancel {
                    if items.len() > 0 {
                        CommandContext::Space(Vec::from_iter(
                            items[0..(items.len() - 1)].iter().map(|i| i.clone()),
                        ))
                    } else {
                        CommandContext::None
                    }
                } else {
                    CommandContext::None
                }
            }
            CommandContext::SelectColorAt(_pos2) => CommandContext::None,
            CommandContext::None => CommandContext::None,
            CommandContext::SelectTheme => CommandContext::None,
            CommandContext::Translate(_) => CommandContext::None,
            CommandContext::ScaleAround(_pos2, _opt_ref) => CommandContext::None,
            CommandContext::EditGcode(opt_gcode) => {
                if was_cancel {
                    if let Some(gcode) = opt_gcode {
                        self.gcode = gcode.clone();
                    }
                }
                CommandContext::None
            }
            CommandContext::Configure(opt_app_config) => {
                if was_cancel {
                    if let Some(config) = opt_app_config {
                        self.config = config.clone();
                    }
                };
                CommandContext::None
            }
            CommandContext::MatToTarget(_mat_target) => CommandContext::None,
        };
    }

    pub fn set_command_context(&mut self, ctx: CommandContext) {
        self.command_context = match &self.command_context {
            CommandContext::Origin => ctx,
            CommandContext::PaperChooser => ctx,
            CommandContext::MachineEdit(_machine_config) => ctx,
            CommandContext::PenCrib => ctx,
            CommandContext::PenEdit(_idx, _pen_detail) => ctx,
            CommandContext::PenDelete(_idx) => ctx,
            CommandContext::Clip(_pos2, _pos3) => ctx,
            CommandContext::Rotate(_pos2, _pos3, _pos4) => ctx,
            CommandContext::Scale(_scale) => ctx,
            CommandContext::Space(_items) => ctx,
            CommandContext::SelectColorAt(_opt_pos2) => ctx,
            CommandContext::None => ctx,
            CommandContext::SelectTheme => ctx,
            CommandContext::Translate(_) => ctx,
            CommandContext::ScaleAround(_pos2, _opt_ref) => ctx,
            CommandContext::EditGcode(_) => ctx,
            CommandContext::Configure(_app_config) => ctx,
            CommandContext::MatToTarget(_mat_target) => ctx,
        };
    }

    pub fn command_context(&self) -> CommandContext {
        self.command_context.clone()
    }

    pub fn command_context_mut(&mut self) -> &mut CommandContext {
        &mut self.command_context
    }
}
