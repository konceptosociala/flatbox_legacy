use thiserror::Error;

#[derive(Debug, Error)]
pub enum Result {
	#[error("Error handling material: {0}")]
	MaterialError(String),
	#[error("Rendering error")]
	RenderError(#[from] ash::vk::Result),
}

pub type DesperoResult<T> = std::result::Result<T, Result>;
