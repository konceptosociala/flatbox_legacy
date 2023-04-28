use std::path::Path;
use std::fs::read_to_string;
use std::sync::Arc;
use std::collections::HashMap;

use serde::{
    Serialize, 
    Deserialize
};

use crate::ecs::*;
use crate::error::*;
use crate::assets::asset_manager::*;

#[typetag::serde(tag = "component")]
pub trait SerializableComponent: Component {}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "Entity")]
pub struct SerializableEntity {
    pub components: Vec<Arc<dyn SerializableComponent + 'static>>
}

#[derive(Default, Serialize, Deserialize)]
pub struct Scene {
    pub assets: HashMap<AssetHandle, Arc<dyn Asset>>,
    pub entities: Vec<SerializableEntity>,
}

impl Scene {
    //~ pub const STRUCT: &str = "Scene";
    //~ pub const ASSETS_FIELD: &str = "assets";
    //~ pub const ENTITIES_FIELD: &str = "entities";
    
    pub fn new() -> Self {
        Scene::default()
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> DesperoResult<Self> {
        let scene = read_to_string(path)?;
        
        Ok(Scene {
            assets: HashMap::new(),
            entities: vec![],
        })
    }
}

//~ impl Serialize for Scene {
    //~ fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        //~ let mut state = serializer.serialize_struct(Scene::STRUCT, 2)?;
        //~ state.serialize_field(
            //~ Scene::ASSETS_FIELD,
            //~ &AssetsSerializer {
                //~ assets: &self.assets,
            //~ }
        //~ )?;
        //~ state.serialize_field(Scene::ENTITIES_FIELD, &self.entities)?;
        //~ state.end()
    //~ }
//~ }

//~ pub struct AssetsSerializer<'a> {
    //~ pub assets: &'a [Arc<dyn Asset>],
//~ }

//~ impl<'a> Serialize for AssetsSerializer<'a> {
    //~ fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        //~ let mut state = serializer.serialize_map(Some(self.assets.len()))?;
    //~ }
//~ }

pub trait SpawnSceneExt {
    fn spawn_scene(&mut self, scene: Scene);
}

impl SpawnSceneExt for CommandBuffer {
    fn spawn_scene(&mut self, scene: Scene) {
        for entity in scene.entities {
            let mut entity_builder = EntityBuilder::new();
            
            for component in entity.components {
                entity_builder.add(component);
            }
            
            self.spawn(entity_builder.build());
        }
    }
}
