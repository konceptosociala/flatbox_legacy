use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use serde::{Serialize, Deserialize, Deserializer};

use crate::prelude::DesperoResult;

use super::{AudioError, cast::AudioCast};

#[derive(Debug, Clone, Serialize)]
pub struct Sound {
    path: String,

    #[serde(skip_serializing)]
    pub(crate) static_data: StaticSoundData,
}

impl Sound {
    pub fn new_from_file(path: &'static str) -> DesperoResult<Self> {
        let static_data = StaticSoundData::from_file(
            path, 
            StaticSoundSettings::default()
        ).map_err(|e| AudioError::from(e))?;

        Ok(Sound {
            path: path.into(),
            static_data,
        })
    }

    pub fn set_cast(&mut self, cast: &AudioCast) {
        let settings = StaticSoundSettings::new().output_destination(&cast.handle);
        let new_data = self.static_data.with_settings(settings);
        self.static_data = new_data;
    }
}

impl<'de> Deserialize<'de> for Sound {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        todo!();
        // TODO: Implement Deserialize
    }
}