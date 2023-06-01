use kira::spatial::emitter::EmitterHandle;
use kira::tween::Tween;

use crate::error::DesperoResult;
use crate::audio::{
    error::AudioError,
    AudioManager, AudioSceneId,
};
use crate::math::transform::Transform;

pub struct AudioCast {
    pub(crate) handle: EmitterHandle,
}

impl AudioCast {
    pub fn new(
        audio_manager: &mut AudioManager,
        scene_id: AudioSceneId,
    ) -> Self {
        audio_manager.new_cast(scene_id)
    }

    pub(crate) fn set_transform(&mut self, t: &Transform) -> DesperoResult<()> {
        self.handle.set_position(t.translation, Tween::default()).map_err(|e| AudioError::from(e))?;

        Ok(())
    }
}