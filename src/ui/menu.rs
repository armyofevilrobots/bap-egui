use crate::BAPViewModel;
use crate::view_model::CommandContext;
use crate::view_model::space_commands::{SPACE_CMDS, SpaceCommandBranch};
use eframe::egui;
use egui::{Button, Key, Rect, Ui};

fn grow_stack(stack: &Vec<Key>, key: &Key) -> Vec<Key> {
    let mut new_stack = stack.clone();
    new_stack.push(key.clone());
    new_stack
}

fn format_name_and_key_as_menu_string(
    name: &str,
    stack: &Vec<Key>,
    key: &Key,
    space_mode: bool,
) -> String {
    let stack = grow_stack(stack, key);
    let key_combo = stack
        .into_iter()
        .map(|k| k.symbol_or_name().to_string())
        .collect::<Vec<String>>()
        .join("-");

    let out = if space_mode {
        format!("{} [{}]", name, key_combo)
    } else {
        format!("{}", name)
    };
    // println!("Generated {} as key combo", out);
    out
}

fn menu_from_tree(
    model: &mut BAPViewModel,
    ui: &mut Ui,
    key: &Key,
    stack: &Vec<Key>,
    tree: &SpaceCommandBranch,
) {
    let space_mode: bool = if let CommandContext::Space(_) = model.command_context() {
        true
    } else {
        false
    };
    match tree {
        SpaceCommandBranch::Branch(cmd_map) => {
            for (key, (name, subtree)) in cmd_map.iter() {
                if let SpaceCommandBranch::Branch(_) = subtree {
                    ui.menu_button(
                        format_name_and_key_as_menu_string(name, stack, key, space_mode),
                        |ui| menu_from_tree(model, ui, key, &grow_stack(stack, key), subtree),
                    );
                } else {
                    menu_from_tree(model, ui, key, stack, subtree);
                }
            }
        }
        SpaceCommandBranch::Leaf(name, command_fn, opt_enabled_fn) => {
            let response = ui.add_enabled(
                match opt_enabled_fn {
                    Some(enfn) => enfn(model),
                    None => true,
                },
                Button::new(format_name_and_key_as_menu_string(
                    name, stack, key, space_mode,
                )),
            );
            if response.clicked() {
                command_fn(model)
            };
        }
        SpaceCommandBranch::Stub(_name) => {
            ui.label(_name);
        }
    }
}

pub(crate) fn main_menu(model: &mut BAPViewModel, ctx: &egui::Context) -> Rect {
    let tbp = egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        /*
            .frame(
                Frame::new()
                    .outer_margin(Margin {
                        left: 0,
                        right: 0,
                        top: 0,
                        bottom: -20,
                    })
                    .fill(ctx.style().visuals.window_fill.clone()),
            )
        */
        .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                menu_from_tree(
                    model,
                    ui,
                    &Key::Space,
                    &vec![Key::Space],
                    &*SPACE_CMDS.lock(),
                )
            })
        });
    tbp.response.rect
}
