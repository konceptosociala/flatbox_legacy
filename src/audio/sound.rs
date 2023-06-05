use std::fmt;
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
use crate::assets::AssetLoadType;

use super::{AudioError, cast::AudioCast};

#[derive(Debug, Clone, Serialize)]
pub struct Sound {
    loader_type: AssetLoadType,

    #[serde(skip_serializing)]
    pub(crate) static_data: Option<StaticSoundData>,
}

impl Sound {
    pub fn new_from_file(path: &'static str) -> DesperoResult<Self> {
        let static_data = Some(
            StaticSoundData::from_file(
                path, 
                StaticSoundSettings::default()
            ).map_err(|e| AudioError::from(e))?
        );

        Ok(Sound {
            loader_type: path.into(),
            static_data,
        })
    }

    pub fn set_cast(&mut self, cast: &AudioCast) {
        let settings = StaticSoundSettings::new().output_destination(&cast.handle);
        let new_data = self.static_data.clone().unwrap().with_settings(settings);
        self.static_data = Some(new_data);
    }
}

impl<'de> Deserialize<'de> for Sound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum SoundField { LoaderType }

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
                let loader_type: AssetLoadType = seq.next_element()?.ok_or_else(|| DeError::invalid_length(0, &self))?;

                let static_data = match loader_type.clone() {
                    AssetLoadType::Path(path) => {
                        Some(
                            StaticSoundData::from_file(
                                path, 
                                StaticSoundSettings::default()
                            ).map_err(|e| AudioError::from(e)).expect("Cannot deserialize audio with path")
                        )
                    },
                    _ => None,
                };

                Ok(Sound {
                    loader_type,
                    static_data,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Sound, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut loader_type: Option<AssetLoadType> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        SoundField::LoaderType => {
                            if loader_type.is_some() {
                                return Err(DeError::duplicate_field("loader_type"));
                            }
                            loader_type = Some(map.next_value()?);
                        }
                    }
                }
                let loader_type = loader_type.ok_or_else(|| DeError::missing_field("loader_type"))?;
                
                let static_data = match loader_type.clone() {
                    AssetLoadType::Path(path) => {
                        Some(
                            StaticSoundData::from_file(
                                path, 
                                StaticSoundSettings::default()
                            ).map_err(|e| AudioError::from(e)).expect("Cannot deserialize audio with path")
                        )
                    },
                    _ => None,
                };

                Ok(Sound {
                    loader_type,
                    static_data,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["loader_type"];
        deserializer.deserialize_struct("Sound", FIELDS, SoundVisitor)
    }
}