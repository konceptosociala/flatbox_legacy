use std::fmt;
use std::path::PathBuf;
use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundSettings}, 
    spatial::emitter::EmitterId
};
use serde::{
    Serialize, Deserialize, 
    de::{
        Visitor,
        SeqAccess,
        MapAccess,
        Error as DeError,
    },
};

use crate::error::FlatboxResult;

use super::{
    AudioError, 
    cast::AudioCast
};

#[derive(Debug, Clone, Serialize)]
pub struct Sound {
    pub(crate) path: PathBuf,

    #[serde(skip_serializing)]
    pub(crate) cast_id: Option<EmitterId>,
    #[serde(skip_serializing)]
    pub(crate) static_data: StaticSoundData,
}

impl Sound {
    pub fn new_from_file(path: &'static str) -> FlatboxResult<Self> {
        let static_data = StaticSoundData::from_file(
            path, 
            StaticSoundSettings::default()
        ).map_err(|e| AudioError::from(e))?;

        Ok(Sound {
            path: path.into(),
            cast_id: None,
            static_data,
        })
    }

    pub(crate) fn set_cast(&mut self, cast: &AudioCast) {
        let id = Some(cast.handle.id());
        if self.cast_id == id {
            return;
        }

        let settings = StaticSoundSettings::new().output_destination(&cast.handle);
        let new_data = self.static_data.clone().with_settings(settings);

        self.static_data = new_data;
        self.cast_id = id;
    }
}

impl<'de> Deserialize<'de> for Sound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum SoundField { Path }

        struct SoundVisitor;

        impl<'de> Visitor<'de> for SoundVisitor {
            type Value = Sound;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Sound")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Sound, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let path: PathBuf = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;

                let static_data = StaticSoundData::from_file(
                    path.clone(), 
                    StaticSoundSettings::default()
                ).map_err(|e| AudioError::from(e)).expect("Cannot deserialize audio with path");

                Ok(Sound {
                    path,
                    cast_id: None,
                    static_data,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Sound, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut path: Option<PathBuf> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        SoundField::Path => {
                            if path.is_some() {
                                return Err(DeError::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                    }
                }
                let path = path.ok_or_else(|| DeError::missing_field("path"))?;
                
                let static_data = StaticSoundData::from_file(
                    path.clone(), 
                    StaticSoundSettings::default()
                ).map_err(|e| AudioError::from(e)).expect("Cannot deserialize audio with path");

                Ok(Sound {
                    path,
                    cast_id: None,
                    static_data,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["path"];
        deserializer.deserialize_struct("Sound", FIELDS, SoundVisitor)
    }
}