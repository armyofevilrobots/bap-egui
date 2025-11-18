use egui::{Align2, Key, Link, WidgetText, Window, vec2};

use crate::view_model::space_commands::{SPACE_CMDS, SpaceCommandBranch};
use crate::view_model::{BAPViewModel, CommandContext};

// Let's try this again from scratch
pub fn shortcut_panel(model: &mut BAPViewModel, ctx: &egui::Context) {
    if let CommandContext::Space(mut keys) = model.command_context.clone() {
        let mut coldata: Vec<Vec<(Key, Vec<Key>, String, bool)>> = Vec::new(); // Key, name, current?
        let mut tree = &*SPACE_CMDS.lock(); //.expect("Couldn't take over CMDS list");
        let pressed_keys = keys.clone();
        keys.reverse();
        let mut stack: Vec<Key> = Vec::new();
        loop {
            let key = keys.pop();
            let mut next: Option<&SpaceCommandBranch> = None;
            let subtree: Vec<(Key, Vec<Key>, String, bool)> = match tree {
                SpaceCommandBranch::Branch(cmds) => {
                    let ccmds = cmds;
                    ccmds
                        .iter()
                        .map(|(ckey, cmd)| {
                            let mut stackc = stack.clone();
                            stackc.push(ckey.clone());
                            if Some(*ckey) == key {
                                next = Some(&cmd.1);
                                (ckey.clone(), stackc.clone(), cmd.0.clone(), true)
                            } else {
                                (ckey.clone(), stackc.clone(), cmd.0.clone(), false)
                            }
                        })
                        .collect()
                }
                SpaceCommandBranch::Leaf(name, _) => {
                    vec![(Key::Pipe, stack.clone(), name.clone(), true)]
                }
                SpaceCommandBranch::Stub(name) => {
                    vec![(Key::Pipe, stack.clone(), name.clone(), true)]
                }
            };
            coldata.push(subtree);
            if key.is_none() || next.is_none() {
                break;
            } else if next.is_some() {
                tree = next.unwrap();
            }
            if let Some(key) = key {
                stack.push(key.clone());
            }
        }
        // println!("The output columns are: {:?}", coldata);

        Window::new("Space Commands")
            .anchor(Align2::CENTER_BOTTOM, vec2(0., -24.))
            .collapsible(false)
            .min_width(800.)
            .default_size([800., 128.])
            .title_bar(true)
            .min_size(vec2(800., 128.))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // println!("PRessed keys are: {:?}", pressed_keys);

                        if ui.link("<SPACE>").clicked() {
                            model.command_context = CommandContext::Space(Vec::new());
                        };

                        for (idx, key) in pressed_keys.clone().iter().enumerate() {
                            if ui.link(key.symbol_or_name()).clicked() {
                                model.command_context = CommandContext::Space(
                                    pressed_keys
                                        .clone()
                                        .iter()
                                        .take(idx + 1)
                                        .map(|x| x.clone())
                                        .collect(),
                                )
                            };
                        }
                    });
                    // ui.add_space(8.);
                    ui.separator();

                    // The columns for the keys.
                    ui.horizontal(|ui| {
                        for (idx, col) in coldata.iter().enumerate() {
                            ui.vertical(|ui| {
                                for (key, stack, name, selected) in col {
                                    let rt = WidgetText::from(format!(
                                        "{}-{}",
                                        key.symbol_or_name(),
                                        name
                                    ));
                                    let rt = if *selected { rt.strong() } else { rt };
                                    let response = ui.add_enabled(
                                        *selected || idx == (coldata.len() - 1),
                                        Link::new(rt),
                                    );
                                    if response.clicked() {
                                        model.command_context =
                                            CommandContext::Space(stack.clone());
                                        // println!("STACK: {:?}", stack);
                                    }
                                }
                            });
                        }
                    });
                });
            });
    } //End of commandcontext is Space Command
}
