use std::sync::Arc;
use serde::{Serialize, Deserialize};
use ash::vk;
use gpu_allocator::vulkan::{
    Allocation,
};

use crate::render::*;

#[typetag::serde(tag = "asset")]
pub trait Asset {}

#[typetag::serde(name = "asset")]
impl Asset for DefaultMat {}
#[typetag::serde(name = "asset")]
impl Asset for Texture {}

#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetHandle(usize);

impl AssetHandle {
    pub fn new() -> Self {
        AssetHandle::default()
    }
    
    pub fn from_index(index: usize) -> Self {
        AssetHandle(index)
    }
    
    pub fn invalid() -> Self {
        AssetHandle(usize::MAX)
    }
    
    pub fn unwrap(&self) -> usize {
        self.0
    }
}

#[derive(Default)]
pub struct AssetManager {
    pub textures: Vec<Texture>,
    pub materials: Vec<Arc<(dyn Material + Send + Sync)>>, //TODO: Replace with HashMap<AssetHandle, Arc<dyn Asset>>
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager::default()
    }
    
    pub fn create_texture(
        &mut self,
        path: &'static str,
        filter: Filter,
        renderer: &mut Renderer,
    ) -> AssetHandle {
        let new_texture = Texture::new_from_file(
            path,
            filter,
            &renderer.device,
            &mut *renderer.allocator.lock().unwrap(),
            &renderer.commandbuffer_pools.commandpool_graphics,
            &renderer.queue_families.graphics_queue,
        ).expect("Cannot create texture");
        
        let new_id = self.textures.len();
        self.textures.push(new_texture);
        AssetHandle(new_id)
    }
    
    pub fn create_material<M: Material + Send + Sync>(
        &mut self,
        material: M,
    ) -> AssetHandle {
        let index = self.materials.len();
        self.materials.push(Arc::new(material));
        AssetHandle(index)
    }
    
    pub fn get_texture(&self, handle: AssetHandle) -> Option<&Texture> {
        self.textures.get(handle.0)
    }
    
    pub fn get_texture_mut(&mut self, handle: AssetHandle) -> Option<&mut Texture> {
        self.textures.get_mut(handle.0)
    }
    
    pub fn get_material(&self, handle: AssetHandle) -> Option<&Arc<dyn Material + Send + Sync>> {
        self.materials.get(handle.0)
    }
    
    pub fn descriptor_image_info(&self) -> Vec<vk::DescriptorImageInfo> {
        self.textures
            .iter()
            .map(|t| vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: t.imageview,
                sampler: t.sampler,
                ..Default::default()
            })
            .collect()
    }
    
    pub fn cleanup(
        &mut self,
        renderer: &mut Renderer,
    ){
        for texture in &mut self.textures {
            let mut alloc: Option<Allocation> = None;
            std::mem::swap(&mut alloc, &mut texture.image_allocation);
            let alloc = alloc.unwrap();
            (*renderer.allocator.lock().unwrap()).free(alloc).unwrap();
            unsafe { 
                renderer.device.destroy_sampler(texture.sampler, None);
                renderer.device.destroy_image_view(texture.imageview, None);
                renderer.device.destroy_image(texture.vk_image, None);
            }
        }
    }    
}
