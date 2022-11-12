use std::collections::HashMap;
use std::mem::size_of;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use ash::vk;

type Handle = usize;

use crate::graphics::{
	vulkanish::*,
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
#[derive(Debug, Clone)]
pub struct InstanceData {
	pub modelmatrix: [[f32; 4]; 4],
	pub colour: [f32; 3],
}

// Model struct
pub struct Model<V, I> {
	// Model vertices
	pub vertexdata: Vec<V>,
	// Vertex indices
	pub indexdata: Vec<u32>,
	// Handle to index of the model instance
	pub handle_to_index: HashMap<usize, Handle>,
	// Vec of the handles
	pub handles: Vec<usize>,
	// Vec of the instances
	pub instances: Vec<I>,
	// Index of first invisible instance
	pub first_invisible: usize,
	// Next handle to use
	pub next_handle: usize,
	pub vertexbuffer: Option<Buffer>,
	pub indexbuffer: Option<Buffer>,
}

impl<V, I: std::fmt::Debug> Model<V, I> {
	pub fn get(&self, handle: Handle) -> Option<&I> {
		if let Some(&index) = self.handle_to_index.get(&handle) {
			self.instances.get(index)
		} else {
			None
		}
	}
	
	pub fn get_mut(&mut self, handle: Handle) -> Option<&mut I> {
		if let Some(&index) = self.handle_to_index.get(&handle) {
			self.instances.get_mut(index)
		} else {
			None
		}
	}
	// Swap instances by handles
	pub fn swap_by_handle(&mut self, handle1: Handle, handle2: Handle) -> Result<(), InvalidHandle> {
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
	
	pub fn is_visible(&self, handle: Handle) -> Result<bool, InvalidHandle> {
		if let Some(index) = self.handle_to_index.get(&handle) {
			Ok(index < &self.first_invisible)
		} else {
			Err(InvalidHandle)
		}
	}
	
	pub fn make_visible(&mut self, handle: Handle) -> Result<(), InvalidHandle> {
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
	
	pub fn make_invisible(&mut self, handle: Handle) -> Result<(), InvalidHandle> {
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
	
	pub fn insert(&mut self, element: I) -> Handle {
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
	
	pub fn insert_visibly(&mut self, element: I) -> Handle {
		let handle = self.insert(element);
		self.make_visible(handle).ok();
		return handle;
	}
	
	// Remove handle and get the element
	pub fn remove(&mut self, handle: Handle) -> Result<I, InvalidHandle> {
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
	
	// Update IndexBuffer
	pub fn update_indexbuffer(
		&mut self,
		logical_device: &ash::Device,
		allocator: &mut Allocator,
	) -> Result<(), vk::Result> {
		// Check whether the buffer exists
		if let Some(buffer) = &mut self.indexbuffer {
			buffer.fill(
				logical_device,
				allocator,
				&self.indexdata,
			)?;
			Ok(())
		} else {
			// Set buffer size
			let bytes = (self.indexdata.len() * size_of::<u32>()) as u64;		
			let mut buffer = Buffer::new(
				&logical_device,
				allocator,
				bytes,
				vk::BufferUsageFlags::INDEX_BUFFER,
				MemoryLocation::CpuToGpu,
				"Model buffer of vertex indices"
			)?;
			
			buffer.fill(
				&logical_device,
				allocator,
				&self.indexdata
			)?;
			self.indexbuffer = Some(buffer);
			Ok(())
		}
	}

	pub fn draw(
		&self, 
		logical_device: &ash::Device, 
		commandbuffer: vk::CommandBuffer,
		layout: vk::PipelineLayout, 
	){
		if let Some(vertexbuffer) = &self.vertexbuffer {
			if let Some(indexbuffer) = &self.indexbuffer {
				if self.first_invisible > 0 {
					unsafe {
						// Bind position buffer						
						logical_device.cmd_bind_index_buffer(
							commandbuffer,
							indexbuffer.buffer,
							0,
							vk::IndexType::UINT32,
						);
						
						logical_device.cmd_bind_vertex_buffers(
							commandbuffer,
							0,
							&[vertexbuffer.buffer],
							&[0],
						);
						
						// Push Constants
						for ins in &self.instances[0..self.first_invisible] {							
							let ptr = ins as *const _ as *const u8;
							let bytes = std::slice::from_raw_parts(ptr, size_of::<InstanceData>());
							
							logical_device.cmd_push_constants(
								commandbuffer,
								layout,
								vk::ShaderStageFlags::VERTEX,
								0,
								bytes,
							);
							
							logical_device.cmd_draw_indexed(
								commandbuffer,
								self.indexdata.len() as u32,
								1,
								0,
								0,
								0,
							);
						}						
					}
				}
			}
		}
	}
}


//Implement cubic model
impl Model<[f32; 3], InstanceData> {
	pub fn cube() -> Model<[f32; 3], InstanceData> {
		let lbf = [-1.0,1.0,0.0]; // Left-bottom-front
		let lbb = [-1.0,1.0,1.0]; // Left-bottom-back
		let ltf = [-1.0,-1.0,0.0];// Left-top-front
		let ltb = [-1.0,-1.0,1.0];// Left-top-back
		let rbf = [1.0,1.0,0.0];  // Right-bottom-front
		let rbb = [1.0,1.0,1.0];  // Right-bottom-back
		let rtf = [1.0,-1.0,0.0]; // Right-top-front
		let rtb = [1.0,-1.0,1.0]; // Right-top-back

		Model {
			vertexdata: vec![lbf,lbb,ltf,ltb,rbf,rbb,rtf,rtb],
			indexdata: vec![
				0, 1, 5, 0, 5, 4, //bottom
				2, 7, 3, 2, 6, 7, //top
				0, 6, 2, 0, 4, 6, //front
				1, 3, 7, 1, 7, 5, //back
				0, 2, 1, 1, 2, 3, //left
				4, 5, 6, 5, 7, 6, //right
			],
			handle_to_index: HashMap::new(),
			handles: Vec::new(),
			instances: Vec::new(),
			first_invisible: 0,
			next_handle: 0,
			vertexbuffer: None,
			indexbuffer: None,
		}
	}
}
