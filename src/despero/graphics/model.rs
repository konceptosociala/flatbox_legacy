use std::collections::HashMap;
use std::mem::size_of;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use ash::vk;

use crate::graphics::{
	vulkanish::*,
	inits::*,
};

// InvalidHandle custom error
#[derive(Debug, Clone)]
pub struct InvalidHandle;

impl std::fmt::Display for InvalidHandle {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Invalid handle")
	}
}

impl std::error::Error for InvalidHandle {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		None
	}
}

#[repr(C)]
pub struct InstanceData {
    pub position: [f32; 3],
    pub colour: [f32; 3],
}

// Model struct
pub struct Model<V, I> {
	pub vertexdata: Vec<V>,
	// Handle to index of the model instance
	pub handle_to_index: HashMap<usize, usize>,
	// Vec of the handles
	pub handles: Vec<usize>,
	// Vec of the instances
	pub instances: Vec<I>,
	// Index of first invisible instance
	pub first_invisible: usize,
	// Next handle to use
	pub next_handle: usize,
	pub vertexbuffer: Option<Buffer>,
	pub instancebuffer: Option<Buffer>,
}

impl<V, I> Model<V, I> {
	pub fn get(&self, handle: usize) -> Option<&I> {
		if let Some(&index) = self.handle_to_index.get(&handle) {
			self.instances.get(index)
		} else {
			None
		}
	}
	
	pub fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
		if let Some(&index) = self.handle_to_index.get(&handle) {
			self.instances.get_mut(index)
		} else {
			None
		}
	}
	// Swap instances by handles
	pub fn swap_by_handle(&mut self, handle1: usize, handle2: usize) -> Result<(), InvalidHandle> {
		if handle1 == handle2 {
			return Ok(());
		}
		// Get indices of the handles
		if let (Some(&index1), Some(&index2)) = (
			self.handle_to_index.get(&handle1),
			self.handle_to_index.get(&handle2),
		) {
			self.handles.swap(index1, index2);
			self.instances.swap(index1, index2);
			self.handle_to_index.insert(index1, handle2);
			self.handle_to_index.insert(index2, handle1);
			Ok(())
		} else {
			Err(InvalidHandle)
		}
	}
	//Swap instances by index
	pub fn swap_by_index(&mut self, index1: usize, index2: usize) {
		if index1 == index2 {
			return;
		}
		let handle1 = self.handles[index1];
		let handle2 = self.handles[index2];
		self.handles.swap(index1, index2);
		self.instances.swap(index1, index2);
		self.handle_to_index.insert(index1, handle2);
		self.handle_to_index.insert(index2, handle1);
	}
	
	pub fn is_visible(&self, handle: usize) -> Result<bool, InvalidHandle> {
		if let Some(index) = self.handle_to_index.get(&handle) {
			Ok(index < &self.first_invisible)
		} else {
			Err(InvalidHandle)
		}
	}
	
	pub fn make_visible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
		// Check if invisible
		if let Some(&index) = self.handle_to_index.get(&handle) {
			if index < self.first_invisible {
				return Ok(());
			}
			// Move to position `first_invisible` and increase value of `first_invisible`
			self.swap_by_index(index, self.first_invisible);
			self.first_invisible += 1;
			Ok(())
		} else {
			Err(InvalidHandle)
		}
	}
	
	pub fn make_invisible(&mut self, handle: usize) -> Result<(), InvalidHandle> {
		// Check if visible
		if let Some(&index) = self.handle_to_index.get(&handle) {
			if index >= self.first_invisible {
				return Ok(());
			}
			// Move to position before `first_invisible` and decrease value of `first_invisible`
			self.swap_by_index(index, self.first_invisible - 1);
			self.first_invisible -= 1;
			Ok(())
		} else {
			Err(InvalidHandle)
		}
	}
	
	pub fn insert(&mut self, element: I) -> usize {
		// Make new handle
		let handle = self.next_handle;
		self.next_handle += 1;
		// Put index at the end
		let index = self.instances.len();
		self.instances.push(element);
		self.handles.push(handle);
		// Link handle and index
		self.handle_to_index.insert(handle, index);
		return handle;
	}
	
	pub fn insert_visibly(&mut self, element: I) -> usize {
		let handle = self.insert(element);
		self.make_visible(handle).ok();
		return handle;
	}
	
	// Remove handle and get the element
	pub fn remove(&mut self, handle: usize) -> Result<I, InvalidHandle> {
		// Get index of the handle
		if let Some(&index) = self.handle_to_index.get(&handle) {
			if index < self.first_invisible {
				self.swap_by_index(index, self.first_invisible - 1);
				self.first_invisible -= 1;
			}
			self.swap_by_index(self.first_invisible, self.instances.len() - 1);
			self.handles.pop();
			self.handle_to_index.remove(&handle);
			Ok(self.instances.pop().unwrap())
		} else {
			Err(InvalidHandle)
		}
	}
	
	// Update VertexBuffer
	pub fn update_vertexbuffer(
		&mut self,
		logical_device: &ash::Device,
		allocator: &mut Allocator,
	) -> Result<(), vk::Result> {
		// Check whether the buffer exists
		if let Some(buffer) = &mut self.vertexbuffer {
			buffer.fill(
				logical_device,
				allocator,
				&self.vertexdata
			)?;
			Ok(())
		} else {
			// Set buffer size
			let bytes = (self.vertexdata.len() * size_of::<V>()) as u64;		
			let mut buffer = Buffer::new(
				&logical_device,
				allocator,
				bytes,
				vk::BufferUsageFlags::VERTEX_BUFFER,
				MemoryLocation::CpuToGpu,
				"Model vertex buffer"
			)?;
			
			buffer.fill(
				&logical_device,
				allocator,
				&self.vertexdata
			)?;
			self.vertexbuffer = Some(buffer);
			Ok(())
		}
	}
	
	// Update InstanceBuffer
	pub fn update_instancebuffer(
		&mut self,
		logical_device: &ash::Device,
		allocator: &mut Allocator,
	) -> Result<(), vk::Result> {
		if let Some(buffer) = &mut self.instancebuffer {
			buffer.fill(
				logical_device,
				allocator, 
				&self.instances[0..self.first_invisible]
			)?;
			Ok(())
		} else {
			let bytes = (self.first_invisible * size_of::<I>()) as u64;			
			let mut buffer = Buffer::new(
				&logical_device,
				allocator,
				bytes,
				vk::BufferUsageFlags::VERTEX_BUFFER,
				MemoryLocation::CpuToGpu,
				"Model instance buffer"
			)?;
			
			buffer.fill(
				&logical_device,
				allocator,
				&self.instances[0..self.first_invisible]
			)?;
			self.instancebuffer = Some(buffer);
			Ok(())
		}
	}

	pub fn draw(
		&self, 
		logical_device: &ash::Device, 
		commandbuffer: vk::CommandBuffer,
	){
		if let Some(vertexbuffer) = &self.vertexbuffer {
			if let Some(instancebuffer) = &self.instancebuffer {
				if self.first_invisible > 0 {
					unsafe {
						logical_device.cmd_bind_vertex_buffers(
							commandbuffer,
							0,
							&[vertexbuffer.buffer],
							&[0],
						);
						logical_device.cmd_bind_vertex_buffers(
							commandbuffer,
							1,
							&[instancebuffer.buffer],
							&[0],
						);
						logical_device.cmd_draw(
							commandbuffer,
							self.vertexdata.len() as u32,
							self.first_invisible as u32,
							0,
							0,
						);
					}
				}
			}
		}
	}
}

//Implement cubic model
impl Model<[f32; 3], InstanceData> {
    pub fn cube() -> Model<[f32; 3], InstanceData> {
		let LBF = [-0.1,0.1,0.0]; // Left-bottom-front
        let LBB = [-0.1,0.1,0.1]; // Left-bottom-back
        let LTF = [-0.1,-0.1,0.0];// Left-top-front
        let LTB = [-0.1,-0.1,0.1];// Left-top-back
        let RBF = [0.1,0.1,0.0];  // Right-bottom-front
        let RBB = [0.1,0.1,0.1];  // Right-bottom-back
        let RTF = [0.1,-0.1,0.0]; // Right-top-front
        let RTB = [0.1,-0.1,0.1]; // Right-top-back

        Model {
            vertexdata: vec![
				LBF, LBB, RBB, LBF, RBB, RBF, // Bottom
                LTF, RTB, LTB, LTF, RTF, RTB, // Top
                LBF, RTF, LTF, LBF, RBF, RTF, // Front
                LBB, LTB, RTB, LBB, RTB, RBB, // Back
                LBF, LTF, LBB, LBB, LTF, LTB, // Left
                RBF, RBB, RTF, RBB, RTB, RTF, // Right
            ],
            handle_to_index: HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            first_invisible: 0,
            next_handle: 0,
            vertexbuffer: None,
            instancebuffer: None,
        }
    }
}
