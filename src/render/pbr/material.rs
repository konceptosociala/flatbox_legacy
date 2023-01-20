use std::any::Any;
use ash::vk;

use crate::render::{
	renderer::*,
	backend::{
		pipeline::*,
		shader::*,
	},
};

pub struct MaterialHandle(usize);

impl MaterialHandle {
	pub fn new(index: usize) -> Self {
		MaterialHandle(index)
	}
	
	pub fn get(&self) -> usize {
		self.0
	}
}

/// Trait for materials to be used in [`ModelBundle`]
pub trait Material: Any + std::fmt::Debug {
	fn pipeline(renderer: &Renderer) -> Pipeline
	where
        Self: Sized;
}

/// Default material, which uses standard shader and graphics pipeline
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DefaultMat {
	pub texture_id: u32,
	pub metallic: f32,
	pub roughness: f32,
}

impl DefaultMat {
	/// Create new instance of default material
	pub fn new(
		texture_id: usize,
		metallic: f32,
		roughness: f32,
	) -> DefaultMat {
		DefaultMat {
			texture_id: texture_id as u32,
			metallic,
			roughness,
		}
	}
}

impl Material for DefaultMat {
	fn pipeline(renderer: &Renderer) -> Pipeline {
		let vertex_shader = vk::ShaderModuleCreateInfo::builder()
			.code(vk_shader_macros::include_glsl!(
				"./shaders/vertex_combined.glsl", 
				kind: vert,
			));
		
		let fragment_shader = vk::ShaderModuleCreateInfo::builder()
			.code(vk_shader_macros::include_glsl!(
				"./shaders/fragment_combined.glsl",
				kind: frag,
			));
			
		let instance_attributes = vec![
			ShaderInputAttribute{
				binding: 1,
				location: 3,
				offset: 0,
				format: vk::Format::R8G8B8A8_UINT,
			},
			ShaderInputAttribute{
				binding: 1,
				location: 4,
				offset: 4,
				format: vk::Format::R32_SFLOAT,
			},
			ShaderInputAttribute{
				binding: 1,
				location: 5,
				offset: 8,
				format: vk::Format::R32_SFLOAT,
			}
		];
		
		unsafe {
			Pipeline::init(
				&renderer,
				&vertex_shader,
				&fragment_shader,
				instance_attributes,
				12,
			).expect("Cannot create pipeline")
		}
	}
}
