use indexmap::IndexMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use eframe::egui;
use egui::{Key, Pos2};

use crate::view_model::BAPViewModel;
// use crate::view_model::project_ops::project_ops;

#[derive(PartialEq, Clone, Debug)]
pub enum CommandContext {
    Origin,
    PaperChooser,
    PenCrib,
    PenEdit(usize),   // The pen index in Vec<Pens>
    PenDelete(usize), // Delete the pen at IDX via modal confirmation
    #[allow(unused)]
    Clip(Option<Pos2>, Option<Pos2>),
    Rotate(Option<Pos2>, Option<Pos2>, Option<Pos2>), // center, reference, angle
    Scale(f64),
    Space(Vec<Key>),
    None,
}

pub enum SpaceCommandStatus {
    Ongoing,
    Invalid,
    Dispatched(String),
}

type SpaceCommandFn = Box<dyn Fn(&mut BAPViewModel)>;

pub enum SpaceCommandBranch {
    Branch(IndexMap<Key, (String, SpaceCommandBranch)>),
    Leaf(String, SpaceCommandFn),
    Stub(String),
}
unsafe impl Send for SpaceCommandBranch {}

impl std::fmt::Debug for SpaceCommandBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Branch(arg0) => f.debug_tuple("Branch").field(arg0).finish(),
            Self::Leaf(arg0, _arg1) => f.debug_tuple("Leaf").field(arg0).finish(),
            Self::Stub(arg0) => f.debug_tuple("Stub").field(arg0).finish(),
        }
    }
}

fn quit_fn(model: &mut BAPViewModel) {
    model.quit();
}

pub static SPACE_CMDS: LazyLock<Mutex<SpaceCommandBranch>> = LazyLock::new(|| {
    let cmd_quit = (
        Key::Q,
        (
            "Quit".to_string(),
            SpaceCommandBranch::Leaf(
                "Quit".to_string(),
                // Box::new(|model: &mut BAPViewModel| {}),
                Box::new(quit_fn),
            ),
        ),
    );

    let cmd_project_open = (
        Key::O,
        (
            "Open".to_string(),
            SpaceCommandBranch::Leaf(
                "Open".to_string(),
                Box::new(|model| model.open_project_with_dialog()),
            ),
        ),
    );

    let cmd_project_saveas = (
        Key::A,
        (
            "Save As".to_string(),
            SpaceCommandBranch::Leaf(
                "Save As".to_string(),
                Box::new(|model| model.save_project_with_dialog()),
            ),
        ),
    );

    let cmd_project_save = (
        Key::S,
        (
            "Save".to_string(),
            SpaceCommandBranch::Leaf(
                "Save".to_string(),
                Box::new(|model| model.save_project(None)),
            ),
        ),
    );

    let cmd_file_project = (
        Key::P,
        (
            "Project".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                cmd_project_open,
                cmd_project_save,
                cmd_project_saveas,
            ])),
        ),
    );

    let cmd_file = (
        Key::F,
        (
            "File".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([cmd_file_project])),
        ),
    );

    let cmd_zoom_all = (
        Key::F,
        (
            "Zoom Fit".to_string(),
            SpaceCommandBranch::Leaf("Zoom Fit".to_string(), Box::new(|model| model.zoom_fit())),
        ),
    );

    let cmd_view_rulers = (
        Key::R,
        (
            "Toggle Rulers".to_string(),
            SpaceCommandBranch::Leaf(
                "Toggle Rulers".to_string(),
                Box::new(|model| model.show_rulers = !model.show_rulers),
            ),
        ),
    );
    let cmd_view_extents = (
        Key::E,
        (
            "Toggle Extents".to_string(),
            SpaceCommandBranch::Leaf(
                "Toggle Extents".to_string(),
                Box::new(|model| model.show_extents = !model.show_extents),
            ),
        ),
    );
    let cmd_view_machine = (
        Key::M,
        (
            "Toggle Machine".to_string(),
            SpaceCommandBranch::Leaf(
                "Toggle Machine".to_string(),
                Box::new(|model| model.show_machine_limits = !model.show_machine_limits),
            ),
        ),
    );
    let cmd_view_paper = (
        Key::P,
        (
            "Toggle Paper".to_string(),
            SpaceCommandBranch::Leaf(
                "Toggle Paper".to_string(),
                Box::new(|model| model.show_paper = !model.show_paper),
            ),
        ),
    );

    let cmd_view = (
        Key::V,
        (
            "View".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                cmd_zoom_all,
                scb_separator(),
                cmd_view_rulers,
                cmd_view_extents,
                cmd_view_machine,
                cmd_view_paper,
            ])),
        ),
    );

    let cmd_project = (
        Key::P,
        (
            "Project".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([])),
        ),
    );

    let cmd_arrange = (
        Key::A,
        (
            "Arrange".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([])),
        ),
    );

    fn scb_separator() -> (Key, (String, SpaceCommandBranch)) {
        (
            Key::Minus,
            (
                "--------".to_string(),
                SpaceCommandBranch::Stub("--------".to_string()),
            ),
        )
    }

    Mutex::new(SpaceCommandBranch::Branch(IndexMap::from([
        cmd_file,
        cmd_project,
        cmd_view,
        cmd_arrange,
        scb_separator(),
        cmd_quit,
    ])))
});

impl CommandContext {
    /// Validates that a given space command is either done or
    /// viable to continue, and dispatches if possible.
    /// Returns a bool to tell the parent to either
    pub fn dispatch_space_cmd(model: &mut BAPViewModel, keys: &Vec<Key>) -> SpaceCommandStatus {
        let mut tree = &*SPACE_CMDS.lock().expect("Failed to lock space commands!");
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
                    /*
                    allkeys
                        .clone()
                        .iter()
                        .map(|&k| k.symbol_or_name())
                        .collect::<Vec<&str>>()
                        .join("-")
                        .to_string(),
                        */
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
