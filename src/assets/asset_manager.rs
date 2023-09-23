use serde::{Serialize, Deserialize};

#[cfg(feature = "render")]
use std::sync::Arc;
#[cfg(feature = "render")]
use parking_lot::{RwLock, MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "render")]
use ash::vk;

use crate::audio::AudioManager;

#[cfg(feature = "render")]
use super::AssetHandle;
#[cfg(feature = "render")]
use crate::render::*;

/// Manager of game assets (e.g. textures, materials, sounds etc.), the part of [`Flatbox`]
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetManager {
    /// Audio loading and processing manager
    pub audio: AudioManager,
    /// Collection of textures, which can be loaded with `create_texture` functions or pushed manually
    #[cfg(feature = "render")]
    pub textures: Vec<Texture>,
    /// Rendered game skybox texture
    #[cfg(feature = "render")]
    pub skybox: Option<SkyBox>,
    /// Game materials collection
    #[cfg(feature = "render")]
    pub materials: Vec<Arc<RwLock<Box<dyn Material>>>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        AssetManager {
            audio: AudioManager::default(),
            #[cfg(feature = "render")]
            textures: vec![
                Texture::new_solid(Color::<u8>::WHITE, TextureType::Plain, 16, 16),
                Texture::new_solid(Color::<u8>::NORMAL, TextureType::Plain, 16, 16),
            ],
            #[cfg(feature = "render")]
            skybox: None,
            #[cfg(feature = "render")]
            materials: vec![],
        }
    }
}

impl AssetManager {
    /// Create new asset manager with audio settings (audio casts and listeners count) specified.
    pub fn new(
        cast_count: usize, 
        listener_count: usize,
    ) -> Self {
        AssetManager {
            audio: AudioManager::new(cast_count, listener_count)
                .expect("Cannot create audio manager"),
            #[cfg(feature = "render")]
            textures: vec![
                Texture::new_solid(Color::<u8>::WHITE, TextureType::Plain, 16, 16),
                Texture::new_solid(Color::<u8>::NORMAL, TextureType::Plain, 16, 16),
            ],
            #[cfg(feature = "render")]
            skybox: None,
            #[cfg(feature = "render")]
            materials: vec![],
        }
    }

    /// Method to destroy all assets and destroy theirs vulkan components. It is automatically called during [`Flatbox::drop`]
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
            
            if let Some(skybox) = &mut self.skybox {
                skybox.0.cleanup(renderer);
                self.skybox = None;
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
        path: impl Into<String>,
        filter: Filter,
    ) -> AssetHandle<'T'> {
        let new_texture = Texture::new_from_path(
            &path.into(),
            filter,
            TextureType::Plain,
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
        let new_texture = Texture::new_solid(color.into(), TextureType::Plain, width, height);
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
        let new_texture = Texture::new_from_raw(raw_data, filter, TextureType::Plain, width, height);
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
