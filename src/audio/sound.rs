use std::fmt;
use std::path::PathBuf;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use serde::{
    Serialize, Deserialize, 
    de::{
        Visitor,
        SeqAccess,
        MapAccess,
        Error as DeError,
    },
};

use crate::error::DesperoResult;

use super::{
    AudioError, 
    cast::AudioCast
};

#[derive(Debug, Clone, Serialize)]
pub struct Sound {
    pub(crate) path: PathBuf,

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
        let new_data = self.static_data.clone().with_settings(settings);
        self.static_data = new_data;
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
                    static_data,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["path"];
        deserializer.deserialize_struct("Sound", FIELDS, SoundVisitor)
    }
}