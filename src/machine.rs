use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MachineConfig {
    name: String,
    post_template: Box<Vec<(String, String)>>,
    skim: Option<f64>,
    keepdown: Option<f64>,
    limits: (f64, f64),
    feedrate: f64,
}

impl MachineConfig {
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
            ("pendown".into(), format!("M400\n{}", pen_downs)),
            (
                "pendown_skim".into(),
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
        ];
        Self {
            name: "BAPv1".into(),
            post_template: Box::new(bap_post_template.clone()),
            skim: Some(10.),
            keepdown: Some(0.5),
            limits: (235., 235.),
            feedrate: 1200.,
        }
    }
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self::bapv1()
    }
}
