use std::f32::consts::PI;
use nalgebra::*;

pub struct Transform {
	pub translation: Vector3<f32>,
	pub rotation: Rotation3<f32>,
	pub scale: f32,
}

impl Default for Transform {
	fn default() -> Self{
		Transform {
			translation: Vector3::new(0.0, 0.0, 0.0),
			rotation: Rotation3::identity(),
			scale: 1.0,
		}
	}
}
