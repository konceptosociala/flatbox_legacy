use ash::vk;
use nalgebra as na;
use despero_ecs::*;

use crate::render::{
	renderer::Renderer,
	debug::*,
};

use crate::math::transform::Transform;

pub struct Camera {
	pub(crate) viewmatrix: na::Matrix4<f32>,
	pub(crate) projectionmatrix: na::Matrix4<f32>,
	pub(crate) position: na::Vector3<f32>,
	pub(crate) view_direction: na::Unit<na::Vector3<f32>>,
	pub(crate) down_direction: na::Unit<na::Vector3<f32>>,
	pub(crate) fovy: f32,
	pub(crate) aspect: f32,
	pub(crate) near: f32,
	pub(crate) far: f32,
	pub(crate) is_active: bool,
}

impl Camera {
	pub fn builder() -> CameraBuilder {
		CameraBuilder {
			position: na::Vector3::new(0.0, -3.0, -3.0),
			view_direction: na::Unit::new_normalize(na::Vector3::new(0.0, 1.0, 1.0)),
			down_direction: na::Unit::new_normalize(na::Vector3::new(0.0, 1.0, -1.0)),
			fovy: std::f32::consts::FRAC_PI_3,
			aspect: 800.0 / 600.0,
			near: 0.1,
			far: 100.0,
			is_active: false,
		}
	}
	
	fn update_projectionmatrix(&mut self) {
		let d = 1.0 / (0.5 * self.fovy).tan();
		self.projectionmatrix = na::Matrix4::new(
			d / self.aspect,
			0.0,
			0.0,
			0.0,
			0.0,
			d,
			0.0,
			0.0,
			0.0,
			0.0,
			self.far / (self.far - self.near),
			-self.near * self.far / (self.far - self.near),
			0.0,
			0.0,
			1.0,
			0.0,
		);
	}
	
	fn update_viewmatrix(&mut self) {
		let r = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction)); // Right
		let d = self.down_direction; // Down
		let v = self.view_direction; // Forward
		self.viewmatrix = na::Matrix4::new(
			r.x, r.y, r.z, -r.dot(&self.position),
			d.x, d.y, d.z, -d.dot(&self.position),
			v.x, v.y, v.z, -v.dot(&self.position),
			0.0, 0.0, 0.0, 1.0,
		);
	}
	
	pub(crate) fn update_buffer(
		&self,
		renderer: &mut Renderer,
	) -> Result<(), vk::Result>{		
		let data: [[[f32; 4]; 4]; 2] = [self.viewmatrix.into(), self.projectionmatrix.into()];
		renderer.camera_buffer.fill(&renderer.device, &mut *renderer.allocator.lock().unwrap(), &data)?;
		Ok(())
	}
	
	pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_projectionmatrix();
    }
	
	pub fn move_forward(&mut self, distance: f32) {
		self.position += distance * self.view_direction.as_ref();
		self.update_viewmatrix();
	}
	
	pub fn move_backward(&mut self, distance:f32){
		self.move_forward(-distance);
	}
	
	pub fn move_left(&mut self, distance: f32) {
		self.position += distance * *na::Unit::new_normalize(-self.down_direction.cross(&self.view_direction));
		self.update_viewmatrix();
	}
	
	pub fn move_right(&mut self, distance:f32){
		self.move_left(-distance);
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

pub struct CameraBuilder {
	position: na::Vector3<f32>,
	view_direction: na::Unit<na::Vector3<f32>>,
	down_direction: na::Unit<na::Vector3<f32>>,
	fovy: f32,
	aspect: f32,
	near: f32,
	far: f32,
	is_active: bool,
}

impl CameraBuilder {
	pub fn build(self) -> Camera {
		if self.far < self.near {
			error!("Far plane (at {}) is closer than near plane (at {})!", self.far, self.near);
		}
		
		let mut cam = Camera {
			position: self.position,
			view_direction: self.view_direction,
			down_direction: na::Unit::new_normalize(
				self.down_direction.as_ref()
					- self
						.down_direction
						.as_ref()
						.dot(self.view_direction.as_ref())
						* self.view_direction.as_ref(),
			),
			fovy: self.fovy,
			aspect: self.aspect,
			near: self.near,
			far: self.far,
			viewmatrix: na::Matrix4::identity(),
			projectionmatrix: na::Matrix4::identity(),
			is_active: self.is_active,
		};
		cam.update_projectionmatrix();
		cam.update_viewmatrix();
		cam
	}
	
	pub fn position(mut self, pos: na::Vector3<f32>) -> CameraBuilder {
		self.position = pos;
		self
	}
	
	pub fn fovy(mut self, fovy: f32) -> CameraBuilder {
		self.fovy = fovy.max(0.01).min(std::f32::consts::PI - 0.01);
		self
	}
	
	pub fn aspect(mut self, aspect: f32) -> CameraBuilder {
		self.aspect = aspect;
		self
	}
	
	pub fn near(mut self, near: f32) -> CameraBuilder {
		if near <= 0.0 {
			error!("Near plane ({}) can't be negative!", near);
		}
		self.near = near;
		self
	}
	
	pub fn far(mut self, far: f32) -> CameraBuilder {
		if far <= 0.0 {
			error!("Far plane ({}) can't be negative!", far);
		}
		self.far = far;
		self
	}
	
	pub fn view_direction(mut self, direction: na::Vector3<f32>) -> CameraBuilder {
		self.view_direction = na::Unit::new_normalize(direction);
		self
	}
	
	pub fn down_direction(mut self, direction: na::Vector3<f32>) -> CameraBuilder {
		self.down_direction = na::Unit::new_normalize(direction);
		self
	}
	
	pub fn is_active(mut self, is_active: bool) -> CameraBuilder {
		self.is_active = is_active;
		self
	}
}

#[derive(Bundle)]
pub struct CameraBundle {
	pub camera: Camera,
	pub transform: Transform,
}
