// Features
#[cfg(all(feature = "x11", feature = "windows"))]
compile_error!("features \"x11\" and \"windows\" cannot be enabled at the same time");

use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use nalgebra as na;

pub mod render;
pub mod engine;
pub mod ecs;
pub mod physics;
pub mod scripting;

use crate::render::{
	surface::Surface,
	swapchain::Swapchain,
	queues::{
		QueueFamilies,
		Queues,
		init_instance,
		init_device_and_queues,
		init_physical_device_and_properties,
	},
	pipeline::{
		GraphicsPipeline,
		init_renderpass,
	},
	commandbuffers::{
		CommandBufferPools,
		create_commandbuffers,
	},
	buffer::Buffer,
};

use crate::engine::{
	debug::Debug,
	model::{
		Model,
		InstanceData,
		VertexData,
	},
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
	pub models: Vec<Model<VertexData, InstanceData>>,
	pub uniformbuffer: Buffer,
	pub descriptor_pool: vk::DescriptorPool,
	pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Despero {
	pub fn init(
		window: winit::window::Window,
		app_title: &str,
	) -> Result<Despero, Box<dyn std::error::Error>> {
		// Set window title
		window.set_title(app_title);
		// Create Entry
		let entry = unsafe { ash::Entry::load()? };
		// Instance, Debug, Surface
		let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
		let instance = init_instance(&entry, &layer_names, app_title)?;
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
		
		// Uniform buffer
		let mut uniformbuffer = Buffer::new(
			&logical_device,
			&mut allocator,
			128,
			vk::BufferUsageFlags::UNIFORM_BUFFER,
			MemoryLocation::CpuToGpu,
			"Uniform buffer"
		)?;
		// Camera transform
		let cameratransform: [[[f32; 4]; 4]; 2] = [
			na::Matrix4::identity().into(),
			na::Matrix4::identity().into(),
		];
		uniformbuffer.fill(&logical_device, &mut allocator, &cameratransform)?;
		
		// Descriptor pool
		//
		// Set pool size
		let pool_sizes = [vk::DescriptorPoolSize {
			ty: vk::DescriptorType::UNIFORM_BUFFER,
			descriptor_count: swapchain.amount_of_images,
		}];
		// PoolCreateInfo
		let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
			// Amount of descriptors
			.max_sets(swapchain.amount_of_images)
			// Size of pool
			.pool_sizes(&pool_sizes); 
		let descriptor_pool = unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;
		
		// Descriptor sets
		//
		// Descriptor layouts
		let desc_layouts = vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images as usize];
		// SetAllocateInfo
		let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			// DescPool
			.descriptor_pool(descriptor_pool)
			// Layouts
			.set_layouts(&desc_layouts);
		let descriptor_sets = unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }?;

		// Fill descriptor sets
		for (_, descset) in descriptor_sets.iter().enumerate() {
			let buffer_infos = [vk::DescriptorBufferInfo {
				buffer: uniformbuffer.buffer,
				offset: 0,
				range: 128,
			}];
			let desc_sets_write = [vk::WriteDescriptorSet::builder()
				.dst_set(*descset)
				.dst_binding(0)
				.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
				.buffer_info(&buffer_infos)
				.build()
			];
			unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
		}
		 
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
			uniformbuffer,
			descriptor_pool,
			descriptor_sets,
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
					float32: [0.0, 0.0, 0.0, 1.0],
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
			// Bind RenderPass
			self.device.cmd_begin_render_pass(
				commandbuffer,
				&renderpass_begininfo,
				vk::SubpassContents::INLINE,
			);
			// Bind Pipeline
			self.device.cmd_bind_pipeline(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline.pipeline,
			);
			// Bind DescriptorSets
			self.device.cmd_bind_descriptor_sets(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline.layout,
				0,
				&[self.descriptor_sets[index]],
				&[],
			);
			for m in &self.models {
				m.draw(
					&self.device,
					commandbuffer,
					self.pipeline.layout,
				);
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
			// Destroy UniformBuffer
			self.device.destroy_buffer(self.uniformbuffer.buffer, None);
			self.device.free_memory(self.uniformbuffer.allocation.as_ref().unwrap().memory(), None);
			for m in &mut self.models {
				if let Some(vb) = &mut m.vertexbuffer {
					// Reassign VertexBuffer allocation to remove
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut vb.allocation);
					let alloc = alloc.unwrap();
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(vb.buffer, None);
				}
				if let Some(ib) = &mut m.indexbuffer {
					// Reassign IndexBuffer allocation to remove
					let mut alloc: Option<Allocation> = None;
					std::mem::swap(&mut alloc, &mut ib.allocation);
					let alloc = alloc.unwrap();
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(ib.buffer, None);
				}
			}
			self.device.destroy_descriptor_pool(self.descriptor_pool, None);
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
