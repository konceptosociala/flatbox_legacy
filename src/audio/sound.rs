use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sound {

}

impl Sound {
    pub fn new() -> Self {
        Sound {}
    }
}