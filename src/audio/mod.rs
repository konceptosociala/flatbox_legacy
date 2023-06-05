use std::sync::Arc;
use std::fmt;
use kira::{ 
    spatial::{scene::{
        SpatialSceneHandle, 
        SpatialSceneSettings
    }, emitter::EmitterSettings, listener::ListenerSettings}, 
    manager::{
        AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
};
use nalgebra::{Vector3, Quaternion};
use parking_lot::{Mutex, MutexGuard};
use serde::{
    Serialize, Deserialize, 
    de::{
        Visitor,
        SeqAccess,
        MapAccess,
        Error as DeError,
    },
};

#[allow(unused_imports)]
use crate::assets::{
    AssetHandle,
    asset_manager::AssetManager,
};
use crate::error::DesperoResult;

pub mod cast;
pub mod error;
pub mod listener;
pub mod sound;

pub use cast::*;
pub use error::*;
pub use listener::*;
pub use sound::*;

type KiraAudioManager = kira::manager::AudioManager<CpalBackend>; 

/// Main audio managment struct. It's actually a part of [`AssetManager`]
#[derive(Serialize)]
pub struct AudioManager {
    sounds: Vec<Sound>,
    cast_count: usize,
    listener_count: usize,

    #[serde(skip_serializing)]
    manager: Arc<Mutex<KiraAudioManager>>,
    #[serde(skip_serializing)]
    scene: SpatialSceneHandle,
}

impl AudioManager {
    pub fn new(
        cast_count: usize,
        listener_count: usize,
    ) -> DesperoResult<Self> {
        let settings = SpatialSceneSettings::new()
            .emitter_capacity(cast_count)
            .listener_capacity(listener_count);
        
        let mut manager = KiraAudioManager::new(AudioManagerSettings::default()).map_err(|e| AudioError::from(e))?;
        let scene = manager.add_spatial_scene(settings).map_err(|e| AudioError::from(e))?;
        
        Ok(AudioManager { 
            sounds: vec![],
            cast_count,
            listener_count,
            manager: Arc::new(Mutex::new(manager)),
            scene,
        })
    }

    pub fn new_sound(&mut self, path: &'static str) -> DesperoResult<AssetHandle> {
        let sound = Sound::new_from_file(path)?;

        Ok(self.push_sound(sound))
    }

    pub fn push_sound(&mut self, sound: Sound) -> AssetHandle {
        let index = self.sounds.len();
        self.sounds.push(sound);

        AssetHandle::from_index(index)
    }

    pub fn play(&mut self, handle: AssetHandle) -> DesperoResult<()>{
        match self.get_sound(handle) {
            Some(sound) => {
                self.inner()
                    .play(sound.static_data.clone().unwrap())
                    .map_err(|e| AudioError::from(e))?;
            },
            None => {
                log::error!("Sound with handle {handle:?} not found!");
            },
        }

        Ok(())
    }

    pub fn new_cast(
        &mut self, 
    ) -> AudioCast {
        let handle = self.scene.add_emitter(
            Vector3::identity(),
            EmitterSettings::default(),
        ).expect("Cannot create audio cast");

        AudioCast { handle }
    }

    pub fn new_listener(
        &mut self,
    ) -> AudioListener {
        let handle = self.scene.add_listener(
            Vector3::identity(),
            Quaternion::identity(),
            ListenerSettings::default(),
        ).expect("Cannot create audio cast");

        AudioListener { handle }
    }

    pub fn create_sound(
        &mut self,
        path: &'static str,
    ) -> DesperoResult<AssetHandle> {
        let index = self.sounds.len();
        self.sounds.push(Sound::new_from_file(path)?);

        Ok(AssetHandle::from_index(index))
    }

    pub fn get_sound(&self, handle: AssetHandle) -> Option<&Sound> {
        self.sounds.get(handle.unwrap())
    }

    pub fn get_sound_mut(&mut self, handle: AssetHandle) -> Option<&mut Sound> {
        self.sounds.get_mut(handle.unwrap())
    }

    pub fn cleanup(&mut self){
        self.sounds.clear();
    }

    fn inner(&self) -> MutexGuard<KiraAudioManager> {
        self.manager.lock()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        AudioManager::new(128, 8).expect("Cannot create audio manager")
    }
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
         .field("sounds", &self.sounds)
         .finish()
    }
}

impl<'de> Deserialize<'de> for AudioManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum AudioManagerField { 
            Sounds,
            CastCount,
            ListenerCount,
        }

        struct AudioManagerVisitor;

        impl<'de> Visitor<'de> for AudioManagerVisitor {
            type Value = AudioManager;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct AudioManager")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<AudioManager, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let sounds: Vec<Sound> = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;
                let cast_count: usize = seq.next_element()?.ok_or_else(|| DeError::invalid_length(1, &self))?;
                let listener_count: usize = seq.next_element()?.ok_or_else(|| DeError::invalid_length(2, &self))?;

                let mut audio_manager = AudioManager::new(cast_count, listener_count).expect("Cannot create audio manager");
                audio_manager.sounds.extend(sounds);

                Ok(audio_manager)
            }

            fn visit_map<V>(self, mut map: V) -> Result<AudioManager, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut sounds: Option<Vec<Sound>> = None;
                let mut cast_count: Option<usize> = None;
                let mut listener_count: Option<usize> = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        AudioManagerField::Sounds => {
                            if sounds.is_some() {
                                return Err(DeError::duplicate_field("sounds"));
                            }
                            sounds = Some(map.next_value()?);
                        },
                        AudioManagerField::CastCount => {
                            if cast_count.is_some() {
                                return Err(DeError::duplicate_field("cast_count"));
                            }
                            cast_count = Some(map.next_value()?);
                        },
                        AudioManagerField::ListenerCount => {
                            if listener_count.is_some() {
                                return Err(DeError::duplicate_field("listener_count"));
                            }
                            listener_count = Some(map.next_value()?);
                        },
                    }
                }

                let sounds = sounds.ok_or_else(|| DeError::missing_field("sounds"))?;
                let cast_count = cast_count.ok_or_else(|| DeError::missing_field("cast_count"))?;
                let listener_count = listener_count.ok_or_else(|| DeError::missing_field("listener_count"))?;

                let mut audio_manager = AudioManager::new(cast_count, listener_count).expect("Cannot create audio manager");
                audio_manager.sounds.extend(sounds);

                Ok(audio_manager)
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "sounds", 
            "cast_count", 
            "listener_count",
        ];

        deserializer.deserialize_struct("AudioManager", FIELDS, AudioManagerVisitor)
    }
}