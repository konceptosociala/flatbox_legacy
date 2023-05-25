use serde::{Serialize, Deserialize};
use std::any::Any;
use ash::vk;

use crate::assets::asset_manager::AssetHandle;
use crate::render::{
    renderer::*,
    backend::{
        pipeline::*,
        shader::*,
    },
};

/// Trait for materials to be used in [`Renderer`]
#[typetag::serde(tag = "material")]
pub trait Material: Any + std::fmt::Debug + Send + Sync {
    fn pipeline(renderer: &Renderer) -> Pipeline
    where
        Self: Sized;
        
    fn as_any(&self) -> &dyn Any;
}

/// Default material, which uses standard shader and graphics pipeline
#[repr(C)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DefaultMat {
    pub color: [f32; 3],
    pub albedo: u32,
    pub metallic: f32,
    pub metallic_map: u32,
    pub roughness: f32,
    pub roughness_map: u32,
    pub normak: f32,
    pub normal_map: u32,
}

impl DefaultMat {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn builder() -> DefaultMatBuilder {
        DefaultMatBuilder::new()
    }
}

impl Default for DefaultMat {
    fn default() -> Self {
        DefaultMat {
            texture_id: 0,
            metallic: 0.0,
            roughness: 1.0,
        }
    }
}

#[typetag::serde]
impl Material for DefaultMat {
    fn pipeline(renderer: &Renderer) -> Pipeline {            
        let instance_attributes = vec![
            ShaderInputAttribute{
                binding: 1,
                location: 3,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 4,
                offset: 12,
                format: vk::Format::R8G8B8A8_UINT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 5,
                offset: 16,
                format: vk::Format::R32_SFLOAT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 6,
                offset: 20,
                format: vk::Format::R8G8B8A8_UINT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 7,
                offset: 24,
                format: vk::Format::R32_SFLOAT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 8,
                offset: 28,
                format: vk::Format::R8G8B8A8_UINT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 9,
                offset: 32,
                format: vk::Format::R32_SFLOAT,
            },
            ShaderInputAttribute{
                binding: 1,
                location: 10,
                offset: 36,
                format: vk::Format::R8G8B8A8_UINT,
            },
        ];
        
        let vertex_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./src/shaders/vertex_combined.glsl", 
                kind: vert,
            ));
        
        let fragment_shader = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!(
                "./src/shaders/fragment_combined.glsl",
                kind: frag,
            ));
        
        Pipeline::init(
            &renderer,
            &vertex_shader,
            &fragment_shader,
            instance_attributes,
            40,
            vk::PrimitiveTopology::TRIANGLE_LIST,
        ).expect("Cannot create pipeline")
    }
    
    fn as_any(&self) -> &dyn Any
    {
        self
    }
}

pub struct DefaultMatBuilder {
    texture_id: AssetHandle,
    metallic: f32,
    roughness: f32,
}

impl DefaultMatBuilder {
    pub fn new() -> Self {
        DefaultMatBuilder {
            texture_id: AssetHandle::new(),
            metallic: 0.25,
            roughness: 0.5,
        }
    }
    
    pub fn texture_id(mut self, id: AssetHandle) -> Self {
        self.texture_id = id;
        self
    }
    
    pub fn metallic(mut self, value: f32) -> Self {
        self.metallic = value;
        self
    }
    
    pub fn roughness(mut self, value: f32) -> Self {
        self.roughness = value;
        self
    }
    
    pub fn build(self) -> DefaultMat {
        DefaultMat {
            texture_id: self.texture_id.unwrap() as u32,
            metallic: self.metallic,
            roughness: self.roughness,
        }
    }
}
