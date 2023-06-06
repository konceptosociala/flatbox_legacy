use std::sync::Arc;
use std::io::{Read, Cursor};
use std::path::Path;
use std::fs::{File, read_to_string};

#[cfg(feature = "render")]
use image::ImageFormat;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use tar::EntryType;
use ron::ser::{Serializer, PrettyConfig};

use serde::{
    Serialize, 
    Deserialize
};

use crate::audio::AudioError;
use crate::ecs::*;
use crate::error::*;
use crate::assets::{
    asset_manager::*,
    ser_component::*,
};

use super::AssetLoadType;

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
    
    /// Load scene with assets from compressed `.tar.lz4` package.
    /// It is ordinary package which can be created manually or with 
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
        let mut scene = Scene::new();

        let package = File::open(path)?;
        let decoded = lz4::Decoder::new(package)?;
        let mut archive = tar::Archive::new(decoded);

        let mut entries = vec![]; // Vec<(Header, Vec<u8>)>

        for file in archive.entries().unwrap() {
            let mut file = file.unwrap();
            let header = file.header().clone();

            let mut bytes = vec![];
            file.read_to_end(&mut bytes)?;

            entries.push((header, bytes));
        }

        for (header, file) in &mut entries {
            let path = header.path().unwrap();
            if path == Path::new("scene.ron") {
                log::debug!("Deserializing scene `{}`...", path.display());
                scene = ron::de::from_reader(&**file)?;
            }
        }

        for (header, file) in entries {
            let filepath = header.path().unwrap();

            if header.entry_type() == EntryType::Regular {
                let name = filepath
                    .file_stem()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();

                #[cfg(feature = "render")]
                let ext = filepath.extension().unwrap().to_owned();

                if filepath.starts_with("textures") {
                    #[cfg(feature = "render")]
                    for texture in &mut scene.assets.textures {
                        if texture.load_type == AssetLoadType::Resource(name.clone()) {
                            let cursor = Cursor::new(file.clone());

                            let image = image::load(
                                cursor, 
                                ImageFormat::from_extension(ext.clone()).expect("Wrong image extension!")
                            )
                            .map(|img| img.to_rgba8())
                            .expect("Unable to open image");
                            
                            texture.image = Some(image);
                        }
                    }
                } else if filepath.starts_with("audio") {
                    for sound in &mut scene.assets.audio.sounds {
                        if sound.load_type == AssetLoadType::Resource(name.clone()) {
                            let cursor = Cursor::new(file.clone());

                            let static_data = StaticSoundData::from_media_source(
                                cursor,
                                StaticSoundSettings::default(),
                            ).map_err(|e| AudioError::from(e))?;

                            sound.static_data = Some(static_data);
                        }
                    }
                }
            }
        }
        
        Ok(scene)
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
        // self.write(|world| {
        //     world.clear();
        // });
        // TODO: Clear world

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
        // self.clear();

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