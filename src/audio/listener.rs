use kira::{spatial::listener::ListenerHandle, tween::Tween};

use crate::{
    audio::{
        AudioManager, AudioSceneId,
        error::AudioError,
    },
    math::Transform, 
    error::DesperoResult,
};

pub struct AudioListener {
    pub(crate) handle: ListenerHandle,
}

impl AudioListener {
    pub fn new(
        audio_manager: &mut AudioManager,
        scene_id: AudioSceneId,
    ) -> Self {
        audio_manager.new_listener(scene_id)
    }

    pub(crate) fn set_transform(&mut self, t: &Transform) -> DesperoResult<()> {
        self.handle.set_position(t.translation, Tween::default()).map_err(|e| AudioError::from(e))?;
        self.handle.set_orientation(t.rotation, Tween::default()).map_err(|e| AudioError::from(e))?;

        Ok(())
    }
}

impl std::fmt::Debug for AudioListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AudioListener")
         .field(&self.handle.id())
         .finish()
    }
}