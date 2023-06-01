use thiserror::Error;
use kira::{
    manager::{
        backend::cpal::Error as CpalError, 
        error::{AddSpatialSceneError, AddClockError, AddModulatorError, AddSubTrackError, PlaySoundError as KiraPlaySoundError}
    }, 
    sound::FromFileError, CommandError
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Error using the cpal backend")]
    CpalError(#[from] CpalError),
    #[error("Error creating a clock")]
    AddClockError(#[from] AddClockError),
    #[error("Error creating a modulator")]
    AddModulatorError(#[from] AddModulatorError),
    #[error("Error creating a spatial scene")]
    AddSpatialSceneError(#[from] AddSpatialSceneError),
    #[error("Error creating a mixer sub-track")]
    AddSubTrackError(#[from] AddSubTrackError),
    #[error("Error loading or streaming an audio file")]
    FromFileError(#[from] FromFileError),
    #[error("Error sending a command to the audio thread")]
    CommandError(#[from] CommandError),
    #[error("Audio playback error: {0}")]
    PlaySoundError(String),
}

impl<T> From<KiraPlaySoundError<T>> for AudioError {
    fn from(err: KiraPlaySoundError<T>) -> Self {
        let content = match err {
			KiraPlaySoundError::SoundLimitReached => String::from(
				"Could not play a sound because the maximum number of sounds has been reached.",
			),
			KiraPlaySoundError::IntoSoundError(_) => String::from(
                "An error occurred when initializing the sound."
            ),
			KiraPlaySoundError::CommandError(error) => error.to_string(),
            _ => String::from("Unknown playback error")
		};

        AudioError::PlaySoundError(content)
    }
}