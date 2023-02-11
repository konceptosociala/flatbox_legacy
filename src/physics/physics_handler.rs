use serde::{Serialize, Deserialize};
use nalgebra::Vector3;
use rapier3d::prelude::*;

use crate::error::DesperoResult;
use super::error::*;

/// Collection for physics simulations
#[derive(Serialize, Deserialize)]
pub struct PhysicsHandler {
    rigidbody_set: RigidBodySet,
    collider_set: ColliderSet,
    
    pub gravity: Vector3<f32>,
    pub integration_parameters: IntegrationParameters,
    #[serde(skip_serializing, skip_deserializing)]
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl PhysicsHandler {
    /// Create new [`PhysicsHandler`] instance
    pub fn new() -> Self {
        PhysicsHandler::default()
    }
    
    /// Add RigidBody to set
    pub fn add_rigidbody(&mut self, rb: RigidBody) -> RigidBodyHandle {
        self.rigidbody_set.insert(rb)
    }
    /// Add Collider to set
    pub fn add_collider(&mut self, col: Collider) -> ColliderHandle {
        self.collider_set.insert(col)
    }
    
    /// Get RigidBody from set
    pub fn rigidbody(&mut self, handle: RigidBodyHandle) -> DesperoResult<&RigidBody> {
        match self.rigidbody_set.get(handle){
            Some(rb) => Ok(rb),
            None => Err(PhysicsError::InvalidRigidBody.into()),
        }
    }
    
    /// Get Collider from set
    pub fn collider(&mut self, handle: ColliderHandle) -> DesperoResult<&Collider> {
        match self.collider_set.get(handle){
            Some(col) => Ok(col),
            None => Err(PhysicsError::InvalidCollider.into()),
        }
    }
    
    /// Mutably get RigidBody from set
    pub fn rigidbody_mut(&mut self, handle: RigidBodyHandle) -> DesperoResult<&mut RigidBody> {
        match self.rigidbody_set.get_mut(handle){
            Some(rb) => Ok(rb),
            None => Err(PhysicsError::InvalidRigidBody.into()),
        }
    }
    
    /// Mutably get Collider from set
    pub fn collider_mut(&mut self, handle: ColliderHandle) -> DesperoResult<&mut Collider> {
        match self.collider_set.get_mut(handle){
            Some(col) => Ok(col),
            None => Err(PhysicsError::InvalidCollider.into()),
        }
    }
    
    /// Does a physical simulations step. Run in a loop
    pub fn step(&mut self){
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbody_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        )
    }
    
    /// Remove RigidBody from set
    pub(crate) fn remove_rigidbody(&mut self, handle: RigidBodyHandle) -> DesperoResult<RigidBody> {
        match self.rigidbody_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            false,
        ){
            Some(rb) => Ok(rb),
            None => Err(PhysicsError::InvalidRigidBody.into()),
        }
    }
    
    /// Remove Collider from set
    pub(crate) fn remove_collider(&mut self, handle: ColliderHandle) -> DesperoResult<Collider> {
        match self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigidbody_set,
            false,
        ){
            Some(col) => Ok(col),
            None => Err(PhysicsError::InvalidCollider.into()),
        }
    }
    
    pub(crate) fn combine(
        &mut self,
        rb: RigidBodyHandle,
        col: ColliderHandle,
    ) -> DesperoResult<()> {
        let col = self.remove_collider(col)?;
        self.collider_set.insert_with_parent(col, rb, &mut self.rigidbody_set);
        Ok(())
    }
}

impl Default for PhysicsHandler {
    fn default() -> Self {
        PhysicsHandler {
            rigidbody_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }
}
