use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BodyHandle(
    pub(crate) RigidBodyHandle,
    pub(crate) ColliderHandle
);

impl BodyHandle {
    pub fn new() -> Self {
        BodyHandle::default()
    }
}
