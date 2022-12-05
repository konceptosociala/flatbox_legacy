use nalgebra::*;

pub struct Transform<T> {
	pub translation: Vector3<T>,
	pub rotation: Unit<Quaternion<T>>,
	pub scale: Vector3<T>,
}
