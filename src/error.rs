use thiserror::Error;

#[derive(Debug, Error)]
pub enum Desperror {
	#[error("Error handling material: {0}")]
	MaterialError(String),
}
