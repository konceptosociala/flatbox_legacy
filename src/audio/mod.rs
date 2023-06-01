use std::sync::Arc;
use kira::{ 
    spatial::scene::{
        SpatialSceneHandle, 
        SpatialSceneSettings
    }, 
    manager::{
        AudioManagerSettings,
        backend::cpal::CpalBackend,
    },
};
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

pub type KiraAudioManager = kira::manager::AudioManager<CpalBackend>; 

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

    /// TODO: Make private and use internally
    pub fn inner(&self) -> MutexGuard<KiraAudioManager> {
        self.manager.lock()
    }
}

// TODO: cast commands from `AudioCast` to AudioManager