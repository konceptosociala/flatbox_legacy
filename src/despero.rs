// Features
#[cfg(all(feature = "x11", feature = "windows"))]
compile_error!("features \"x11\" and \"windows\" cannot be enabled at the same time");

pub mod graphics;

use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;

use graphics::{
	vulkanish::*,
	inits::*,
	models::*,
};

// Main struct
pub struct Despero {
	pub window: winit::window::Window,
	pub entry: ash::Entry,
	pub instance: ash::Instance,
	pub debug: std::mem::ManuallyDrop<Debug>,
	pub surfaces: std::mem::ManuallyDrop<Surface>,
	pub physical_device: vk::PhysicalDevice,
	pub physical_device_properties: vk::PhysicalDeviceProperties,
	pub queue_families: QueueFamilies,
	pub queues: Queues,
	pub device: ash::Device,
	pub swapchain: Swapchain,
	pub renderpass: vk::RenderPass,
	pub pipeline: GraphicsPipeline,
	pub commandbuffer_pools: CommandBufferPools,
	pub commandbuffers: Vec<vk::CommandBuffer>,
	pub allocator: gpu_allocator::vulkan::Allocator,
	pub buffers: Vec<Buffer>,
}

impl Despero {
	pub fn init(window: winit::window::Window)
	-> Result<Despero, Box<dyn std::error::Error>> {
		let entry = unsafe { ash::Entry::load()? };
		// Instance, Debug, Surface
		let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
		let instance = init_instance(&entry, &layer_names)?;	
		let debug = Debug::init(&entry, &instance)?;
		let surfaces = Surface::init(&window, &entry, &instance)?;
		
		// PhysicalDevice and PhysicalDeviceProperties
		let (physical_device, physical_device_properties, _) = init_physical_device_and_properties(&instance)?;
		// QueueFamilies, (Logical) Device, Queues
		let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces)?;
		let (logical_device, queues) = init_device_and_queues(&instance, physical_device, &queue_families, &layer_names)?;
		
		// Swapchain
		let mut swapchain = Swapchain::init(
			&instance, 
			physical_device, 
			&logical_device, 
			&surfaces, 
			&queue_families,
		)?;
		
		// RenderPass, Pipeline
		let renderpass = init_renderpass(&logical_device, physical_device, &surfaces)?;
		swapchain.create_framebuffers(&logical_device, renderpass)?;
		let pipeline = GraphicsPipeline::init(&logical_device, &swapchain, &renderpass)?;
		
		// Create memory allocator
		let mut allocator = Allocator::new(&AllocatorCreateDesc {
			instance: instance.clone(),
			device: logical_device.clone(),
			physical_device,
			debug_settings: Default::default(),
			buffer_device_address: true,
		}).expect("Cannot create allocator");
		
		// Create Buffer 1
		let buffer1 = Buffer::new(
			&logical_device,
			&mut allocator,
			96,
			vk::BufferUsageFlags::VERTEX_BUFFER,
			MemoryLocation::CpuToGpu,
			"Vertex position buffer"
		)?;
		// Fill Buffer 1
		buffer1.fill(&[
			0.5f32,   0.0f32, 0.0f32, 1.0f32, 
			0.0f32,   0.2f32, 0.0f32, 1.0f32, 
			-0.5f32,  0.0f32, 0.0f32, 1.0f32, 
			-0.9f32, -0.9f32, 0.0f32, 1.0f32, 
			0.3f32,  -0.8f32, 0.0f32, 1.0f32,
			0.0f32,  -0.6f32, 0.0f32, 1.0f32,
		])?;
		
		// Create Buffer 2
		let buffer2 = Buffer::new(
			&logical_device,
			&mut allocator,
			120,
			vk::BufferUsageFlags::VERTEX_BUFFER,
			MemoryLocation::CpuToGpu,
			"Vertex size and colour buffer"
		)?;
		// Fill Buffer 2
		buffer2.fill(&[
			1.0f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 
			1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32,
			1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 
			1.0f32, 1.0f32, 1.0f32, 0.0f32, 1.0f32,
			1.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32, 
			1.0f32, 1.0f32, 0.0f32, 1.0f32, 1.0f32,
		])?;

		// CommandBufferPools and CommandBuffers
		let commandbuffer_pools = CommandBufferPools::init(&logical_device, &queue_families)?;
		let commandbuffers = create_commandbuffers(&logical_device, &commandbuffer_pools, swapchain.framebuffers.len())?;
		fill_commandbuffers(
			&commandbuffers,
			&logical_device,
			&renderpass,
			&swapchain,
			&pipeline,
			&buffer1.buffer,
			&buffer2.buffer,
		)?;
		 
		Ok(Despero {
			window,
			entry,
			instance,
			debug: std::mem::ManuallyDrop::new(debug),
			surfaces: std::mem::ManuallyDrop::new(surfaces),
			physical_device,
			physical_device_properties,
			queue_families,
			queues,
			device: logical_device,
			swapchain,
			renderpass,
			pipeline,
			commandbuffer_pools,
			commandbuffers,
			allocator,
			buffers: vec![buffer1, buffer2],
		})
	}
}

impl Drop for Despero {
	fn drop(&mut self) {
		unsafe {
			self.device
				.device_wait_idle()
				.expect("Error halting device");			
			for b in &mut self.buffers {
				// Reassign Allocation to delete
				let mut alloc = Allocation::default();
				std::mem::swap(&mut alloc, &mut b.allocation);
				self.allocator.free(alloc).unwrap();
				self.device.destroy_buffer(b.buffer, None);
			}
			self.commandbuffer_pools.cleanup(&self.device);
			self.pipeline.cleanup(&self.device);
			self.device.destroy_render_pass(self.renderpass, None);
			self.swapchain.cleanup(&self.device);
			self.device.destroy_device(None);
			std::mem::ManuallyDrop::drop(&mut self.surfaces);
			std::mem::ManuallyDrop::drop(&mut self.debug);
			self.instance.destroy_instance(None);
		};
	}
}
