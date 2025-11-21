use std::fmt::{Debug, Display};

use super::space_commands::{SPACE_CMDS, SpaceCommandBranch};
use aoer_plotty_rs::plotter::pen::PenDetail;
use eframe::egui;
use egui::{Key, Pos2};

use crate::{core::machine::MachineConfig, view_model::BAPViewModel};
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
    None,
}

impl Display for CommandContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Origin => write!(f, "Origin"),
            Self::PaperChooser => write!(f, "PaperChooser"),
            Self::MachineEdit(arg0) => f.debug_tuple("MachineEdit").field(arg0).finish(),
            Self::PenCrib => write!(f, "PenCrib"),
            Self::PenEdit(arg0, arg1) => f.debug_tuple("PenEdit").field(arg0).field(arg1).finish(),
            Self::PenDelete(arg0) => f.debug_tuple("PenDelete").field(arg0).finish(),
            Self::Clip(arg0, arg1) => f.debug_tuple("Clip").field(arg0).field(arg1).finish(),
            Self::Rotate(arg0, arg1, arg2) => f
                .debug_tuple("Rotate")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Scale(arg0) => f.debug_tuple("Scale").field(arg0).finish(),
            Self::Space(keys) => write!(
                f,
                "{}",
                keys.iter()
                    .map(|k| format!("[{}]", k.symbol_or_name()))
                    .collect::<Vec<String>>()
                    .join(">")
            ),
            Self::None => write!(f, "None"),
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
                SpaceCommandBranch::Leaf(_name, cmd) => {
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
