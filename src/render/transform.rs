pub use std::f32::consts::PI;
use nalgebra::*;

pub struct Transform {
	pub translation: Vector3<f32>,
	pub rotation: UnitQuaternion<f32>,
	pub scale: f32,
}

impl Default for Transform {
	fn default() -> Self{
		Transform {
			translation: Vector3::new(0.0, 0.0, 0.0),
			rotation: UnitQuaternion::identity(),
			scale: 1.0,
		}
	}
}
