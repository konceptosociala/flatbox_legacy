use crate::ecs::*;
use crate::error::DesperoResult;
use crate::assets::asset_manager::AssetManager;
use crate::physics::physics_handler::PhysicsHandler;

pub trait SaveLoad {
    fn save<P: AsRef<std::path::Path>>(
        &mut self,
        world: &World,
        asset_manager: &AssetManager,
        physics_handler: &PhysicsHandler,
        path: P,
    ) -> DesperoResult<()>;
    
    fn load<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> DesperoResult<(World, AssetManager, PhysicsHandler)>;
}

/// Macro that is used to create custom [`SaveLoad`]ers, 
/// that are capable of saving and loading individual serializable
/// components from the [`World`], scene's [`PhysicsHandler`] and 
/// [`AssetManager`]
/// 
/// # Usage example
/// 
/// ```rust 
/// #[derive(Serialize, Deserialize)]
/// struct MyComponent(u32);
/// 
/// #[derive(Default)]
/// struct MySaveLoader {
///     components: Vec<String>, // required field
/// }
/// 
/// impl_save_load!(
///     // Serializer struct
///     MySaveLoader,
///         // Serializable components
///         Camera,
///         Timer,
///         Transform,
///         MyComponent,
/// );
/// 
/// fn save_world(
///     world: Read<World>,
/// ) -> DesperoResult<()> {
///     let ws = MyWorldSaver::default();
/// 
///     ws.save("/path/to/save", &world)?;
/// }
/// 
/// ```
#[macro_export]
macro_rules! impl_save_load {
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
        
        impl SaveLoad for $ctx {
            fn save<P: AsRef<std::path::Path>>(
                &mut self,
                world: &World,
                asset_manager: &AssetManager,
                physics_handler: &PhysicsHandler,
                path: P,
            ) -> DesperoResult<()> {
                use std::mem::size_of_val;
                use std::fs::File;
                use std::io::Cursor;
                use ron::ser::PrettyConfig;

                let mut buf = vec![];                    
                let mut ser = ron::Serializer::new(&mut buf, Some(PrettyConfig::new()))?;          
                serialize_world(&world, self, &mut ser)?;

                let mut a = vec![];
                let mut archive = tar::Builder::new(&mut a);

                let world = &*buf;
                let world_header = create_header("world.ron", world.len());
                archive.append(&world_header, world)?;

                let assets = ron::ser::to_string_pretty(&asset_manager, PrettyConfig::default())?;
                let assets_bytes = assets.as_bytes();
                let assets_header = create_header("assets.ron", assets_bytes.len());
                archive.append(&assets_header, assets_bytes)?;

                let physics = ron::ser::to_string_pretty(&physics_handler, PrettyConfig::default())?;
                let physics_bytes = physics.as_bytes();
                let physics_header = create_header("physics.ron", physics_bytes.len());
                archive.append(&physics_header, physics_bytes)?;

                let mut inner = archive.into_inner()?;
                let mut cursor = Cursor::new(inner);

                let file = File::create(path)?;
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(4)
                    .build(file)?;  

                std::io::copy(&mut cursor, &mut encoder)?; 

                let (_, result) = encoder.finish();
                
                result?;

                Ok(())
            }
            
            fn load<P: AsRef<std::path::Path>>(
                &mut self,
                path: P,
            ) -> DesperoResult<(World, AssetManager, PhysicsHandler)> {
                use std::fs::File;
                use std::io::Read;

                let package = File::open(path)?;
                let decoded = lz4::Decoder::new(package)?;
                let mut archive = tar::Archive::new(decoded);

                let mut world = None;
                let mut asset_manager = None;
                let mut physics_handler = None;

                for file in archive.entries().unwrap() {
                    let mut file = file.unwrap();
                    let header = file.header().clone();

                    let mut buffer = vec![];
                    file.read_to_end(&mut buffer)?;
                    let mut de = ron::Deserializer::from_bytes(&buffer)
                        .map_err(|e| ron::Error::from(e))?;
                    
                    if header.entry_type() == tar::EntryType::Regular {
                        match header.path().unwrap().to_str().unwrap() {
                            "world.ron" => {
                                world = Some(deserialize_world(self, &mut de)
                                    .map_err(|e| ron::Error::from(e))?);
                            },
                            "assets.ron" => {
                                asset_manager = Some(AssetManager::deserialize(&mut de)?);
                            },
                            "physics.ron" => {
                                physics_handler = Some(PhysicsHandler::deserialize(&mut de)?);
                            },
                            _ => {},
                        }
                    }
                }
                
                Ok((
                    world.unwrap(), 
                    asset_manager.unwrap(), 
                    physics_handler.unwrap()
                ))
            }
        }

        fn create_header(path: &'static str, size: usize) -> tar::Header {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_path(path).unwrap();
            header.set_size(size as u64);
            header.set_cksum();

            header
        }
    };
}
