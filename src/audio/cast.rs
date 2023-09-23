use kira::spatial::emitter::EmitterHandle;
use kira::tween::Tween;

use crate::error::FlatboxResult;
use crate::audio::{
    error::AudioError,
    AudioManager,
};
use crate::math::transform::Transform;

pub struct AudioCast {
    pub(crate) handle: EmitterHandle,
}

impl AudioCast {
    pub fn new(
        audio_manager: &mut AudioManager,
    ) -> Self {
        audio_manager.new_cast()
    }

    pub(crate) fn set_transform(&mut self, t: &Transform) -> FlatboxResult<()> {
        self.handle.set_position(t.translation, Tween::default()).map_err(|e| AudioError::from(e))?;

        Ok(())
    }
}

impl std::fmt::Debug for AudioCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AudioCast")
         .field(&self.handle.id())
         .finish()
    }
}