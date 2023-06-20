use kira::{spatial::listener::ListenerHandle, tween::Tween};

use crate::{
    audio::{
        AudioManager,
        error::AudioError,
    },
    math::Transform, 
    error::SonjaResult,
};

pub struct AudioListener {
    pub(crate) handle: ListenerHandle,
}

impl AudioListener {
    pub fn new(
        audio_manager: &mut AudioManager,
    ) -> Self {
        audio_manager.new_listener()
    }

    pub(crate) fn set_transform(&mut self, t: &Transform) -> SonjaResult<()> {
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