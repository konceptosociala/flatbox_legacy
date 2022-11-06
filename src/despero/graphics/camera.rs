use ash::vk;
use gpu_allocator::vulkan::*;
use nalgebra as na;

use crate::graphics::{
	vulkanish::*,
};

pub struct Camera {
	pub viewmatrix: na::Matrix4<f32>,
	pub position: na::Vector3<f32>,
	pub view_direction: na::Unit<na::Vector3<f32>>,
	pub down_direction: na::Unit<na::Vector3<f32>>,
}

impl Default for Camera {
	fn default() -> Self {
		Camera {
			viewmatrix: na::Matrix4::identity(),
			position: na::Vector3::new(0.0, 0.0, 0.0),
			view_direction: na::Unit::new_normalize(na::Vector3::new(0.0, 0.0, 1.0)),
			down_direction: na::Unit::new_normalize(na::Vector3::new(0.0, 1.0, 0.0)),
		}
	}
}

impl Camera {
	fn update_viewmatrix(&mut self) {
		// Vector in `right` direction
		let r = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction));
		// Vector in `down` direction
		let d = self.down_direction;
		// Vector in `forward` direction
		let v = self.view_direction;
		// Update view matrix
		self.viewmatrix = na::Matrix4::new(
			r.x, r.y, r.z, -r.dot(&self.position),
			d.x, d.y, d.z, -d.dot(&self.position),
			v.x, v.y, v.z, -v.dot(&self.position),
			0.0, 0.0, 0.0, 1.0,
		);
	}
	
	pub fn update_buffer(
		&self,
		logical_device: &ash::Device,
		allocator: &mut Allocator,
		buffer:	&mut Buffer,
	) -> Result<(), vk::Result>{
		let data: [[f32; 4]; 4] = self.viewmatrix.into();
		buffer.fill(logical_device, allocator, &data)?;
		Ok(())
	}
	
	pub fn move_forward(&mut self, distance: f32) {
		self.position += distance * self.view_direction.as_ref();
		self.update_viewmatrix();
	}
	
	pub fn move_backward(&mut self, distance:f32){
		self.move_forward(-distance);
	}

	pub fn turn_right(&mut self, angle: f32) {
		let rotation = na::Rotation3::from_axis_angle(&self.down_direction, angle);
		self.view_direction = rotation * self.view_direction;
		self.update_viewmatrix();
	}
	
	pub fn turn_left(&mut self, angle: f32) {
		self.turn_right(-angle);
	}
	
	pub fn turn_up(&mut self, angle: f32) {
		// Vector in `right` direction
		let right = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction));
		let rotation = na::Rotation3::from_axis_angle(&right, angle);
		self.view_direction = rotation * self.view_direction;
		self.down_direction = rotation * self.down_direction;
		self.update_viewmatrix();
	}
	
	pub fn turn_down(&mut self, angle: f32) {
		self.turn_up(-angle);
	}


} 	

