use thiserror::Error;
use crate::physics::error::PhysicsError;
use crate::audio::error::AudioError;
#[cfg(feature = "gltf")]
use crate::render::pbr::gltf::GltfError;

/// Main universal error handler. Use [`Result::CustomError`] variant, if it doesn't fit your needs
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Result {
    /// Error during invalid creating or accessing memory allocation
    #[cfg(feature = "render")]
    #[error("Allocation error")]
    AllocationError(#[from] gpu_allocator::AllocationError),

    /// Error during loading, decoding or encoding image data
    #[cfg(feature = "render")]
    #[error("Error processing image")]
    ImageError(#[from] image::ImageError),

    /// Internal Vulkan error. Occurs when some Vulkan object is incorrectly created or invalid operation is performed
    #[cfg(feature = "render")]
    #[error("Rendering error")]
    RenderError(#[from] ash::vk::Result),

    /// Error during loading or processing glTF scenes
    #[cfg(feature = "gltf")]
    #[error("Error processing glTF asset")]
    GltfError(#[from] GltfError),

    /// Error during audio playback/instantiating/handling
    #[error("Error during processing audio")]
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
    Custom(String),
}

impl Result {
    pub fn yell<T: Into<String>>(msg: T) -> Self {
        Result::Custom(msg.into())
    }
}

/// Alias for easy error handling
pub type FlatboxResult<T> = std::result::Result<T, Result>;
