use thiserror::Error;
use crate::physics::error::PhysicsError;
use crate::audio::error::AudioError;

/// Main universal error handler. Use [`Result::CustomError`] variant, if it doesn't fit your needs
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Result {
    /// Error during invalid creating or accessing memory allocation
    #[cfg(feature = "render")]
    #[error("Allocation error")]
    AllocationError(#[from] gpu_allocator::AllocationError),

    /// Error while loading, decoding or encoding image data
    #[cfg(feature = "render")]
    #[error("Error processing image")]
    ImageError(#[from] image::ImageError),

    /// Internal Vulkan error. Occurs when some Vulkan object is incorrectly created or invalid operation is performed
    #[cfg(feature = "render")]
    #[error("Rendering error")]
    RenderError(#[from] ash::vk::Result),

    /// Error during audio playback/instantiating/handling
    #[error("Error while processing audio")]
    AudioError(#[from] AudioError),

    /// Deserialization RON-object error
    #[error("Deserialization error")]
    DeserializationError(#[from] ron::error::SpannedError),
    
    /// Standard input/output error
    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    /// Error accessing physics components. It's often caused by invalid body handle
    #[error("Physics error")]
    PhysicsError(#[from] PhysicsError),
    
    /// Extended RON error
    #[error("RON error")]
    RonError(#[from] ron::Error),
    
    /// Custom error type. Use when other error types don't fit
    #[error("Error happened: {0}")]
    CustomError(String),
}

impl From<&str> for Result {
    fn from(msg: &str) -> Self {
        Result::CustomError(String::from(msg))
    }
}

impl From<String> for Result {
    fn from(msg: String) -> Self {
        Result::CustomError(msg)
    }
}

/// Alias for easy error handling
pub type DesperoResult<T> = std::result::Result<T, Result>;
