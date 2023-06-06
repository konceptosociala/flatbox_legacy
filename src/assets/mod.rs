pub mod asset_manager;
pub mod scene;
pub mod ser_component;
pub mod settings;
pub mod world_serializer;

pub use asset_manager::*;
pub use scene::*;
pub use ser_component::*;
pub use settings::*;
pub use world_serializer::*;

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AssetLoadType {
    Resource(String),
    Path(PathBuf),
}

impl Default for AssetLoadType {
    fn default() -> Self {
        AssetLoadType::Resource("".into())
    }
}

#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetHandle(usize);

impl AssetHandle {
    pub fn new() -> Self {
        AssetHandle::default()
    }
    
    pub fn from_index(index: usize) -> Self {
        AssetHandle(index)
    }
    
    pub fn invalid() -> Self {
        AssetHandle(usize::MAX)
    }
    
    pub fn unwrap(&self) -> usize {
        self.0
    }
    
    pub fn append(&mut self, count: usize) {
        self.0 += count;
    }
}

impl From<AssetHandle> for u32 {
    fn from(value: AssetHandle) -> Self {
        value.unwrap() as u32
    }
}