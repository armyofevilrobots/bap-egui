use anyhow::Result;
use dirs::config_dir;
use rand;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{fs::create_dir_all, io::Read, path::PathBuf};

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug, Default)]
pub enum RulerOrigin {
    #[default]
    Origin,
    Source,
}

impl RulerOrigin {
    pub fn toggle(&self) -> Self {
        match self {
            RulerOrigin::Origin => RulerOrigin::Source,
            RulerOrigin::Source => RulerOrigin::Origin,
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug, Default)]
pub enum DockPosition {
    #[default]
    Left,
    Right,
    Floating(f32, f32),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ImportOptions {
    pub import_pgf_pens: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PostOptions {
    pub reorder_by_tool: bool,
}

impl Default for PostOptions {
    fn default() -> Self {
        Self {
            reorder_by_tool: true,
        }
    }
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            import_pgf_pens: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct UIConfig {
    #[serde(default)]
    pub tool_dock_position: DockPosition,
    #[serde(default)]
    pub ruler_origin: RulerOrigin,
    #[serde(default)]
    pub show_paper: bool,
    #[serde(default)]
    pub show_limits: bool,
    #[serde(default)]
    pub show_extents: bool,
    #[serde(default)]
    pub show_rulers: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            tool_dock_position: DockPosition::Left,
            ruler_origin: RulerOrigin::Origin,
            show_paper: true,
            show_limits: true,
            show_extents: true,
            show_rulers: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub config_dir: PathBuf,
    #[serde(default)]
    pub ui_config: UIConfig,
    #[serde(default)]
    pub import_options: ImportOptions,
    #[serde(default)]
    pub post_options: PostOptions,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_dir: config_dir()
                .expect("Failed to determine home dir automatically!")
                .join("bot-a-plot"),
            ui_config: Default::default(),
            import_options: Default::default(),
            post_options: Default::default(),
        }
    }
}

impl AppConfig {
    pub fn load_from(from: Option<PathBuf>) -> Result<Self> {
        let from = match from {
            Some(path) => path,
            None => AppConfig::default().config_dir,
        };
        let from = from.join("config.bap");
        let mut reader = std::fs::File::open(from.clone())?;
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        let app_config = ron::from_str(data.as_str())?;
        eprintln!("Read config from file: {:?}", from);
        Ok(app_config)
    }

    pub fn save_to(&self, dest: Option<PathBuf>) -> Result<()> {
        let path = match dest {
            Some(path) => path,
            None => self.config_dir.clone(),
        };
        let path = path.join("config.bap");
        println!("Final save dest is {:?}", path);
        let tmp_path = path.with_added_extension(format!("tmp-bap-{}", rand::random::<u64>()));
        println!("Tmp save dest is {:?}", tmp_path);
        // let content = self.to_string();
        let writer = std::fs::File::create(tmp_path.clone())?;
        // ron::ser::to_io_writer_pretty(writer, self, PrettyConfig::default())?;
        // ron::Options::default().to_io_writer_pretty(writer, &self, PrettyConfig::default())?;
        ron::Options::default().to_io_writer_pretty(writer, &self, PrettyConfig::default())?;
        std::fs::rename(&tmp_path, &path)?;
        println!("Renamed to {:?}", path);
        Ok(())
    }

    pub fn preflight(dest: Option<PathBuf>) -> Result<()> {
        eprintln!("Preflight...");
        let path = match dest {
            Some(path) => path,
            None => Self::default().config_dir,
        };
        eprintln!("Preflight save-to path is {:?}", path);
        if !path.is_dir() {
            eprintln!("Creating new config path at {:?}", path);
            create_dir_all(path.clone())?;
        }
        eprintln!("Path exists and is a dir.");
        let cfgpath = path.join("config.bap"); //.with_extension("bap");
        eprintln!("Will save to {:?}", cfgpath);
        if !cfgpath.is_file() {
            eprintln!("No config. Creating default.");
            let cfg = AppConfig {
                config_dir: path.clone(),
                ..Default::default()
            };
            cfg.save_to(None)?;
        } else {
            eprintln!("Config already exists.");
        };
        eprintln!("Done preflight.");
        Ok(())
    }
}
