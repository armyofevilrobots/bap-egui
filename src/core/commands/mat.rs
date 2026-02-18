use serde::{Deserialize, Serialize};
use std::fmt::Display;
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum MatValues {
    Equal(f64),
    VertHoriz(f64, f64),
    TopRightBottomLeft(f64, f64, f64, f64),
}

impl MatValues {
    pub fn get_trbl(&self) -> (f64, f64, f64, f64) {
        match *self {
            MatValues::Equal(all) => (all, all, all, all),
            MatValues::VertHoriz(vert, horiz) => (vert, horiz, vert, horiz),
            MatValues::TopRightBottomLeft(t, r, b, l) => (t, r, b, l),
        }
    }
}

impl Display for MatValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatValues::Equal(value) => write!(f, "Equal({:3.1})", value),
            MatValues::VertHoriz(vert, horiz) => {
                write!(f, "Top/Bottom({:3.1}), Left/Right({:3.1})", vert, horiz)
            }
            MatValues::TopRightBottomLeft(top, right, bottom, left) => {
                write!(
                    f,
                    "Top:({:3.1}), Right:({:3.1}), Bottom:({:3.1}), Left:({:3.1}))",
                    top, right, bottom, left
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum MatTarget {
    Machine(MatValues),
    Paper(MatValues),
    Smart(MatValues),
}

impl MatTarget {
    pub fn values(&self) -> MatValues {
        match self {
            MatTarget::Machine(mat_values) => mat_values.clone(),
            MatTarget::Paper(mat_values) => mat_values.clone(),
            MatTarget::Smart(mat_values) => mat_values.clone(),
        }
    }

    pub fn get_trbl(&self) -> (f64, f64, f64, f64) {
        match self {
            MatTarget::Machine(mat_values) => mat_values.get_trbl(),
            MatTarget::Paper(mat_values) => mat_values.get_trbl(),
            MatTarget::Smart(mat_values) => mat_values.get_trbl(),
        }
    }

    pub fn options() -> Vec<MatTarget> {
        vec![
            MatTarget::Machine(MatValues::Equal(10.)),
            MatTarget::Paper(MatValues::Equal(10.)),
            MatTarget::Smart(MatValues::Equal(10.)),
        ]
    }
    pub fn options_with_values(values: &MatValues) -> Vec<MatTarget> {
        let (mtop, mright, mbottom, mleft) = match values.clone() {
            MatValues::Equal(all) => (all, all, all, all),
            MatValues::VertHoriz(vert, horiz) => (vert, horiz, vert, horiz),
            MatValues::TopRightBottomLeft(t, r, b, l) => (t, r, b, l),
        };
        let values_list = (
            MatValues::Equal(mtop),
            MatValues::VertHoriz(mtop, mright),
            MatValues::TopRightBottomLeft(mtop, mright, mbottom, mleft),
        );

        vec![
            MatTarget::Machine(values_list.0.clone()),
            MatTarget::Machine(values_list.1.clone()),
            MatTarget::Machine(values_list.2.clone()),
            MatTarget::Paper(values_list.0.clone()),
            MatTarget::Paper(values_list.1.clone()),
            MatTarget::Paper(values_list.2.clone()),
            MatTarget::Smart(values_list.0.clone()),
            MatTarget::Smart(values_list.1.clone()),
            MatTarget::Smart(values_list.2.clone()),
        ]
    }
}

impl Display for MatTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatTarget::Machine(values) => write!(f, "Machine: {}", values),
            MatTarget::Paper(values) => write!(f, "Paper: {}", values),
            MatTarget::Smart(values) => write!(f, "Smart: {}", values),
        }
    }
}
// impl Display for MatTarget

impl Default for MatTarget {
    fn default() -> Self {
        MatTarget::Smart(MatValues::Equal(10.))
    }
}
