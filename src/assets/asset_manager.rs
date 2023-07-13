use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use serde::{Serialize, Deserialize};

#[cfg(feature = "render")]
use std::sync::Arc;
#[cfg(feature = "render")]
use parking_lot::{RwLock, MappedRwLockReadGuard, MappedRwLockWriteGuard};
#[cfg(feature = "render")]
use ash::vk;

use crate::audio::AudioManager;

#[cfg(feature = "render")]
use super::AssetHandle;
#[cfg(feature = "render")]
use crate::render::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetManager {
    pub audio: AudioManager,
    #[cfg(feature = "render")]
    pub textures: Vec<Texture>,
    #[cfg(feature = "render")]
    pub materials: Vec<Arc<RwLock<Box<dyn Material>>>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        AssetManager {
            audio: AudioManager::default(),
            #[cfg(feature = "render")]
            textures: vec![
                Texture::new_solid(Color::WHITE, 16, 16),
                Texture::new_solid(Color::NORMAL, 16, 16),
            ],
            #[cfg(feature = "render")]
            materials: vec![],
        }
    }
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager::default()
    }

    pub fn cleanup(
        &mut self,
        #[cfg(feature = "render")]
        renderer: &mut Renderer,
    ){
        self.audio.cleanup();

        #[cfg(feature = "render")]{
            for texture in &mut self.textures {
                texture.cleanup(renderer);
            }
            
            self.textures.clear();
            self.materials.clear();
        }
    }
}

#[cfg(feature = "render")]
impl AssetManager {
    pub fn create_texture(
        &mut self,
        path: &'static str,
        filter: Filter,
    ) -> AssetHandle<'T'> {
        let new_texture = Texture::new_from_path(
            path,
            filter,
        );
        
        let new_id = self.textures.len();
        self.textures.push(new_texture);
        AssetHandle(new_id)
    }

    pub fn create_solid_texture(
        &mut self,
        color: impl Into<Color<u8>>,
        width: u32,
        height: u32,
    ) -> AssetHandle<'T'> {
        let new_texture = Texture::new_solid(color.into(), width, height);
        let new_id = self.textures.len();
        self.textures.push(new_texture);
        AssetHandle(new_id)
    }

    pub fn create_raw_texture(
        &mut self,
        raw_data: &[u8],
        filter: Filter,
        width: u32,
        height: u32,
    ) -> AssetHandle<'T'> {
        let new_texture = Texture::new_from_raw(raw_data, filter, width, height);
        let new_id = self.textures.len();
        self.textures.push(new_texture);
        AssetHandle(new_id)
    }
    
    pub fn create_material<M: Material + Send + Sync>(
        &mut self,
        material: M,
    ) -> AssetHandle<'M'> {
        let index = self.materials.len();
        self.materials.push(Arc::new(RwLock::new(Box::new(material))));
        AssetHandle(index)
    }
    
    pub fn get_texture(&self, handle: AssetHandle<'T'>) -> Option<&Texture> {
        self.textures.get(handle.0)
    }
    
    pub fn get_texture_mut(&mut self, handle: AssetHandle<'T'>) -> Option<&mut Texture> {
        self.textures.get_mut(handle.0)
    }

    pub fn get_material(&self, handle: AssetHandle<'M'>) -> Option<RwLockReadGuard<Box<dyn Material>>> {
        if let Some(material) = self.materials.get(handle.0) {
            return material.try_read();  
        }

        None
    }

    pub fn get_material_mut(&self, handle: AssetHandle<'M'>) -> Option<RwLockWriteGuard<Box<dyn Material>>> {
        if let Some(material) = self.materials.get(handle.0) {
            return material.try_write();  
        }

        None
    }
    
    pub fn get_material_downcast<M: Material>(&self, handle: AssetHandle<'M'>) -> Option<MappedRwLockReadGuard<M>> {
        if let Some(material) = self.materials.get(handle.0) {
            let data = match material.try_read() {
                Some(data) => data,
                None => return None,
            };
            
            return RwLockReadGuard::try_map(data, |data| {
                data.as_any().downcast_ref::<M>()
            }).ok()            
        }

        None
    }

    pub fn get_material_downcast_mut<M: Material>(&self, handle: AssetHandle<'M'>) -> Option<MappedRwLockWriteGuard<M>> {
        if let Some(material) = self.materials.get(handle.0) {
            let data = match material.try_write() {
                Some(data) => data,
                None => return None,
            };
            
            return RwLockWriteGuard::try_map(data, |data| {
                data.as_any_mut().downcast_mut::<M>()
            }).ok()            
        }

        None
    }
    
    pub fn descriptor_image_info(&self) -> Vec<vk::DescriptorImageInfo> {
        self.textures
            .iter()
            .filter_map(|t| {
                if let (Some(image_view), Some(sampler)) = (t.imageview, t.sampler) {
                    Some(
                        vk::DescriptorImageInfo {
                            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                            image_view,
                            sampler,
                            ..Default::default()
                        }
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}
