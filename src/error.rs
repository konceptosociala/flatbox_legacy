use thiserror::Error;

#[derive(Debug, Error)]
pub enum Result {
	#[error("Rendering error")]
	RenderError(#[from] ash::vk::Result),
	#[error("Allocation error")]
	AllocationError(#[from] gpu_allocator::AllocationError),
	#[error("Error processing image")]
	ImageError(#[from] image::ImageError),
	#[error("Error happened: {0}")]
	CustomError(String),
}

pub type DesperoResult<T> = std::result::Result<T, Result>;
