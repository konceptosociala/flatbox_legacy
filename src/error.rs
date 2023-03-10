use thiserror::Error;
use crate::physics::*;

#[derive(Debug, Error)]
pub enum Result {
    #[error("Allocation error")]
    AllocationError(#[from] gpu_allocator::AllocationError),
    #[error("Error processing image")]
    ImageError(#[from] image::ImageError),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Physics error")]
    PhysicsError(#[from] PhysicsError),
    #[error("Rendering error")]
    RenderError(#[from] ash::vk::Result),
    #[error("RON error")]
    RonError(#[from] ron::Error),
    
    #[error("Error happened: {0}")]
    CustomError(String),
}

pub type DesperoResult<T> = std::result::Result<T, Result>;
