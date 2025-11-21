use egui::{Pos2, Vec2};

pub trait IsPos2Able {
    fn into_pos2(&self) -> Pos2;
}

impl IsPos2Able for Pos2 {
    fn into_pos2(&self) -> Pos2 {
        self.clone()
    }
}
impl IsPos2Able for Vec2 {
    fn into_pos2(&self) -> Pos2 {
        self.to_pos2()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RulerOrigin {
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

pub fn rotate_pos2(pos: Pos2, angle: f32) -> Pos2 {
    Pos2::new(
        pos.x * angle.cos() - pos.y * angle.sin(),
        pos.y * angle.cos() + pos.x * angle.sin(),
    )
}

/// Helper that rotates a point around another point.
pub fn rotate_pos2_around_pos2(pos: Pos2, around: Pos2, angle: f32) -> Pos2 {
    let tmp_pos = (pos - around).to_pos2();
    let tmp_pos = rotate_pos2(tmp_pos, angle);
    let tmp_pos = tmp_pos + around.to_vec2();
    tmp_pos
}
