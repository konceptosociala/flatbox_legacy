use serde::{Serialize, Deserialize};
use despero::world_serializer;
use despero::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct WorldSaver;

impl WorldSaver {
    pub fn new() -> Self {
        WorldSaver
    }
}

world_serializer!(
    WorldSaver, 
        Mesh, 
        Transform, 
        MaterialHandle
);
