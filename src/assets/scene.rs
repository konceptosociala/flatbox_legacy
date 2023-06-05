use std::path::Path;
use std::fs::read_to_string;
use std::sync::Arc;
use std::fs::File;
use tar::EntryType;
use ron::ser::{Serializer, PrettyConfig};

use serde::{
    Serialize, 
    Deserialize
};

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
    
    /// Load scene with assets from compressed `.lvl` package.
    /// It is `.tar.lz4` package which can be created manually or with 
    /// [Metio Editor](https://konceptosociala.eu.org/softvaro/metio).
    /// 
    /// It has the following structure:
    /// 
    /// ```text
    /// my_awesome_scene.lvl/
    /// ├─ scene.ron
    /// ├─ textures/
    /// │  ├─ texture1.jpg
    /// │  ├─ texture2.png
    /// ├─ sounds/
    /// │  ├─ sound1.mp3
    /// ```
    pub fn load_packaged<P: AsRef<Path>>(path: P) -> DesperoResult<Self> {
        // TODO: Packaged scene
        // 
        let mut asset_manager = AssetManager::new();

        let package = File::open("assets.pkg")?;
        let decoded = lz4::Decoder::new(package)?;
        let mut archive = tar::Archive::new(decoded);

        for file in archive.entries().unwrap() {
            let file = file.unwrap();
            let header = file.header();
            match header.entry_type() {
                EntryType::Regular => {
                    if header.path().unwrap() == Path::new("manager.ron") {
                        asset_manager = ron::de::from_reader(file)?;
                    }
                },
                EntryType::Directory => {
                    // if `sounds`:
                    //
                    //

                    // if `textures`:
                    // for texture in asset_manager.textures {
                    //     let reader = BufReader::new(file);
                    //     let image = image::load(reader, ImageFormat::from_path(path));
                    //     texture.generate_from(image);
                    // }
                    // 
                },
                _ => {},
            }
        }
        Ok(Self::new())
    }
    
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> DesperoResult<()> {     
        let buf = File::create(path)?;                    
        let mut ser = Serializer::new(buf, Some(
            PrettyConfig::new()
                .struct_names(true)
        ))?;   
        
        self.serialize(&mut ser)?;
                        
        Ok(())
    }
}

pub trait SpawnSceneExt {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager);
}

impl SpawnSceneExt for CommandBuffer {
    fn spawn_scene(&mut self, scene: Scene, asset_manager: &mut AssetManager) {
        self.write(|world| {
            world.clear();
        });

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