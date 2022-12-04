use ash::vk;
use gpu_allocator::vulkan::Allocator;
use nalgebra as na;

use crate::render::buffer::Buffer;

pub struct DirectionalLight {
	pub direction: na::Vector3<f32>,
	pub illuminance: [f32; 3],
}

pub struct PointLight {
	pub position: na::Point3<f32>,
	pub luminous_flux: [f32; 3],
}

pub enum Light {
	Directional(DirectionalLight),
	Point(PointLight),
}

// Turn object into enum variant
//
// Point Light
impl From<PointLight> for Light {
	fn from(p: PointLight) -> Self {
		Light::Point(p)
	}
}
// Directional Light
impl From<DirectionalLight> for Light {
	fn from(d: DirectionalLight) -> Self {
		Light::Directional(d)
	}
}

// LightManager
pub struct LightManager {
	directional_lights: Vec<DirectionalLight>,
	point_lights: Vec<PointLight>,
}

impl Default for LightManager {
	fn default() -> Self {
		LightManager {
			directional_lights: vec![],
			point_lights: vec![],
		}
	}
}

impl LightManager {
	pub fn add_light<T: Into<Light>>(&mut self, l: T) {
		// Check whether it is a light
		match l.into() {
			Light::Directional(dl) => {
				self.directional_lights.push(dl);
			}
			Light::Point(pl) => {
				self.point_lights.push(pl);
			}
		}
	}
	
	// Push lights to buffer
	pub fn update_buffer(
		&self,
		logical_device: &ash::Device,
		allocator: &mut Allocator,
		buffer:	&mut Buffer,
		descriptor_sets_light: &mut [vk::DescriptorSet],
	) -> Result<(), vk::Result> {
		let mut data: Vec<f32> = vec![];
		data.push(self.directional_lights.len() as f32);
        data.push(self.point_lights.len() as f32);
        data.push(0.0);
        data.push(0.0);
		for dl in &self.directional_lights {
			data.push(dl.direction.x);
			data.push(dl.direction.y);
			data.push(dl.direction.z);
			data.push(0.0);
			data.push(dl.illuminance[0]);
			data.push(dl.illuminance[1]);
			data.push(dl.illuminance[2]);
			data.push(0.0);
		}
		for pl in &self.point_lights {
			data.push(pl.position.x);
			data.push(pl.position.y);
			data.push(pl.position.z);
			data.push(0.0);
			data.push(pl.luminous_flux[0]);
			data.push(pl.luminous_flux[1]);
			data.push(pl.luminous_flux[2]);
			data.push(0.0);
		}
		buffer.fill(logical_device, allocator, &data)?;
		// Update descriptor_sets
		for descset in descriptor_sets_light {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: buffer.buffer,
                offset: 0,
                range: 4 * data.len() as u64,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
        }
		Ok(())
	}
}
