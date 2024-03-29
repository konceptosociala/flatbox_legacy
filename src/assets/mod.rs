pub mod asset_manager;
pub mod scene;
pub mod ser_component;
pub mod save_load;

pub use asset_manager::*;
pub use scene::*;
pub use ser_component::*;
pub use save_load::*;

pub mod tar {
    pub use tar::*;
}

pub mod lz4 {
    pub use lz4::*;
}

pub use typetag;

use serde::{Serialize, Deserialize};

#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetHandle<const TYPE: char>(usize);

impl<const TYPE: char> AssetHandle<TYPE> {
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

#[cfg(feature = "render")]
impl AssetHandle<'T'> {
    pub const BUILTIN_ALBEDO: AssetHandle::<'T'> = AssetHandle(0);
    pub const BUILTIN_ROUGHNESS: AssetHandle::<'T'> = AssetHandle(0);
    pub const BUILTIN_METALLIC: AssetHandle::<'T'> = AssetHandle(0);
    pub const BUILTIN_AO: AssetHandle::<'T'> = AssetHandle(0);
    pub const BUILTIN_NORMAL: AssetHandle::<'T'> = AssetHandle(1);
}

impl<const TYPE: char> From<AssetHandle<TYPE>> for u32 {
    fn from(value: AssetHandle<TYPE>) -> Self {
        value.unwrap() as u32
    }
}