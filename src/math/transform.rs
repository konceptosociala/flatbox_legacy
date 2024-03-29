use serde::{Serialize, Deserialize};
use nalgebra::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Scale3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vector3::identity(),
            rotation: UnitQuaternion::identity(),
            scale: Scale3::identity(),
        }
    }
}

impl Transform {
    pub fn new(
        translation: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        scale: Scale3<f32>,
    ) -> Self {
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self::default()
    }
    
    pub fn to_matrices(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let new_matrix = 
            Matrix4::new_translation(&Vector3::new(
                self.translation.x,
                -self.translation.y,
                self.translation.z,
            ))
            * Matrix4::from(self.rotation)
            * Matrix4::from(self.scale);
        
        (new_matrix, new_matrix.try_inverse().unwrap())
    }

    pub fn from_scale(scale: Scale3<f32>) -> Self {
        Transform { 
            translation: Vector3::identity(), 
            rotation: UnitQuaternion::identity(), 
            scale,
        }
    }
    
    pub fn from_translation(translation: Vector3<f32>) -> Self {
        Transform {
            translation,
            rotation: UnitQuaternion::identity(),
            scale: Scale3::identity(),
        }
    }

    pub fn from_rotation(rotation: UnitQuaternion<f32>) -> Self {
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation,
            scale: Scale3::identity(),
        }
    }
    
    pub fn local_x(&self) -> Unit<Vector3<f32>> {
        let m = self.to_matrices().0;
        
        Unit::new_normalize(Vector3::new(
            m[(0, 0)],
            m[(0, 1)], 
            m[(0, 2)]
        ))
    }
    
    pub fn local_y(&self) -> Unit<Vector3<f32>> {
        let m = self.to_matrices().0;
        
        Unit::new_normalize(Vector3::new(
            m[(1, 0)],
            m[(1, 1)], 
            m[(1, 2)]
        ))
    }
    
    pub fn local_z(&self) -> Unit<Vector3<f32>> {
        let m = self.to_matrices().0;
        
        Unit::new_normalize(Vector3::new(
            m[(2, 0)],
            m[(2, 1)], 
            m[(2, 2)]
        ))
    }
}
