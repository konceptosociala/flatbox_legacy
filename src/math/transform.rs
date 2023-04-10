use serde::{Serialize, Deserialize};
use nalgebra::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: f32,
}

impl Default for Transform {
    fn default() -> Self{
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        }
    }
}

impl Transform {
    pub fn new(
        translation: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        scale: f32,
    ) -> Self {
        Transform {
            translation,
            rotation,
            scale,
        }
    }
    
    pub fn to_matrices(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let new_matrix = 
            Matrix4::new_translation(&Vector3::new(
                self.translation.x,
                self.translation.y,
                self.translation.z,
            ))
            * Matrix4::from(self.rotation)
            * Matrix4::from([
                [self.scale, 0.0, 0.0, 0.0],
                [0.0, self.scale, 0.0, 0.0],
                [0.0, 0.0, self.scale, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]);
        
        (new_matrix, new_matrix.try_inverse().unwrap())
    }
    
    pub fn from_translation(translation: Vector3<f32>) -> Self {
        Transform {
            translation,
            rotation: UnitQuaternion::identity(),
            scale: 1.0,
        }
    }

    pub fn from_rotation(rotation: UnitQuaternion<f32>) -> Self {
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation,
            scale: 1.0,
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
