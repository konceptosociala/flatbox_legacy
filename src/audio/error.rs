use thiserror::Error;
use kira::manager::{
    backend::cpal::Error as CpalError, 
    error::AddSpatialSceneError
};

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("W")]
    CpalError(#[from] CpalError),
    #[error("W")]
    AddSpatialSceneError(#[from] AddSpatialSceneError),
}