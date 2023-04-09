use std::f32::consts::PI;

pub fn to_radian(deg: f32) -> f32 {
    deg / 180.0 * PI
}

pub fn to_degrees(rad: f32) -> f32 {
    rad / PI * 180.0
}
