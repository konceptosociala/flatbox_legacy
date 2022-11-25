use std::collections::HashMap;
use std::mem::size_of;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use ash::vk;
use nalgebra as na;

use crate::render::buffer::Buffer;
use crate::engine::model::Model;

// Textured VertexData
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TexturedVertexData {
    pub position: [f32; 3],
}

// Textured InstanceData
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TexturedInstanceData {
    pub modelmatrix: [[f32; 4]; 4],
    pub inverse_modelmatrix: [[f32; 4]; 4],
}

impl TexturedInstanceData {
    pub fn new(
		modelmatrix: na::Matrix4<f32>
	) -> TexturedInstanceData {
        TexturedInstanceData {
            modelmatrix: modelmatrix.into(),
            inverse_modelmatrix: modelmatrix.try_inverse().unwrap().into(),
        }
    }
}

impl Model<TexturedVertexData, TexturedInstanceData> {
    pub fn quad() -> Self {
        let lb = TexturedVertexData {
            position: [-1.0, 1.0, 0.0],
        };
        
        let lt = TexturedVertexData {
            position: [-1.0, -1.0, 0.0],
        };
        
        let rb = TexturedVertexData {
            position: [1.0, 1.0, 0.0],
        };
        
        let rt = TexturedVertexData {
            position: [1.0, -1.0, 0.0],
        };
        
        Model {
            vertexdata: vec![lb, lt, rb, rt],
            indexdata: vec![0, 2, 1, 1, 2, 3],
            handle_to_index: std::collections::HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            first_invisible: 0,
            next_handle: 0,
            vertexbuffer: None,
            indexbuffer: None,
            instancebuffer: None,
        }
    }
}
