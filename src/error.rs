use thiserror::Error;

#[derive(Debug, Error)]
pub enum Result {
	#[error("Event error")]
	EventError(#[from] despero_ecs::EventError),
	#[error("Rendering error")]
	RenderError(#[from] ash::vk::Result),
	#[error("Allocation error")]
	AllocationError(#[from] gpu_allocator::AllocationError),
	#[error("Error processing image")]
	ImageError(#[from] image::ImageError),
	#[error("Error happened: {0}")]
	CustomError(&'static str),
}

pub type DesperoResult<T> = std::result::Result<T, Result>;
