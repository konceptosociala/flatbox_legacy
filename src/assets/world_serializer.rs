use crate::ecs::*;
use crate::error::DesperoResult;

pub trait WorldSerializer {
    fn save<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        world: &World,
    ) -> DesperoResult<()>;
    
    fn load<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        world: &mut World,
    ) -> DesperoResult<()>;
}

// TODO: WorldSerializer description
#[macro_export]
macro_rules! world_serializer {
    ($ctx:ident, $($comp:ty),*) => {
        impl SerializeContext for $ctx {
            fn component_count(&self, archetype: &Archetype) -> usize {                
                archetype.component_types()
                    .filter(|&t|
                        $(
                            t == std::any::TypeId::of::<$comp>() ||
                        )*
                        false
                    )
                    .count()
            }
            
            fn serialize_component_ids<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                    try_serialize_id::<$comp, _, _>(archetype, stringify!($comp), &mut out)?;
                )*
                
                out.end()
            }
            
            fn serialize_components<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                    try_serialize::<$comp, _>(archetype, &mut out)?;
                )*
                
                out.end()
            }
        }
        
        impl DeserializeContext for $ctx {
            fn deserialize_component_ids<'de, A: serde::de::SeqAccess<'de>>(
                &mut self,
                mut seq: A,
            ) -> Result<ColumnBatchType, A::Error> {
                self.components.clear();
                let mut batch = ColumnBatchType::new();
                while let Some(id) = seq.next_element::<String>()? {
                    match id.as_str() {                        
                        $(                            
                            stringify!($comp) => {
                                batch.add::<$comp>();
                            }
                        )*
                        
                        _ => {},
                    }
                    self.components.push(id);
                }
                
                Ok(batch)
            }
            
            fn deserialize_components<'de, A: serde::de::SeqAccess<'de>>(
                &mut self,
                entity_count: u32,
                mut seq: A,
                batch: &mut ColumnBatchBuilder,
            ) -> Result<(), A::Error> {
                for component in &self.components {
                    match component.as_str() {
                        $(                            
                            stringify!($comp) => {
                                deserialize_column::<$comp, _>(entity_count, &mut seq, batch)?;
                            }
                        )*
                        
                        _ => {},
                    }
                }
                
                Ok(())
            }

        }
        
        impl WorldSerializer for $ctx {
            fn save<P: AsRef<std::path::Path>>(
                &mut self,
                path: P,
                world: &World,
            ) -> DesperoResult<()> {
                let buf = std::fs::File::create(path)?;                    
                let mut ser = ron::Serializer::new(buf, Some(ron::ser::PrettyConfig::new()))?;                
                
                serialize_world(
                    &world,
                    self,
                    &mut ser,
                )?;
                                
                Ok(())
            }
            
            fn load<P: AsRef<std::path::Path>>(
                &mut self,
                path: P,
                world: &mut World,
            ) -> DesperoResult<()> {
                let buf = std::fs::read_to_string(path)?;
                let mut de = ron::Deserializer::from_str(&buf)
                    .map_err(|e| ron::Error::from(e))?;
                
                *world = deserialize_world(                
                    self,
                    &mut de,
                ).map_err(|e| ron::Error::from(e))?; // TODO: Serde AssetManager
                
                Ok(())
            }
        }
    };
}
