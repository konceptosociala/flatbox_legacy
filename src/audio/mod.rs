use std::sync::Arc;
use kira::manager::{AudioManager as KiraAudioManager, AudioManagerSettings};
use parking_lot::{Mutex, MutexGuard};

pub mod cast;
pub mod listener;

pub use cast::*;
pub use listener::*;
pub use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

pub struct AudioManager {
    inner: Arc<Mutex<KiraAudioManager>>,
}

impl AudioManager {
    pub fn new() -> Self {
        AudioManager { 
            inner: Arc::new(Mutex::new(
                KiraAudioManager::new(AudioManagerSettings::default()).expect("Cannot create audio manager")
            ))
        }
    }

    pub fn inner(&self) -> MutexGuard<KiraAudioManager> {
        self.inner.lock()
    }
}

// TODO: cast commands from `AudioCast` to AudioManager