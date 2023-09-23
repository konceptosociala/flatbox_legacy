use std::sync::Arc;
use std::path::Path;
use std::fs::{File, read_to_string};

use ron::ser::{Serializer, PrettyConfig};

use serde::{
    Serialize, 
    Deserialize
};

#[cfg(feature = "gltf")]
use crate::render::pbr::gltf::GltfCache;
use crate::ecs::*;
use crate::error::*;
use crate::assets::{
    asset_manager::*,
    ser_component::*,
};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "Entity")]
pub struct SerializableEntity {
    pub components: Vec<Arc<dyn SerializableComponent + 'static>>
}

/// Macro for easy [`SerializableEntity`] creation. Often used along with
/// `scene!` macro during [`Scene`] creating
/// 
/// # Usage example
/// ```rust
/// let entity = entity![
///     AssetHandle::<'M'>::invalid(), 
///     Model::cube(),
///     Transform::default()
/// ];
/// ```
#[macro_export]
macro_rules! entity {
    [$( $comp:expr ),+] => {
        {
            use std::sync::Arc;

            let mut entity = SerializableEntity::default();
            $(
                entity.components.push(Arc::new($comp));
            )+
            entity
        }
    };
}

#[derive(Default, Serialize, Deserialize)]
pub struct Scene {
    pub assets: AssetManager,
    pub entities: Vec<SerializableEntity>,
    #[cfg(feature = "gltf")]
    pub gltf_cache: GltfCache,
}

impl Scene {    
    pub fn new() -> Self {
        Scene::default()
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> FlatboxResult<Self> {     
        Ok(ron::from_str::<Scene>(
            &read_to_string(path)?
        )?)
    }
    
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> FlatboxResult<()> {     
        let buf = File::create(path)?;                    
        let mut ser = Serializer::new(buf, Some(
            PrettyConfig::new()
                .struct_names(true)
        ))?;   
        
        self.serialize(&mut ser)?;
                        
        Ok(())
    }
}

/// Macro for easy [`Scene`] creation. `entities` can be created with [`entity!`] 
/// macro or manually:
/// ```rust
/// let entity = SerializableEntity {
///     components: vec![
///         Arc::new(comp1),
///         Arc::new(comp2),
///     ],
/// };
/// ```
/// 
/// # Usage example
/// ```rust
/// let scene = scene! {
///     assets: AssetManager::new(),
///     entities: [
///         entity![
///             Camera::builder()
///                 .is_active(true)
///                 .camera_type(CameraType::FirstPerson)
///                 .build(),
///             Transform::default()
///         ],
///         entity![
///             AssetHandle::<'M'>::invalid(), 
///             Model::cube(),
///             Transform::default()
///         ]
///     ]
/// };
/// ```
#[macro_export]
macro_rules! scene {
    {
        assets: $assets:expr,
        entities: [$( $entity:expr ),+]
    } => {
        {
            let mut entities = Vec::new();
            $(
                entities.push($entity);
            )+

            Scene {
                assets: $assets,
                entities,
            }
        }
    };
}

pub trait SpawnSceneExt {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager);
}

impl SpawnSceneExt for World {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager) {
        self.clear();

        for entity in scene.entities {
            let mut entity_builder = EntityBuilder::new();
            
            for component in entity.components {
                component.add_into(&mut entity_builder);
            }
            
            self.spawn(entity_builder.build());
        }
        
        *asset_manager = scene.assets;
    }
}