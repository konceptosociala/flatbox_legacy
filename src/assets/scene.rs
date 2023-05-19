use std::path::Path;
use std::fs::read_to_string;
use std::sync::Arc;

use serde::{
    Serialize, 
    Deserialize
};

use crate::ecs::*;
use crate::error::*;
use crate::assets::asset_manager::*;

#[typetag::serde(tag = "component")]
pub trait SerializableComponent: Component {
    fn add_into(&self, entity_builder: &mut EntityBuilder);
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "Entity")]
pub struct SerializableEntity {
    pub components: Vec<Arc<dyn SerializableComponent + 'static>>
}

#[derive(Default, Serialize, Deserialize)]
pub struct Scene {
    pub assets: AssetManager,
    pub entities: Vec<SerializableEntity>,
}

impl Scene {    
    pub fn new() -> Self {
        Scene::default()
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> DesperoResult<Self> {     
        Ok(ron::from_str::<Scene>(
            &read_to_string(path)?
        )?)
    }
}

pub trait SpawnSceneExt {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager);
}

impl SpawnSceneExt for CommandBuffer {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager) {
        for entity in scene.entities {
            let mut entity_builder = EntityBuilder::new();
            
            for component in entity.components {
                component.add_into(&mut entity_builder);
            }
            
            self.spawn(entity_builder.build());
        }
        
        asset_manager.append(scene.assets);
    }
}

impl SpawnSceneExt for World {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager) {
        for entity in scene.entities {
            let mut entity_builder = EntityBuilder::new();
            
            for component in entity.components {
                component.add_into(&mut entity_builder);
            }
            
            self.spawn(entity_builder.build());
        }
        
        asset_manager.append(scene.assets);
    }
}
