use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhysicsError {
    #[error("Invalid RigidBody handle provided")]
    InvalidRigidBody,
    #[error("Invalid Collider handle provided")]
    InvalidCollider,
}
