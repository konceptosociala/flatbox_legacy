//~ use std::path::Path;
//~ use std::fs::read_to_string;
//~ use erased_serde::{Serialize, Deserialize};

//~ use crate::ecs::*;
//~ use crate::error::*;

//~ pub trait LoadScene {
    //~ fn load_scene<P: AsRef<Path>>(&mut self, path: P) -> DesperoResult<()>;
//~ }

//~ impl LoadScene for CommandBuffer {
    //~ fn load_scene<P: AsRef<Path>>(&mut self, path: P) -> DesperoResult<()> {
        //~ let scene = read_to_string(path)?;
        //~ let objects: Scene = ron::from_str(scene)?;
        
        //~ Ok(())
    //~ }
//~ }
