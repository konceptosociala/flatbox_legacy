// Features
#[cfg(all(feature = "x11", feature = "windows"))]
compile_error!("features \"x11\" and \"windows\" cannot be enabled at the same time");

pub mod graphics;

use ash::vk;
use gpu_allocator::vulkan::*;

use graphics::{
	vulkanish::*,
	inits::*,
	model::*,
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
	pub models: Vec<Model<[f32; 3], InstanceData>>,
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
		
		// Create memory allocator
		let mut allocator = Allocator::new(&AllocatorCreateDesc {
			instance: instance.clone(),
			device: logical_device.clone(),
			physical_device,
			debug_settings: Default::default(),
			buffer_device_address: true,
		}).expect("Cannot create allocator");
		
		// Swapchain
		let mut swapchain = Swapchain::init(
			&instance, 
			physical_device, 
			&logical_device, 
			&surfaces, 
			&queue_families,
			&mut allocator
		)?;
		
		// RenderPass, Pipeline
		let renderpass = init_renderpass(&logical_device, physical_device, &surfaces)?;
		swapchain.create_framebuffers(&logical_device, renderpass)?;
		let pipeline = GraphicsPipeline::init(&logical_device, &swapchain, &renderpass)?;

		// CommandBufferPools and CommandBuffers
		let commandbuffer_pools = CommandBufferPools::init(&logical_device, &queue_families)?;
		let commandbuffers = create_commandbuffers(&logical_device, &commandbuffer_pools, swapchain.framebuffers.len())?;
		 
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
			models: vec![],
		})
	}
	
	pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
		let commandbuffer = self.commandbuffers[index];
		let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
		unsafe {
			self.device
				.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
		}
		let clearvalues = [
			vk::ClearValue {
				color: vk::ClearColorValue {
					float32: [0.0, 0.0, 0.08, 1.0],
				},
			},
			vk::ClearValue {
				depth_stencil: vk::ClearDepthStencilValue {
					depth: 1.0,
					stencil: 0,
				},
			},
		];
		
		let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
			.render_pass(self.renderpass)
			.framebuffer(self.swapchain.framebuffers[index])
			.render_area(vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent: self.swapchain.extent,
			})
			.clear_values(&clearvalues);
		unsafe {
			self.device.cmd_begin_render_pass(
				commandbuffer,
				&renderpass_begininfo,
				vk::SubpassContents::INLINE,
			);
			self.device.cmd_bind_pipeline(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline.pipeline,
			);
			for m in &self.models {
				m.draw(&self.device, commandbuffer);
			}
			self.device.cmd_end_render_pass(commandbuffer);
			self.device.end_command_buffer(commandbuffer)?;
		}
		Ok(())
	}

}

impl Drop for Despero {
	fn drop(&mut self) {
		unsafe {
			self.device
				.device_wait_idle()
				.expect("Error halting device");			
			for m in &mut self.models {
				if let Some(vb) = &mut m.vertexbuffer {
					// Reassign VertexBuffer allocation to remove
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut vb.allocation);
					let alloc = alloc.unwrap();
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(vb.buffer, None);
				}
				if let Some(ib) = &mut m.instancebuffer {
					// Reassign InstanceBuffer allocation to remove
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut ib.allocation);
					let alloc = alloc.unwrap();
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(ib.buffer, None);
				}
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
