use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Orientation {
    Landscape,
    Portrait,
}
impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::Landscape => write!(f, "Landscape"),
            Orientation::Portrait => write!(f, "Portrait"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PaperSize {
    Letter,
    Ansi_A,
    Ansi_B,
    Ansi_C,
    Ansi_D,
    US_Legal,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    ISO216,
    USBusinessCard,
    EuroBusinessCard,
    Custom(f64, f64),
}

impl PaperSize {
    /// Get the mm measurements for this paper size, as a tuple of f64s.
    pub fn dims(&self) -> (f64, f64) {
        match self {
            PaperSize::Letter => (216., 279.),
            PaperSize::Ansi_A => (216., 279.),
            PaperSize::Ansi_B => (279., 432.),
            PaperSize::Ansi_C => (432., 559.),
            PaperSize::Ansi_D => (559., 864.),
            PaperSize::US_Legal => (216., 356.),
            PaperSize::A0 => (841., 1189.),
            PaperSize::A1 => (594., 841.),
            PaperSize::A2 => (420., 594.),
            PaperSize::A3 => (297., 420.),
            PaperSize::A4 => (210., 297.),
            PaperSize::A5 => (148., 210.),
            PaperSize::A6 => (105., 148.),
            PaperSize::A7 => (74., 105.),
            PaperSize::ISO216 => (74., 52.),
            PaperSize::USBusinessCard => (88.9, 50.8),
            PaperSize::EuroBusinessCard => (85., 55.),
            PaperSize::Custom(x, y) => (*x, *y),
        }
    }

    pub fn all() -> Vec<PaperSize> {
        vec![
            PaperSize::Letter,
            PaperSize::Ansi_A,
            PaperSize::Ansi_B,
            PaperSize::Ansi_C,
            PaperSize::Ansi_D,
            PaperSize::US_Legal,
            PaperSize::A0,
            PaperSize::A1,
            PaperSize::A2,
            PaperSize::A3,
            PaperSize::A4,
            PaperSize::A5,
            PaperSize::A6,
            PaperSize::A7,
            PaperSize::ISO216,
            PaperSize::USBusinessCard,
            PaperSize::EuroBusinessCard,
            PaperSize::Custom(200., 200.),
        ]
    }
}

impl Display for PaperSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmtarg = match self {
            Self::Letter => "US Letter".to_string(),
            Self::Ansi_A => "Ansi A".to_string(),
            Self::Ansi_B => "Ansi B".to_string(),
            Self::Ansi_C => "Ansi C".to_string(),
            Self::Ansi_D => "Ansi D".to_string(),
            Self::A0 => "A0".to_string(),
            Self::A1 => "A1".to_string(),
            Self::A2 => "A2".to_string(),
            Self::A3 => "A3".to_string(),
            Self::A4 => "A4".to_string(),
            Self::A5 => "A5".to_string(),
            Self::A6 => "A6".to_string(),
            Self::A7 => "A7".to_string(),
            Self::Custom(a, b) => format!("Custom({:.2}x{:.2})", a, b),
            Self::US_Legal => "US Legal".to_string(),
            PaperSize::ISO216 => "ISO216".to_string(),
            PaperSize::USBusinessCard => "US Business Card".to_string(),
            PaperSize::EuroBusinessCard => "Euro Business Card".to_string(),
        };
        write!(f, "{}", fmtarg)
    }
}

impl PaperSize {
    pub fn dimensions(&self) -> (f64, f64) {
        match self {
            PaperSize::Letter => (216., 279.),
            PaperSize::Ansi_A => (216., 279.),
            PaperSize::Ansi_B => (279., 432.),
            PaperSize::Ansi_C => (432., 559.),
            PaperSize::Ansi_D => (559., 864.),
            PaperSize::US_Legal => (216., 356.),
            PaperSize::A0 => (841., 1189.),
            PaperSize::A1 => (594., 841.),
            PaperSize::A2 => (420., 594.),
            PaperSize::A3 => (297., 420.),
            PaperSize::A4 => (210., 297.),
            PaperSize::A5 => (148., 210.),
            PaperSize::A6 => (105., 148.),
            PaperSize::A7 => (74., 105.),
            PaperSize::ISO216 => (74., 52.),
            PaperSize::USBusinessCard => (88.9, 50.8),
            PaperSize::EuroBusinessCard => (85., 55.),
            PaperSize::Custom(x, y) => (*x, *y),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Paper {
    pub weight_gsm: f64,
    pub rgb: (f64, f64, f64), // For display purposes only.
    pub size: PaperSize,
    pub orientation: Orientation,
}
