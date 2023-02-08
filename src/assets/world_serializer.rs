use crate::ecs::*;
use crate::error::DesperoResult;

pub trait WorldSerializer {
    fn save<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        world: &World,
    ) -> DesperoResult<()>;
}

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
        
        impl WorldSerializer for $ctx {
            fn save<P: AsRef<std::path::Path>>(
                &mut self,
                path: P,
                world: &World,
            ) -> DesperoResult<()> {
                let mut buf = std::fs::File::create(path)?;
                let mut ser = ron::Serializer::new(buf, None)?;
                
                serialize_world(
                    &world,
                    self,
                    &mut ser,
                )?;
                
                Ok(())
            }
        }
    };
}
