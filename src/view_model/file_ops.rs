use crate::core::commands::ViewCommand;

use super::{BAPViewModel, FileDialog, FileSelector};

use std::{path::PathBuf, sync::mpsc, thread::spawn};

impl BAPViewModel {
    pub fn load_machine_with_dialog(&mut self) {
        let (tx, rx) = mpsc::channel::<FileSelector>();
        self.file_selector = Some(rx);
        let the_path = self.config.config_dir.clone().join("machines");
        spawn(move || {
            let file = FileDialog::new()
                .add_filter("bap-machine", &["bam", "bap-machine"])
                .set_directory(the_path)
                .pick_file();
            if let Some(path) = file {
                tx.send(FileSelector::LoadMachineFrom(path.into()))
                    .expect("Failed to load machine from file");
            }
        });
    }

    pub fn save_machine_with_dialog(&mut self) {
        let (tx, rx) = mpsc::channel::<FileSelector>();
        self.file_selector = Some(rx);
        let the_path = self.config.config_dir.clone().join("machines");
        spawn(move || {
            let file = FileDialog::new()
                .add_filter("bap-machine", &["bam", "bap-machine"])
                .set_directory(the_path)
                .save_file();
            if let Some(path) = file {
                tx.send(FileSelector::SaveMachineAs(path.into()))
                    .expect("Failed to save machine to file");
            }
        });
    }

    pub fn load_pgf_with_dialog(&mut self) {
        let (tx, rx) = mpsc::channel::<FileSelector>();
        self.file_selector = Some(rx);
        spawn(move || {
            let file = FileDialog::new()
                .add_filter("pgf", &["pgf"])
                .set_directory("")
                .pick_file();
            if let Some(path) = file {
                tx.send(FileSelector::LoadPGF(path.into()))
                    .expect("Failed to send SVG import over MPSC.");
            }
        });
    }

    pub fn import_svg_with_dialog(&mut self) {
        let (tx, rx) = mpsc::channel::<FileSelector>();
        self.file_selector = Some(rx);
        spawn(move || {
            let file = FileDialog::new()
                .add_filter("svg", &["svg"])
                .add_filter("hpgl", &["hpgl"])
                .add_filter("wkt", &["wkt"])
                .set_directory("")
                .pick_file();
            if let Some(path) = file {
                tx.send(FileSelector::ImportSVG(path.into()))
                    .expect("Failed to send SVG import over MPSC.");
            }
        });
    }

    pub fn save_project_with_dialog(&mut self) {
        let (tx, rx) = mpsc::channel::<FileSelector>();
        self.file_selector = Some(rx);
        spawn(move || {
            let file = FileDialog::new()
                .add_filter("bap2", &["bap2"])
                .set_directory("")
                .save_file();
            if let Some(path) = file {
                tx.send(FileSelector::SaveProjectAs(path.into()))
                    .expect("Failed to e project");
            }
        });
    }

    pub fn open_project_with_dialog(&mut self) {
        let (tx, rx) = mpsc::channel::<FileSelector>();
        self.file_selector = Some(rx);
        spawn(move || {
            let file = FileDialog::new()
                .add_filter("bap2", &["bap2"])
                .set_directory("")
                .pick_file();
            if let Some(path) = file {
                tx.send(FileSelector::OpenProject(path.into()))
                    .expect("Failed to load project");
            }
        });
    }

    pub fn handle_file_selector(&mut self) {
        if let Some(msg_in) = &self.file_selector {
            match msg_in.try_recv() {
                Ok(path_selector) => {
                    match path_selector {
                        FileSelector::ImportSVG(path_buf) => {
                            self.yolo_view_command(ViewCommand::ImportSVG(path_buf))
                        }
                        FileSelector::OpenProject(path_buf) => {
                            self.yolo_view_command(ViewCommand::LoadProject(path_buf))
                        }
                        FileSelector::SaveProjectAs(path_buf) => {
                            self.yolo_view_command(ViewCommand::SaveProject(Some(path_buf)))
                        }
                        FileSelector::LoadPGF(path_buf) => {
                            self.yolo_view_command(ViewCommand::LoadPGF(path_buf))
                        }
                        FileSelector::SaveMachineAs(path_buf) => {
                            self.yolo_view_command(ViewCommand::SaveMachineConfig(path_buf))
                        }
                        FileSelector::LoadMachineFrom(path_buf) => {
                            self.yolo_view_command(ViewCommand::LoadMachineConfig(path_buf))
                        }
                    }
                    self.file_selector = None; // Delete it now that the command is done.
                }
                Err(_) => (),
            }
        }
    }

    pub fn save_project(&mut self, path: Option<PathBuf>) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::SaveProject(path))
                .expect("Failed to send SaveProject command?");
        }
    }

    #[allow(unused)]
    pub fn load_project(&mut self, path: PathBuf) {
        if let Some(cmd_out) = &self.cmd_out {
            cmd_out
                .send(ViewCommand::LoadProject(path))
                .expect("Failed to send Loadt command?");
        }
    }
}
