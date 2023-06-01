use std::sync::Arc;
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

use crate::prelude::DesperoResult;

pub mod cast;
pub mod error;
pub mod listener;
pub mod sound;

pub use cast::*;
pub use error::*;
pub use listener::*;
pub use sound::*;
pub use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

type KiraAudioManager = kira::manager::AudioManager<CpalBackend>; 

// TODO: Better organization (AssetManager & AudioManager ??)
pub struct AudioManager {
    manager: Arc<Mutex<KiraAudioManager>>,
    scenes: Vec<SpatialSceneHandle>,
}

impl AudioManager {
    pub fn new() -> DesperoResult<Self> {
        let mut manager = KiraAudioManager::new(AudioManagerSettings::default()).map_err(|e| AudioError::from(e))?;
        let scene = manager.add_spatial_scene(SpatialSceneSettings::default()).map_err(|e| AudioError::from(e))?;
        
        Ok(AudioManager { 
            manager: Arc::new(Mutex::new(manager)),
            scenes: vec![scene],
        })
    }

    pub fn play(&mut self, sound: Sound){
        self.inner().play(sound.static_data).expect("Cannot play sound");
    }

    pub fn new_cast(
        &mut self, 
        scene_id: AudioSceneId
    ) -> AudioCast {
        let index = match scene_id {
            AudioSceneId::Main => 0,
            AudioSceneId::Additional(i) => i,
        };

        let scene = self.scenes.get_mut(index).expect("Audio scene with this index not found");
        let handle = scene.add_emitter(
            Vector3::identity(),
            EmitterSettings::default(),
        ).expect("Cannot create audio cast");

        AudioCast { handle }
    }

    pub fn new_listener(
        &mut self,
        scene_id: AudioSceneId,
    ) -> AudioListener {
        let index = match scene_id {
            AudioSceneId::Main => 0,
            AudioSceneId::Additional(i) => i,
        };

        let scene = self.scenes.get_mut(index).expect("Audio scene with this index not found");
        let handle = scene.add_listener(
            Vector3::identity(),
            Quaternion::identity(),
            ListenerSettings::default(),
        ).expect("Cannot create audio cast");

        AudioListener { handle }
    }

    pub fn new_spatial_scene(
        &mut self,
        cast_count: usize,
        listener_count: usize,
    ) -> DesperoResult<usize> {
        let index = self.scenes.len();
        let settings = SpatialSceneSettings::new()
            .emitter_capacity(cast_count)
            .listener_capacity(listener_count);

        let scene = self.inner().add_spatial_scene(settings).map_err(|e| AudioError::from(e))?;
        self.scenes.push(scene);

        Ok(index)
    }

    fn inner(&self) -> MutexGuard<KiraAudioManager> {
        self.manager.lock()
    }
}

pub enum AudioSceneId {
    Main,
    Additional(usize),
}