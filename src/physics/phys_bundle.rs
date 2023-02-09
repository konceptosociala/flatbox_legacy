use hecs::{Bundle};
use rapier3d::prelude::{ColliderHandle, RigidBodyHandle};

#[derive(Bundle, Default)]
pub struct PhysBundle {
    pub rigidbody: RigidBodyHandle,
    pub collider: ColliderHandle,
}

impl PhysBundle {
    pub fn new() -> Self {
        PhysBundle::default()
    }
    
    pub fn builder() -> PhysBundleBuilder {
        PhysBundleBuilder::new()
    }
}

pub struct PhysBundleBuilder {
    rigidbody: RigidBodyHandle,
    collider: ColliderHandle,
}

impl PhysBundleBuilder {
    pub fn new() -> Self {
        PhysBundleBuilder {
            rigidbody: RigidBodyHandle::default(),
            collider: ColliderHandle::default(),
        }
    }
    
    pub fn rigidbody(mut self, rigidbody: RigidBodyHandle) -> Self {
        self.rigidbody = rigidbody;
        self
    }
    
    pub fn collider(mut self, collider: ColliderHandle) -> Self {
        self.collider = collider;
        self
    }
    
    pub fn build(self) -> PhysBundle {
        PhysBundle {
            rigidbody: self.rigidbody,
            collider: self.collider,
        }
    }
}
