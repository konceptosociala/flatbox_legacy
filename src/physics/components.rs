use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

#[derive(Default, Copy, Clone)]
pub struct BodyHandle(
    pub(crate) RigidBodyHandle,
    pub(crate) ColliderHandle
);

impl BodyHandle {
    pub fn new() -> Self {
        BodyHandle::default()
    }
}
