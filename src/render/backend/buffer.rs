use std::mem::size_of;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use ash::vk;

// Buffer
#[derive(Debug)]
pub(crate) struct Buffer {
	pub(crate) buffer: vk::Buffer,
	pub(crate) allocation: Option<Allocation>,
	pub(crate) allocation_name: String,
	pub(crate) size_in_bytes: u64,
	pub(crate) buffer_usage: vk::BufferUsageFlags,
	pub(crate) memory_location: MemoryLocation,
}

impl Buffer {
	pub(crate) fn new(
		logical_device: &ash::Device,
		allocator: &mut gpu_allocator::vulkan::Allocator,
		size_in_bytes: u64,
		buffer_usage: vk::BufferUsageFlags,
		memory_location: MemoryLocation,
		alloc_name: &str,
	) -> Result<Buffer, vk::Result> {
		//Buffer creating
		let buffer = unsafe { logical_device.create_buffer(
			&vk::BufferCreateInfo::builder()
				.size(size_in_bytes)
				.usage(buffer_usage),
			None
		) }?;
		// Buffer memory requirements
		let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
		// Allocation info
		let allocation_info = &AllocationCreateDesc {
			name: alloc_name,
			requirements,
			location: memory_location,
			linear: true,
		};
		// Create memory allocation
		let allocation = allocator.allocate(allocation_info).unwrap();
		// Bind memory allocation to the buffer
		unsafe { logical_device.bind_buffer_memory(
			buffer, 
			allocation.memory(), 
			allocation.offset()).unwrap() 
		};
		
		Ok(Buffer {
			buffer,
			allocation: Some(allocation),
			allocation_name: String::from(alloc_name),
			size_in_bytes,
			buffer_usage,
			memory_location,
		})
	}
	
	pub(crate) fn fill<T: Sized>(
		&mut self,
		logical_device: &ash::Device,
		allocator: &mut gpu_allocator::vulkan::Allocator,
		data: &[T],
	) -> Result<(), vk::Result> {
		let bytes_to_write = (data.len() * size_of::<T>()) as u64;
		if bytes_to_write > self.size_in_bytes {			
			let mut alloc: Option<Allocation> = None;
			std::mem::swap(&mut alloc, &mut self.allocation);
			let alloc = alloc.unwrap();
			allocator.free(alloc).unwrap();
			unsafe { logical_device.destroy_buffer(self.buffer, None); }
			
			let newbuffer = Buffer::new(
				logical_device,
				allocator,
				bytes_to_write,
				self.buffer_usage,
				self.memory_location,
				self.allocation_name.as_str(),
			)?;
			*self = newbuffer;
		}
		
		// Get memory pointer
		let data_ptr = self.allocation.as_ref().unwrap().mapped_ptr().unwrap().as_ptr() as *mut T;
		// Write to the buffer
		unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
		Ok(())
	}
}
