use std::sync::Arc;
use serde::{Serialize, Deserialize};
use as_any::AsAny;
use parking_lot::RwLock;
use vk_shader_macros::include_glsl;

use crate::assets::AssetHandle;
use crate::render::backend::shader::*;

pub use flatbox_macros::Material;

/// Trait for materials to be used in [`Renderer`]
#[typetag::serde(tag = "material")]
pub trait Material: AsAny + std::fmt::Debug + Send + Sync {
    fn vertex() -> &'static [u32]
    where 
        Self: Sized;

    fn fragment() -> &'static [u32]
    where 
        Self: Sized;

    fn input() -> ShaderInput
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
    pub ao: f32,
    pub ao_map: u32,
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
            ao: 1.0,
            ao_map: 0,
        }
    }
}

#[typetag::serde]
impl Material for DefaultMat {
    fn vertex() -> &'static [u32] {
        include_glsl!(
            "src/shaders/defaultmat.vs", 
            kind: vert,
        )
    }

    fn fragment() -> &'static [u32] {
        include_glsl!(
            "src/shaders/defaultmat.fs", 
            kind: frag,
        )
    }

    fn input() -> ShaderInput {
        ShaderInput { 
            attributes: vec![
                ShaderInputAttribute{
                    binding: 1,
                    location: 3,
                    offset: 0,
                    format: ShaderInputFormat::R32G32B32_SFLOAT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 4,
                    offset: 12,
                    format: ShaderInputFormat::R8G8B8A8_UINT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 5,
                    offset: 16,
                    format: ShaderInputFormat::R32_SFLOAT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 6,
                    offset: 20,
                    format: ShaderInputFormat::R8G8B8A8_UINT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 7,
                    offset: 24,
                    format: ShaderInputFormat::R32_SFLOAT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 8,
                    offset: 28,
                    format: ShaderInputFormat::R8G8B8A8_UINT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 9,
                    offset: 32,
                    format: ShaderInputFormat::R32_SFLOAT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 10,
                    offset: 36,
                    format: ShaderInputFormat::R8G8B8A8_UINT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 11,
                    offset: 40,
                    format: ShaderInputFormat::R32_SFLOAT,
                },
                ShaderInputAttribute{
                    binding: 1,
                    location: 12,
                    offset: 44,
                    format: ShaderInputFormat::R8G8B8A8_UINT,
                },
            ], 
            instance_size: 48,
            topology: ShaderTopology::TRIANGLE_LIST,
        }
    }
}

pub struct DefaultMatBuilder {
    color: [f32; 3],
    albedo: AssetHandle<'T'>,
    metallic: f32,
    metallic_map: AssetHandle<'T'>,
    roughness: f32,
    roughness_map: AssetHandle<'T'>,
    normal: f32,
    normal_map: AssetHandle<'T'>,
    ao: f32,
    ao_map: AssetHandle<'T'>,
}

impl DefaultMatBuilder {
    pub fn new() -> Self {
        DefaultMatBuilder {
            color: [1.0, 1.0, 1.0],
            albedo: AssetHandle::BUILTIN_ALBEDO,
            metallic: 0.5,
            metallic_map: AssetHandle::BUILTIN_METALLIC,
            roughness: 0.5,
            roughness_map: AssetHandle::BUILTIN_ROUGHNESS,
            normal: 1.0,
            normal_map: AssetHandle::BUILTIN_NORMAL,
            ao: 1.0,
            ao_map: AssetHandle::BUILTIN_AO,
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

    pub fn ao(mut self, value: f32) -> Self {
        self.ao = value;
        self
    }

    pub fn ao_map(mut self, handle: AssetHandle<'T'>) -> Self {
        self.ao_map = handle;
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
            ao: self.ao,
            ao_map: self.ao_map.into(),
        }
    }
}

pub struct CachedMaterials {
    pub materials: Vec<Arc<RwLock<Box<dyn Material>>>>,
}