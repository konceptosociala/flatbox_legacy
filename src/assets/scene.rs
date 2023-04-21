//~ use std::path::Path;
//~ use std::fs::read_to_string;
//~ use std::sync::Arc;

//~ use serde::{
    //~ Serializer, 
    //~ Serialize, 
    //~ Deserialize
//~ };

//~ use crate::ecs::*;
//~ use crate::error::*;

//~ pub struct SerializableEntity {
    //~ pub components: Vec<Arc<dyn Component + 'static>>
//~ }

//~ Components(
    //~ mesh: Mesh {...},
    //~ transform: Transform {...},
    //~ material: MaterialHandle(11),
//~ )




//~ #[derive(Default)]
//~ pub struct Scene {
    //~ pub entities: Vec<SerializableEntity>,
//~ }

//~ impl Scene {
    //~ pub fn load<P: AsRef<Path>>(path: P) -> DesperoResult<Self> {
        //~ let scene = read_to_string(path)?;
        
        //~ Ok(Scene {
            //~ entities: vec![],
        //~ })
    //~ }
//~ }

//~ impl Serialize for SerializableScene {
    //~ fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        //~ let mut state = serializer.serialize_struct("Scene", 2)?;
    //~ }
//~ }
