use nalgebra::*;

pub struct Transform {
	pub translation: Vector3<f32>,
	pub rotation: Unit<Quaternion<f32>>,
	pub scale: Vector3<f32>,
}

impl Default for Transform {
	fn default() -> Self{
		Transform {
			translation: Vector3::new(0.0, 0.0, 0.0),
			rotation: Unit::new_normalize(
				Quaternion::new(0.0, 0.0, 0.0, 1.0)
			),
			scale: Vector3::new(1.0, 1.0, 1.0),
		}
	}
}
