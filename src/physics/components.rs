use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use serde::{Serialize, Deserialize};

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct BodyHandle(
    pub(crate) RigidBodyHandle,
    pub(crate) ColliderHandle
);

impl BodyHandle {
    pub fn new() -> Self {
        BodyHandle::default()
    }
}
