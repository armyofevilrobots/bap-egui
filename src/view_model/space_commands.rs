use std::sync::LazyLock;

use egui::{Key, mutex::Mutex};
use indexmap::IndexMap;

use crate::{
    core::{commands::ViewCommand, project::Orientation},
    view_model::{BAPViewModel, CommandContext},
};

type SpaceCommandFn = Box<dyn Fn(&mut BAPViewModel)>;
type SCEnabledFn = Box<dyn Fn(&mut BAPViewModel) -> bool>;

pub enum SpaceCommandBranch {
    Branch(IndexMap<Key, (String, SpaceCommandBranch)>),
    Leaf(String, SpaceCommandFn, Option<SCEnabledFn>),
    Stub(String),
}
unsafe impl Send for SpaceCommandBranch {}

impl std::fmt::Debug for SpaceCommandBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Branch(branch_map) => f.debug_tuple("Branch").field(branch_map).finish(),
            Self::Leaf(leaf_name, _leaf_fn, _valid_fn) => {
                f.debug_tuple("Leaf").field(leaf_name).finish()
            }
            Self::Stub(stub_name) => f.debug_tuple("Stub").field(stub_name).finish(),
        }
    }
}

fn quit_fn(model: &mut BAPViewModel) {
    model.quit();
}

fn scb_separator() -> (Key, (String, SpaceCommandBranch)) {
    (
        Key::Minus,
        (
            "--------".to_string(),
            SpaceCommandBranch::Stub("--------".to_string()),
        ),
    )
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
                None,
            ),
        ),
    );

    let cmd_project_open = (
        Key::O,
        (
            "Open Project".to_string(),
            SpaceCommandBranch::Leaf(
                "Open Project".to_string(),
                Box::new(|model| model.open_project_with_dialog()),
                None,
            ),
        ),
    );

    let cmd_load_pgf = (
        Key::G,
        (
            "Load PGF".to_string(),
            SpaceCommandBranch::Leaf(
                "Load PGF".to_string(),
                Box::new(|model| model.load_pgf_with_dialog()),
                None,
            ),
        ),
    );

    let cmd_import_svg = (
        Key::V,
        (
            "Import SVG".to_string(),
            SpaceCommandBranch::Leaf(
                "Import SVG".to_string(),
                Box::new(|model| model.import_svg_with_dialog()),
                None,
            ),
        ),
    );

    let cmd_project_saveas = (
        Key::A,
        (
            "Save Project As".to_string(),
            SpaceCommandBranch::Leaf(
                "Save Project As".to_string(),
                Box::new(|model| model.save_project_with_dialog()),
                None,
            ),
        ),
    );

    let cmd_project_save = (
        Key::S,
        (
            "Save Project".to_string(),
            SpaceCommandBranch::Leaf(
                "Save Project".to_string(),
                Box::new(|model| model.save_project(None)),
                Some(Box::new(|model| model.file_path.is_some())),
            ),
        ),
    );

    let cmd_project_new = (
        Key::N,
        (
            "New Project".to_string(),
            SpaceCommandBranch::Leaf(
                "New Project".to_string(),
                Box::new(|model| model.yolo_view_command(ViewCommand::ResetProject)),
                None,
            ),
        ),
    );

    let _cmd_file_project = (
        Key::P,
        (
            "Project".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                //cmd_project_open,
                //cmd_project_save,
                //cmd_project_saveas,
            ])),
        ),
    );

    let cmd_file = (
        Key::F,
        (
            "File".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                // cmd_file_project,
                cmd_project_new,
                cmd_project_open,
                cmd_project_save,
                cmd_project_saveas,
                scb_separator(),
                cmd_load_pgf,
                cmd_import_svg,
                scb_separator(),
                cmd_quit,
            ])),
        ),
    );

    let cmd_zoom_all = (
        Key::F,
        (
            "Zoom Fit".to_string(),
            SpaceCommandBranch::Leaf(
                "Zoom Fit".to_string(),
                Box::new(|model| model.zoom_fit()),
                None,
            ),
        ),
    );

    let cmd_view_rulers = (
        Key::R,
        (
            "Toggle Rulers".to_string(),
            SpaceCommandBranch::Leaf(
                "Toggle Rulers".to_string(),
                Box::new(|model| model.show_rulers = !model.show_rulers),
                None,
            ),
        ),
    );

    let cmd_view_zero_rulers = (
        Key::Z,
        (
            "Zero rulers to...".to_string(),
            SpaceCommandBranch::Leaf(
                "Zero rulers to...".to_string(),
                Box::new(|model| model.ruler_origin = model.ruler_origin.toggle()),
                None,
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
                None,
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
                None,
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
                None,
            ),
        ),
    );

    let cmd_select_theme = (
        Key::T,
        (
            "Select Theme".to_string(),
            SpaceCommandBranch::Leaf(
                "Select Theme".to_string(),
                Box::new(|model| {
                    model.set_command_context(CommandContext::SelectTheme);
                }),
                None,
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
                cmd_view_zero_rulers,
                cmd_view_extents,
                cmd_view_machine,
                cmd_view_paper,
                scb_separator(),
                cmd_select_theme, // cmd_toggle_dark_light,
            ])),
        ),
    );

    let cmd_smart_arrange = (
        Key::S,
        (
            "Smart Arrange".to_string(),
            SpaceCommandBranch::Leaf(
                "Smart Arrange".to_string(),
                Box::new(|model| model.center_smart()),
                None,
            ),
        ),
    );

    let cmd_arrange_paper = (
        Key::P,
        (
            "Align to paper".to_string(),
            SpaceCommandBranch::Leaf(
                "Align to paper".to_string(),
                Box::new(|model| model.center_paper()),
                None,
            ),
        ),
    );

    let cmd_arrange_machine = (
        Key::M,
        (
            "Align to machine".to_string(),
            SpaceCommandBranch::Leaf(
                "Align to machine".to_string(),
                Box::new(|model| model.center_machine()),
                None,
            ),
        ),
    );

    let cmd_set_origin = (
        Key::O,
        (
            "Set Origin".to_string(),
            SpaceCommandBranch::Leaf(
                "Set Origin".to_string(),
                Box::new(|model| model.command_context = CommandContext::Origin),
                None,
            ),
        ),
    );

    let cmd_project_undo = (
        Key::U,
        (
            "Undo".to_string(),
            SpaceCommandBranch::Leaf(
                "Undo".to_string(),
                Box::new(|model| model.undo()),
                Some(Box::new(|model| model.undo_available.clone())),
            ),
        ),
    );

    let cmd_scale_factor = (
        Key::F,
        (
            "Scale by factor".to_string(),
            SpaceCommandBranch::Leaf(
                "Scale by factor".to_string(),
                Box::new(|model| model.command_context = CommandContext::Scale(1.)),
                None,
            ),
        ),
    );

    let cmd_rotate = (
        Key::R,
        (
            "Rotate".to_string(),
            SpaceCommandBranch::Leaf(
                "Rotate".to_string(),
                Box::new(|model| model.command_context = CommandContext::Rotate(None, None, None)),
                None,
            ),
        ),
    );

    let cmd_scale = (
        Key::S,
        (
            "Scale".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([cmd_scale_factor])),
        ),
    );

    let cmd_arrange = (
        Key::A,
        (
            "Arrange".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                cmd_smart_arrange,
                cmd_arrange_machine,
                cmd_arrange_paper,
                scb_separator(),
                cmd_set_origin,
                scb_separator(),
            ])),
        ),
    );

    let cmd_media_edit_paper = (
        Key::P,
        (
            "Edit Paper".to_string(),
            SpaceCommandBranch::Leaf(
                "Edit Paper".to_string(),
                Box::new(|model| model.set_command_context(CommandContext::PaperChooser)),
                None,
            ),
        ),
    );

    let cmd_media_edit_pencrib = (
        Key::C,
        (
            "Pen (C)rib".to_string(),
            SpaceCommandBranch::Leaf(
                "Pen (C)rib".to_string(),
                Box::new(|model| model.command_context = CommandContext::PenCrib),
                None,
            ),
        ),
    );

    let cmd_media_swap_orientation = (
        Key::O,
        (
            "Swap Paper Orientation".to_string(),
            SpaceCommandBranch::Leaf(
                "Swap Paper Orientation".to_string(),
                Box::new(|model| {
                    model.paper_orientation = match model.paper_orientation {
                        Orientation::Landscape => Orientation::Portrait,
                        Orientation::Portrait => Orientation::Landscape,
                    }
                }),
                None,
            ),
        ),
    );

    let cmd_project_post_to_plotter = (
        Key::P,
        (
            "Post to plotter".to_string(),
            SpaceCommandBranch::Leaf(
                "Post to plotter".to_string(),
                Box::new(|model| {
                    model.request_post();
                }),
                None,
            ),
        ),
    );

    let cmd_geometry_ungroup = (
        Key::U,
        (
            "Ungroup".to_string(),
            SpaceCommandBranch::Leaf(
                "Ungroup".to_string(),
                Box::new(|model| {
                    model.ungroup();
                }),
                Some(Box::new(|model| model.picked().is_some())),
            ),
        ),
    );

    let cmd_geometry_group = (
        Key::G,
        (
            "Group".to_string(),
            SpaceCommandBranch::Leaf(
                "Group".to_string(),
                Box::new(|model| {
                    model.merge_group();
                }),
                Some(Box::new(|model| model.picked().is_some())),
            ),
        ),
    );

    let cmd_geometry = (
        Key::G,
        (
            "Geometry".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                cmd_geometry_ungroup,
                cmd_geometry_group,
                scb_separator(),
                cmd_rotate,
                cmd_scale,
            ])),
        ),
    );

    let cmd_select_all = (
        Key::A,
        (
            "Select All".to_string(),
            SpaceCommandBranch::Leaf(
                "Select All".to_string(),
                Box::new(|model| {
                    model.pick_all();
                }),
                None,
            ),
        ),
    );

    let cmd_select_by_color = (
        Key::C,
        (
            "Select by Color".to_string(),
            SpaceCommandBranch::Leaf(
                "Select by Color".to_string(),
                Box::new(|model| {
                    // model.select_by_color_pick();
                    model.set_command_context(CommandContext::SelectColorAt(None))
                }),
                None,
            ),
        ),
    );

    let cmd_select = (
        Key::S,
        (
            "Select".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([cmd_select_all, cmd_select_by_color])),
        ),
    );

    let cmd_edit = (
        Key::E,
        (
            "Edit".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                cmd_project_undo,
                scb_separator(),
                cmd_select,
                cmd_geometry,
                // cmd_scale,
                // cmd_rotate,
            ])),
        ),
    );

    let cmd_media = (
        Key::M,
        (
            "Media".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([
                cmd_media_edit_paper,
                cmd_media_edit_pencrib,
                cmd_media_swap_orientation,
                scb_separator(),
            ])),
        ),
    );

    let cmd_project = (
        Key::P,
        (
            "Project".to_string(),
            SpaceCommandBranch::Branch(IndexMap::from([cmd_project_post_to_plotter])),
        ),
    );

    Mutex::new(SpaceCommandBranch::Branch(IndexMap::from([
        cmd_file,
        cmd_edit,
        cmd_project,
        cmd_media,
        cmd_arrange,
        cmd_view,
    ])))
});
