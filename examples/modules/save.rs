//~ use serde::{Serialize, Deserialize};
//~ use despero::world_serializer;
//~ use despero::prelude::*;

//~ #[derive(Default, Deserialize, Serialize)]
//~ pub struct WorldSaver {
    //~ components: Vec<String>,
//~ }

//~ impl WorldSaver {
    //~ pub fn new() -> Self {
        //~ WorldSaver::default()
    //~ }
//~ }

//~ world_serializer!(
    //~ WorldSaver, 
        //~ Mesh, 
        //~ Transform, 
        //~ MaterialHandle
//~ );
