use serde::{Serialize, Deserialize};
use as_any::AsAny;
use ash::vk;

use crate::assets::AssetHandle;
use crate::render::{
    renderer::*,
    backend::{
        pipeline::*,
        shader::*,
    },
};

/// Trait for materials to be used in [`Renderer`]
#[typetag::serde(tag = "material")]
pub trait Material: AsAny + std::fmt::Debug + Send + Sync {
    fn pipeline(renderer: &Renderer) -> Pipeline
    where
        Self: Sized;
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
    pub normal: f32,
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
            color: [1.0, 1.0, 1.0],
            albedo: 0,
            metallic: 0.5,
            metallic_map: 0,
            roughness: 0.5,
            roughness_map: 0,
            normal: 1.0,
            normal_map: 0,
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
}

pub struct DefaultMatBuilder {
    pub color: [f32; 3],
    pub albedo: AssetHandle<'T'>,
    pub metallic: f32,
    pub metallic_map: AssetHandle<'T'>,
    pub roughness: f32,
    pub roughness_map: AssetHandle<'T'>,
    pub normal: f32,
    pub normal_map: AssetHandle<'T'>,
}

impl DefaultMatBuilder {
    pub fn new() -> Self {
        DefaultMatBuilder {
            color: [1.0, 1.0, 1.0],
            albedo: AssetHandle::new(),
            metallic: 0.5,
            metallic_map: AssetHandle::new(),
            roughness: 0.5,
            roughness_map: AssetHandle::new(),
            normal: 1.0,
            normal_map: AssetHandle::new(),
        }
    }
    
    pub fn color(mut self, value: impl Into<[f32; 3]>) -> Self {
        self.color = value.into();
        self
    }
    
    pub fn albedo(mut self, handle: AssetHandle<'T'>) -> Self {
        self.albedo = handle;
        self
    }

    pub fn metallic(mut self, value: f32) -> Self {
        self.metallic = value;
        self
    }

    pub fn metallic_map(mut self, handle: AssetHandle<'T'>) -> Self {
        self.metallic_map = handle;
        self
    }
    
    pub fn roughness(mut self, value: f32) -> Self {
        self.roughness = value;
        self
    }
    
    pub fn roughness_map(mut self, handle: AssetHandle<'T'>) -> Self {
        self.roughness_map = handle;
        self
    }

    pub fn normal(mut self, value: f32) -> Self {
        self.normal = value;
        self
    }

    pub fn normal_map(mut self, handle: AssetHandle<'T'>) -> Self {
        self.normal_map = handle;
        self
    }
    
    pub fn build(self) -> DefaultMat {
        DefaultMat {
            color: self.color,
            albedo: self.albedo.into(),
            metallic: self.metallic,
            metallic_map: self.metallic_map.into(),
            roughness: self.roughness,
            roughness_map: self.roughness_map.into(),
            normal: self.normal,
            normal_map: self.normal_map.into(),
        }
    }
}
