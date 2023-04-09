use ash::vk;
use nalgebra as na;

use crate::render::{
    renderer::Renderer,
    debug::*,
};

use crate::ecs::*;
use crate::math::transform::Transform;

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum CameraType {
    FirstPerson,
    LookAt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    camera_type: CameraType,
    projectionmatrix: na::Matrix4<f32>,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    is_active: bool,
}

impl Camera {
    pub fn new() -> Self {
        Camera::default()
    }
    
    pub fn builder() -> CameraBuilder {
        CameraBuilder {
            camera_type: CameraType::LookAt,
            fovy: std::f32::consts::FRAC_PI_3,
            aspect: 800.0 / 600.0,
            near: 0.1,
            far: 100.0,
            is_active: false,
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.is_active.clone()
    }
    
    
    pub fn camera_type(&self) -> CameraType {
        self.camera_type.clone()
    }
    
    pub fn set_active(&mut self, is_active: bool){
        self.is_active = is_active;
    }
    
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_projectionmatrix();
    }
    
    pub(crate) fn update_buffer(
        &self,
        renderer: &mut Renderer,
        transform: &Transform,
    ) -> Result<(), vk::Result>{     
        let mut rotation_matrix = na::Matrix4::<f32>::identity();        
        rotation_matrix *= na::Matrix4::from(transform.rotation);
        
        let mut translation = transform.translation;
        translation.y *= -1.0;
        let translation_matrix = na::Matrix4::identity().prepend_translation(&translation);
        
        let viewmatrix = {
            if self.camera_type == CameraType::FirstPerson {
                rotation_matrix * translation_matrix
            } else {
                translation_matrix * rotation_matrix
            }
        };
        
        let buffer_data: [[[f32; 4]; 4]; 2] = [
            viewmatrix.into(), 
            self.projectionmatrix.into()
        ];
        
        renderer.camera_buffer.fill(
            &renderer.device, 
            &mut *renderer.allocator.lock().unwrap(), 
            &buffer_data
        )?;
        
        Ok(())
    }
    
    fn update_projectionmatrix(&mut self) {
        let d = 1.0 / (0.5 * self.fovy).tan();
        self.projectionmatrix = na::Matrix4::new(
            d / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            d,
            0.0,
            0.0,
            0.0,
            0.0,
            self.far / (self.far - self.near),
            -self.near * self.far / (self.far - self.near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::builder().build()
    }
}

pub struct CameraBuilder {
    camera_type: CameraType,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    is_active: bool,
}

impl CameraBuilder {
    pub fn build(self) -> Camera {
        if self.far < self.near {
            error!("Far plane (at {}) is closer than near plane (at {})!", self.far, self.near);
        }
        
        let mut cam = Camera {
            camera_type: self.camera_type,
            fovy: self.fovy,
            aspect: self.aspect,
            near: self.near,
            far: self.far,
            projectionmatrix: na::Matrix4::identity(),
            is_active: self.is_active,
        };
        cam.update_projectionmatrix();
        cam
    }
    
    pub fn camera_type(mut self, camera_type: CameraType) -> CameraBuilder {
        self.camera_type = camera_type;
        self
    }
    
    pub fn fovy(mut self, fovy: f32) -> CameraBuilder {
        self.fovy = fovy.max(0.01).min(std::f32::consts::PI - 0.01);
        self
    }
    
    pub fn aspect(mut self, aspect: f32) -> CameraBuilder {
        self.aspect = aspect;
        self
    }
    
    pub fn near(mut self, near: f32) -> CameraBuilder {
        if near <= 0.0 {
            error!("Near plane ({}) can't be negative!", near);
        }
        self.near = near;
        self
    }
    
    pub fn far(mut self, far: f32) -> CameraBuilder {
        if far <= 0.0 {
            error!("Far plane ({}) can't be negative!", far);
        }
        self.far = far;
        self
    }
    
    pub fn is_active(mut self, is_active: bool) -> CameraBuilder {
        self.is_active = is_active;
        self
    }
}

#[derive(Bundle, )]
pub struct CameraBundle {
    pub camera: Camera,
    pub transform: Transform,
}
