use std::ffi::OsString;
use std::io::BufWriter;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fmt::Debug, fs::File, path::PathBuf};

use anyhow::Result as AnyResult;
use anyhow::anyhow;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum MachineVariant {
    GRBL,
    HPGL,
}

impl Default for MachineVariant {
    fn default() -> Self {
        MachineVariant::GRBL
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct MachineConfig {
    name: String,
    post_template: Box<Vec<(String, String)>>,
    skim: Option<f64>,
    keepdown: Option<f64>,
    limits: (f64, f64),
    feedrate: f64,
    #[serde(default)]
    variant: MachineVariant,
}

impl Debug for MachineConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MachineConfig")
            .field("name", &self.name)
            .field("skim", &self.skim)
            .field("keepdown", &self.keepdown)
            .field("limits", &self.limits)
            .field("feedrate", &self.feedrate)
            .finish()
    }
}

impl MachineConfig {
    pub fn save_to_path(&self, path: &PathBuf) -> AnyResult<PathBuf> {
        let mut path = path.clone(); //std::fs::canonicalize(path)?;
        let mut dest_path = path.clone();
        dest_path.set_extension(OsString::from_str("bap-machine")?);
        // We save, then move, to ensure we don't accidentally delete if something bad happens.
        let tmptime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        path.set_extension(OsString::from_str(
            format!("bap-machine.{tmptime}.tmp").as_str(),
        )?);

        // Write the file
        {
            let writer = std::fs::File::create(&path)?;
            let writer = Box::new(BufWriter::new(writer));
            //
            // Turns out writer_pretty is hella slow too.
            // Or... Actually it was the lack of a bufwriter. Wups.
            ron::Options::default().to_io_writer_pretty(writer, self, PrettyConfig::default())?;
        } // Falls out of scope, closes file, we hope.
        std::fs::rename(&path, &dest_path)?;

        Ok(dest_path)
    }

    pub fn load_from_path(path: &PathBuf) -> AnyResult<Self> {
        if let Ok(path) = std::fs::canonicalize(path) {
            let machine_rdr = std::fs::File::open(path.clone())?;
            return match ron::de::from_reader::<File, Self>(machine_rdr) {
                Ok(machine) => Ok(machine),
                Err(err) => {
                    eprintln!("Failed to load due to: {:?}", &err);
                    Err(anyhow!(format!("Error was: {:?}", &err)))
                }
            };
        };
        Err(anyhow!(format!("Invalid machine path {:?}", path)))
    }

    pub fn set_post_template(&mut self, post_template: &Vec<(String, String)>) {
        self.post_template = Box::new(post_template.clone())
    }

    pub fn get_post_template(&self) -> Vec<(String, String)> {
        self.post_template.as_ref().clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[allow(dead_code)]
    pub fn set_feedrate(&mut self, feedrate: f64) {
        self.feedrate = feedrate;
    }

    #[allow(dead_code)]
    pub fn set_limits(&mut self, limits: (f64, f64)) {
        self.limits = limits;
    }

    #[allow(dead_code)]
    pub fn set_keepdown(&mut self, keepdown: Option<f64>) {
        self.keepdown = keepdown
    }

    #[allow(dead_code)]
    pub fn set_skim(&mut self, skim: Option<f64>) {
        self.skim = skim
    }

    pub fn post_template(&self) -> AnyResult<Tera> {
        let mut tera = Tera::default();
        tera.add_raw_templates(self.post_template.clone().into_iter())?;
        Ok(tera)
    }

    pub fn skim(&self) -> Option<f64> {
        self.skim
    }

    pub fn keepdown(&self) -> Option<f64> {
        self.keepdown
    }

    pub fn limits(&self) -> (f64, f64) {
        self.limits.clone()
    }

    pub fn feedrate(&self) -> f64 {
        self.feedrate
    }

    pub fn bapv1() -> Self {
        let bap_top = 4.;
        let bap_bottom = 13.;
        let bap_pen_steps = 20;
        let bap_pen_delay = 20;
        let pen_downs: Vec<String> = (0..=bap_pen_steps)
            .map(|i| {
                let penpos =
                    bap_top + (bap_bottom - bap_top) * (f64::from(i) / f64::from(bap_pen_steps));
                format!("M280 S{}\nG4 P{}\n", penpos, bap_pen_delay + i)
            })
            .collect();
        let pen_downs = pen_downs.concat() + &format!("G4 P{}\n", bap_pen_delay * 6).to_string();

        let bap_post_template: Vec<(String, String)>=vec![
            (
                "prelude".into(),
                format!("M280 S{}\nG4 P150\nG28 X Y\nG90\nG92 X0 Y0 ; HOME", bap_top),
            ),
            (
                "epilog".into(),
                format!("M280 S{}\nG4 P150\nG0 X0 Y230\nM281 ; FINISHED", bap_top),
            ),
            (
                "penup".into(),
                format!("M400\nM280 S{}\nG4 P150\nM400\nM281 ; PENUP", bap_top),
            ),
            (
                "penup_skim".into(),
                "M400\nM280 S{{skim}}\nG4 P150\nM400 ; PENUP_SKIM".to_string(),
            ),
            ("pendrop".into(),
                format!("M280 S{}", bap_bottom)),

            ("pendown".into(), format!("M400\n{}", pen_downs)),
            ("pendown_skim".into(),
                format!(
                    "M400\nM280 S{}\nG4 P{}\nM400\nM280 S{}\nG4 P{}\nM400\nM280 S{}\nG4 P{}\nM281; PENDOWN SKIM",
                    bap_bottom + 2.,
                    bap_pen_delay,
                    bap_bottom + 1.,
                    bap_pen_delay,
                    bap_bottom,
                    bap_pen_delay * 8
                ),
                // (10..bap_bottom.round() as i32)
                //     .rev()
                //     .map(|i| format!("M400 S280 S{}\nG4 P{}\n", i, bap_pen_delay).to_string())
                //     .collect::<Vec<String>>()
                //     .join(""),
            ),
            (
                "moveto".into(),
                "G0 X{{xmm|round(precision=2)}} Y{{ymm|round(precision=2)}} ; NEW LINE START"
                    .to_string(),
            ),
            (
                "lineto".into(),
                "G01 F{{feedrate|round(precision=2)}} X{{xmm|round(precision=2)}} Y{{ymm|round(precision=2)}}".to_string(),
            ),
            // ("coords".into(),
            //     "X{{xmm|round(precision=2)}} Y{{ymm|round(precision=2)}}".to_string()),
            ("toolchange".into(),
                //"M600 ; Pause for change to tool {{tool_id}}".to_string()),
                format!("M280 S{}\nG0 X115 Y230\n$M06 T{{tool_id}}", &bap_top).to_string())
        ];
        // println!("Template is: {:?}", bap_post_template);
        Self {
            name: "BAPv1".into(),
            post_template: Box::new(bap_post_template.clone()),
            skim: Some(10.),
            keepdown: Some(0.0),
            limits: (235., 235.),
            feedrate: 1200.,
            variant: Default::default(),
        }
    }
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self::bapv1()
    }
}
