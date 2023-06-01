use serde::{Serialize, Deserialize};

#[cfg(feature = "render")]
use std::sync::Arc;
#[cfg(feature = "render")]
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
#[cfg(feature = "render")]
use ash::vk;

use crate::audio::sound::Sound;
#[cfg(feature = "render")]
use crate::render::*;

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
    
    pub fn append(&mut self, count: usize) {
        self.0 += count;
    }
}

impl From<AssetHandle> for u32 {
    fn from(value: AssetHandle) -> Self {
        value.unwrap() as u32
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AssetManager {
    pub sounds: Vec<Sound>,
    #[cfg(feature = "render")]
    pub textures: Vec<Texture>,
    #[cfg(feature = "render")]
    pub materials: Vec<Arc<Mutex<Box<dyn Material>>>>,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager::default()
    }    

    pub fn create_sound() -> AssetHandle {
        todo!();
    }

    pub fn get_sound(&self, _handle: AssetHandle) -> Option<&Sound> {
        todo!();
    }

    pub fn get_sound_mut(&mut self, _handle: AssetHandle) -> Option<&mut Sound> {
        todo!();
    }

    pub fn append(&mut self, other: Self) {
        self.sounds.extend(other.sounds);
        #[cfg(feature = "render")]{
            self.textures.extend(other.textures);
            self.materials.extend(other.materials);
        }
    }

    pub fn cleanup(
        &mut self,
        #[cfg(feature = "render")]
        renderer: &mut Renderer,
    ){
        self.sounds.clear();

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
    ) -> AssetHandle {
        let new_texture = Texture::new_blank(
            path,
            filter,
        );
        
        let new_id = self.textures.len();
        self.textures.push(new_texture);
        AssetHandle(new_id)
    }
    
    pub fn create_material<M: Material + Send + Sync>(
        &mut self,
        material: M,
    ) -> AssetHandle {
        let index = self.materials.len();
        self.materials.push(Arc::new(Mutex::new(Box::new(material))));
        AssetHandle(index)
    }
    
    pub fn get_texture(&self, handle: AssetHandle) -> Option<&Texture> {
        self.textures.get(handle.0)
    }
    
    pub fn get_texture_mut(&mut self, handle: AssetHandle) -> Option<&mut Texture> {
        self.textures.get_mut(handle.0)
    }

    pub fn get_material(&self, handle: AssetHandle) -> Option<MutexGuard<Box<dyn Material>>> {
        if let Some(material) = self.materials.get(handle.0) {
            return Some(material.lock());  
        }

        None
    }
    
    pub fn get_material_downcast<M: Material>(&self, handle: AssetHandle) -> Option<MappedMutexGuard<M>> {
        if let Some(material) = self.materials.get(handle.0) {
            let data = material.lock();
            return MutexGuard::try_map(data, |data| {
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
