use nalgebra as na;
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: na::Vector3<f32>,
    pub illuminance: [f32; 3],
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub position: na::Point3<f32>,
    pub luminous_flux: [f32; 3],
}
