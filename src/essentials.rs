// Features
#[cfg(all(feature = "x11", feature = "windows"))]
compile_error!("features \"x11\" and \"windows\" cannot be enabled at the same time");

use colored::Colorize;
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;

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
		let (physical_device, physical_device_properties) = init_physical_device_and_properties(&instance)?;
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
			15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 
			15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32,
			15.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 
			1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32,
			1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32, 
			1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32,
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

// Debug
pub struct Debug {
	loader: ash::extensions::ext::DebugUtils,
	messenger: vk::DebugUtilsMessengerEXT
}

impl Debug {
	pub fn init(
		entry: &ash::Entry,
		instance: &ash::Instance,
	) -> Result<Debug, vk::Result> {
		let debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(
				vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
					| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
					| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
					| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
			)
			.message_type(
				vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
					| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
					| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
			)
			.pfn_user_callback(Some(vulkan_debug_utils_callback));
		
		let loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
		let messenger = unsafe { loader.create_debug_utils_messenger(&debugcreateinfo, None)? };
		
		Ok(Debug {loader, messenger})
	}
}

impl Drop for Debug {
	fn drop(&mut self) {
		unsafe { self.loader.destroy_debug_utils_messenger(self.messenger, None) };
	}
}

// Surface
pub struct Surface {
	pub surface: vk::SurfaceKHR,
	pub surface_loader: ash::extensions::khr::Surface,
}

impl Surface {
	pub fn init(
		window: &winit::window::Window,
		entry: &ash::Entry,
		instance: &ash::Instance,
	) -> Result<Surface, vk::Result> {
		// Creating surface from `raw` handles with `ash-window` crate
		let surface = unsafe { ash_window::create_surface(
			&entry, 
			&instance, 
			window.raw_display_handle(), 
			window.raw_window_handle(), 
			None
		)? };
		
		let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
		Ok(Surface {
			surface,
			surface_loader,
		})
	}
	
	pub fn get_capabilities(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
		unsafe {
			self.surface_loader.get_physical_device_surface_capabilities(physical_device, self.surface)
		}
	}
	
	pub fn get_present_modes(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
		unsafe {
			self.surface_loader
				.get_physical_device_surface_present_modes(physical_device, self.surface)
		}
	}
	
	pub fn get_formats(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
		unsafe {
			self.surface_loader
				.get_physical_device_surface_formats(physical_device, self.surface)
		}
	}
	
	pub fn get_physical_device_surface_support(
		&self,
		physical_device: vk::PhysicalDevice,
		queuefamilyindex: usize,
	) -> Result<bool, vk::Result> {
		unsafe {
			self.surface_loader.get_physical_device_surface_support(
				physical_device,
				queuefamilyindex as u32,
				self.surface,
			)
		}
	}

}

impl Drop for Surface {
	fn drop(&mut self) {
		unsafe {
			self.surface_loader.destroy_surface(self.surface, None);
		}
	}
}

// QueueFamilies
pub struct QueueFamilies {
	graphics_q_index: Option<u32>,
	transfer_q_index: Option<u32>,
}

impl QueueFamilies {
	pub fn init(
		instance: &ash::Instance,
		physical_device: vk::PhysicalDevice,
		surfaces: &Surface,
	) -> Result<QueueFamilies, vk::Result>{
		// Get queue families
		let queuefamilyproperties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
		let mut found_graphics_q_index = None;
		let mut found_transfer_q_index = None;
		// Get indices of queue families
		for (index, qfam) in queuefamilyproperties.iter().enumerate() {
			if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS) && 
				surfaces.get_physical_device_surface_support(physical_device, index)?
			{
				found_graphics_q_index = Some(index as u32);
			}
			if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::TRANSFER) {
				if found_transfer_q_index.is_none()
					|| !qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
				{
					found_transfer_q_index = Some(index as u32);
				}
			}
		}
		
		Ok(QueueFamilies {
			graphics_q_index: found_graphics_q_index,
			transfer_q_index: found_transfer_q_index,
		})
	}
}

// Queues
pub struct Queues {
	pub graphics_queue: vk::Queue,
	pub transfer_queue: vk::Queue,
}

// Swapchain
pub struct Swapchain {
	pub swapchain_loader: ash::extensions::khr::Swapchain,
	pub swapchain: vk::SwapchainKHR,
	pub images: Vec<vk::Image>,
	pub imageviews: Vec<vk::ImageView>,
	pub framebuffers: Vec<vk::Framebuffer>,
	pub surface_format: vk::SurfaceFormatKHR,
	pub extent: vk::Extent2D,
	// Fence
	pub may_begin_drawing: Vec<vk::Fence>,
	// Semaphores
	pub image_available: Vec<vk::Semaphore>,
	pub rendering_finished: Vec<vk::Semaphore>,
	pub amount_of_images: u32,
	pub current_image: usize,
}

impl Swapchain {
	pub fn init(
		instance: &ash::Instance,
		physical_device: vk::PhysicalDevice,
		logical_device: &ash::Device,
		surfaces: &Surface,
		queue_families: &QueueFamilies,
	) -> Result<Swapchain, vk::Result> {
		let surface_capabilities = surfaces.get_capabilities(physical_device)?;
		let extent = surface_capabilities.current_extent;
		let surface_format = *surfaces.get_formats(physical_device)?.first().unwrap();
		
		// Get graphics queue family
		let queuefamilies = [queue_families.graphics_q_index.unwrap()];
		// Swapchain creation
		let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(surfaces.surface)
			.min_image_count(
				3.max(surface_capabilities.max_image_count)
					.min(surface_capabilities.min_image_count),
			)
			.image_format(surface_format.format)
			.image_color_space(surface_format.color_space)
			.image_extent(surface_capabilities.current_extent)
			.image_array_layers(1)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
			.queue_family_indices(&queuefamilies)
			.pre_transform(surface_capabilities.current_transform)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(vk::PresentModeKHR::FIFO);
			
		let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &logical_device);
		
		let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
		
		let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
		let amount_of_images = swapchain_images.len() as u32;
		let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
		// Push swapchain images to ImageViews
		for image in &swapchain_images {
			let subresource_range = vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1);
			let imageview_create_info = vk::ImageViewCreateInfo::builder()
				.image(*image)
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(vk::Format::B8G8R8A8_SRGB)
				.subresource_range(*subresource_range);
			let imageview = unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
			swapchain_imageviews.push(imageview);
		}
		
		// Creating Semaphores and Fences
		// 
		// Available images
		let mut image_available = vec![];
		// Is rendering finished
		let mut rendering_finished = vec![];
		// May begin drawing 
		let mut may_begin_drawing = vec![];
		let semaphoreinfo = vk::SemaphoreCreateInfo::builder();
		let fenceinfo = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
		for _ in 0..amount_of_images {
			// semaphores
			let semaphore_available = unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
			let semaphore_finished = unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
			image_available.push(semaphore_available);
			rendering_finished.push(semaphore_finished);
			// fences
			let fence = unsafe { logical_device.create_fence(&fenceinfo, None) }?;
			may_begin_drawing.push(fence);
		}
		
		Ok(Swapchain {
			swapchain_loader,
			swapchain,
			images: swapchain_images,
			imageviews: swapchain_imageviews,
			framebuffers: vec![],
			surface_format,
			extent,
			image_available,
			rendering_finished,
			may_begin_drawing,
			amount_of_images,
			current_image:0,
		})
	}
	
	// Create FBs for the swapchain
	fn create_framebuffers(
		&mut self,
		logical_device: &ash::Device,
		renderpass: vk::RenderPass,
	) -> Result<(), vk::Result> {
		for iv in &self.imageviews {
			let iview = [*iv];
			let framebuffer_info = vk::FramebufferCreateInfo::builder()
				.render_pass(renderpass)
				.attachments(&iview)
				.width(self.extent.width)
				.height(self.extent.height)
				.layers(1);
			let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
			self.framebuffers.push(fb);
		}
		Ok(())
	}
	
	unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
		// Remove Fences
		for fence in &self.may_begin_drawing {
			logical_device.destroy_fence(*fence, None);
		}
		// Remove Semaphores
		for semaphore in &self.image_available {
			logical_device.destroy_semaphore(*semaphore, None);
		}
		for semaphore in &self.rendering_finished {
			logical_device.destroy_semaphore(*semaphore, None);
		}
		// Remove ImageViews and FrameBuffers
		for iv in &self.imageviews {
			logical_device.destroy_image_view(*iv, None);
		}
		for fb in &self.framebuffers {
			logical_device.destroy_framebuffer(*fb, None);
		}
		self.swapchain_loader.destroy_swapchain(self.swapchain, None)
	}
}

// Pipeline
pub struct GraphicsPipeline {
	pipeline: vk::Pipeline,
	layout: vk::PipelineLayout,
}

impl GraphicsPipeline {
	pub fn init(
		logical_device: &ash::Device,
		swapchain: &Swapchain,
		renderpass: &vk::RenderPass,
	) -> Result<GraphicsPipeline, vk::Result>{
		// Include shaders
		let vertexshader_createinfo = vk::ShaderModuleCreateInfo::builder().code(vk_shader_macros::include_glsl!(
			"./shaders/vertex.glsl", 
			kind: vert,
		));
		let fragmentshader_createinfo = vk::ShaderModuleCreateInfo::builder().code(vk_shader_macros::include_glsl!(
			"./shaders/fragment.glsl",
			kind: frag,
		));
		let vertexshader_module = unsafe { logical_device.create_shader_module(&vertexshader_createinfo, None)? };
		let fragmentshader_module = unsafe { logical_device.create_shader_module(&fragmentshader_createinfo, None)? };
		
		// Set main function's name to `main`
		let main_function = std::ffi::CString::new("main").unwrap();
		
		let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vertexshader_module)
			.name(&main_function);
		let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(fragmentshader_module)
			.name(&main_function);
			
		let shader_stages = vec![vertexshader_stage.build(), fragmentshader_stage.build()];
		
		// Vertex Input Info
		// 
		// Attribute description
		let vertex_attrib_descs = [
			vk::VertexInputAttributeDescription {
				binding: 0,
				location: 0,
				offset: 0,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			vk::VertexInputAttributeDescription {
				binding: 1,
				location: 1,
				offset: 0,
				format: vk::Format::R32_SFLOAT,
			},
			vk::VertexInputAttributeDescription {
				binding: 1,
				location: 2,
				offset: 4,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
		];
		// Input Bindings' description
		//
		// stride	  - binding variables' size
		// input_rate - frequency, when the data is changed (per vertex/ per instance)
		let vertex_binding_descs = [
			vk::VertexInputBindingDescription {
				binding: 0,
				stride: 16,
				input_rate: vk::VertexInputRate::VERTEX,
			},
			vk::VertexInputBindingDescription {
				binding: 1,
				stride: 20,
				input_rate: vk::VertexInputRate::VERTEX,
			}
		];
		// Bind vertex inputs
		let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_attribute_descriptions(&vertex_attrib_descs)
			.vertex_binding_descriptions(&vertex_binding_descs);
			
		let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST);
		
		// Viewports
		let viewports = [vk::Viewport {
			x: 0.,
			y: 0.,
			width: swapchain.extent.width as f32,
			height: swapchain.extent.height as f32,
			min_depth: 0.,
			max_depth: 1.,
		}];
		
		let scissors = [vk::Rect2D {
			offset: vk::Offset2D { x: 0, y: 0 },
			extent: swapchain.extent,
		}];

		let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
			.viewports(&viewports)
			.scissors(&scissors);
			
		// Rasterizer
		let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
			.line_width(1.0)
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.cull_mode(vk::CullModeFlags::NONE)
			.polygon_mode(vk::PolygonMode::FILL);
			
			
		// Multisampler	
		let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);
			
		// Color blend
		let colorblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
			.blend_enable(true)
			.src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
			.dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
			.color_blend_op(vk::BlendOp::ADD)
			.src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
			.dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
			.alpha_blend_op(vk::BlendOp::ADD)
			.color_write_mask(
				vk::ColorComponentFlags::R
					| vk::ColorComponentFlags::G
					| vk::ColorComponentFlags::B
					| vk::ColorComponentFlags::A,
			)
			.build()];
		let colourblend_info = vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colorblend_attachments);
		
		// Pipeline layout
		let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder();
		let pipelinelayout = unsafe { logical_device.create_pipeline_layout(&pipelinelayout_info, None) }?;
		
		// Graphics Pipeline
		let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&shader_stages)
			.vertex_input_state(&vertex_input_info)
			.input_assembly_state(&input_assembly_info)
			.viewport_state(&viewport_info)
			.rasterization_state(&rasterizer_info)
			.multisample_state(&multisampler_info)
			.color_blend_state(&colourblend_info)
			.layout(pipelinelayout)
			.render_pass(*renderpass)
			.subpass(0);
		let graphicspipeline = unsafe {
			logical_device
				.create_graphics_pipelines(
					vk::PipelineCache::null(),
					&[pipeline_info.build()],
					None,
				).expect("Cannot create pipeline")				
		}[0];
		
		// Destroy useless shader modules
		unsafe {
			logical_device.destroy_shader_module(fragmentshader_module, None);
			logical_device.destroy_shader_module(vertexshader_module, None);
		}
		
		Ok(GraphicsPipeline {
			pipeline: graphicspipeline,
			layout: pipelinelayout,
		})
	}
	
	fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
			logical_device.destroy_pipeline(self.pipeline, None);
			logical_device.destroy_pipeline_layout(self.layout, None);
		}
	}
}

// Buffer
pub struct Buffer {
	buffer: vk::Buffer,
	allocation: gpu_allocator::vulkan::Allocation,
}

impl Buffer {
	pub fn new(
		logical_device: &ash::Device,
		allocator: &mut gpu_allocator::vulkan::Allocator,
		size_in_bytes: u64,
		usage: vk::BufferUsageFlags,
		memory_location: MemoryLocation,
		alloc_name: &str,
	) -> Result<Buffer, vk::Result> {
		//Buffer creating
		let buffer = unsafe { logical_device.create_buffer(
			&vk::BufferCreateInfo::builder()
				.size(size_in_bytes)
				.usage(usage),
			None
		) }?;
		// Buffer memory requirements
		let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
		// Create memory allocation
		let allocation = allocator
			.allocate(&AllocationCreateDesc {
				name: alloc_name,
				requirements,
				location: memory_location,
				linear: true,
			}).unwrap();
		// Bind memory allocation to the buffer
		unsafe { logical_device.bind_buffer_memory(
			buffer, 
			allocation.memory(), 
			allocation.offset()).unwrap() 
		};
		
		Ok(Buffer {
			buffer,
			allocation,
		})
	}
	
	pub fn fill<T: Sized>(
		&self,
		data: &[T],
	) -> Result<(), vk::Result> {
		// Get memory pointer
		let data_ptr = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut T;
		// Write to the buffer
		unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
		Ok(())
	}
}


pub struct CommandBufferPools {
	commandpool_graphics: vk::CommandPool,
	commandpool_transfer: vk::CommandPool,
}

impl CommandBufferPools {
	pub fn init(
		logical_device: &ash::Device,
		queue_families: &QueueFamilies,
	) -> Result<CommandBufferPools, vk::Result> {
		// Creating Graphics CommandPool
		let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
			// Select QueueFamily
			.queue_family_index(queue_families.graphics_q_index.unwrap())
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
		let commandpool_graphics = unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
		
		// Creating Transfer CommandPool
		let transfer_commandpool_info = vk::CommandPoolCreateInfo::builder()
			// Select QueueFamily
			.queue_family_index(queue_families.transfer_q_index.unwrap())
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
		let commandpool_transfer = unsafe { logical_device.create_command_pool(&transfer_commandpool_info, None) }?;

		Ok(CommandBufferPools {
			commandpool_graphics,
			commandpool_transfer,
		})
	}
	
	fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
			logical_device.destroy_command_pool(self.commandpool_graphics, None);
			logical_device.destroy_command_pool(self.commandpool_transfer, None);
		}
	}
}

// ===Initialization functions===

// Create Instance
pub fn init_instance(
	entry: &ash::Entry,
	layer_names: &[&str],
) -> Result<ash::Instance, vk::Result> {
	let enginename = std::ffi::CString::new("DesperØ").unwrap();
	let appname = std::ffi::CString::new("Ash Application").unwrap();
	let app_info = vk::ApplicationInfo::builder()
		.application_name(&appname)
		.application_version(vk::make_api_version(0, 0, 0, 1))
		.engine_name(&enginename)
		.engine_version(vk::make_api_version(0, 0, 0, 1))
		.api_version(vk::make_api_version(0, 1, 0, 106));
	
	let layer_names_c: Vec<std::ffi::CString> = layer_names
		.iter()
		.map(|&ln| std::ffi::CString::new(ln).unwrap())
		.collect();
	let layer_name_pointers: Vec<*const i8> = layer_names_c
		.iter()
		.map(|layer_name| layer_name.as_ptr())
		.collect();
	let extension_name_pointers: Vec<*const i8> = vec![
		ash::extensions::ext::DebugUtils::name().as_ptr(),
		ash::extensions::khr::Surface::name().as_ptr(),
		#[cfg(feature = "x11")]
		ash::extensions::khr::XlibSurface::name().as_ptr(),
		#[cfg(feature = "windows")]
		ash::extensions::khr::Win32Surface::name().as_ptr(),
	];
	let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
		.message_severity(
			vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
				| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
				| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
		)
		.message_type(
			vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
				| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
				| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
		)
		.pfn_user_callback(Some(vulkan_debug_utils_callback));

	let instance_create_info = vk::InstanceCreateInfo::builder()
		.push_next(&mut debugcreateinfo)
		.application_info(&app_info)
		.enabled_layer_names(&layer_name_pointers)
		.enabled_extension_names(&extension_name_pointers);
	unsafe { entry.create_instance(&instance_create_info, None) }
}

// Create LogicalDevice and Queues
pub fn init_device_and_queues(
	instance: &ash::Instance,
	physical_device: vk::PhysicalDevice,
	queue_families: &QueueFamilies,
	layer_names: &[&str],
) -> Result<(ash::Device, Queues), vk::Result> {
	let layer_names_c: Vec<std::ffi::CString> = layer_names
		.iter()
		.map(|&ln| std::ffi::CString::new(ln).unwrap())
		.collect();
	let layer_name_pointers: Vec<*const i8> = layer_names_c
		.iter()
		.map(|layer_name| layer_name.as_ptr())
		.collect();

	let priorities = [1.0f32];
	let queue_infos = [
		vk::DeviceQueueCreateInfo::builder()
			.queue_family_index(queue_families.graphics_q_index.unwrap())
			.queue_priorities(&priorities)
			.build(),
		vk::DeviceQueueCreateInfo::builder()
			.queue_family_index(queue_families.transfer_q_index.unwrap())
			.queue_priorities(&priorities)
			.build(),
	];
	let device_extension_name_pointers: Vec<*const i8> =
		vec![ash::extensions::khr::Swapchain::name().as_ptr()];
	let device_create_info = vk::DeviceCreateInfo::builder()
		.queue_create_infos(&queue_infos)
		.enabled_extension_names(&device_extension_name_pointers)
		.enabled_layer_names(&layer_name_pointers);
	let logical_device =
		unsafe { instance.create_device(physical_device, &device_create_info, None)? };
	let graphics_queue =
		unsafe { logical_device.get_device_queue(queue_families.graphics_q_index.unwrap(), 0) };
	let transfer_queue =
		unsafe { logical_device.get_device_queue(queue_families.transfer_q_index.unwrap(), 0) };
	Ok((
		logical_device,
		Queues {
			graphics_queue,
			transfer_queue,
		},
	))
}

// Create PhysicalDevice and PhysicalDeviceProperties
pub fn init_physical_device_and_properties(
	instance: &ash::Instance
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties), vk::Result> {
	let phys_devs = unsafe { instance.enumerate_physical_devices()? };
	let (&physical_device, physical_device_properties) = {
		if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::DISCRETE_GPU) { 
			(physical_device, physical_device_properties) 
		} else if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::INTEGRATED_GPU) {
			(physical_device, physical_device_properties) 
		} else if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::OTHER) {
			(physical_device, physical_device_properties) 
		} else if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::CPU) {
			(physical_device, physical_device_properties) 
		} else {
			panic!("No device detected!");
		}
	};
	
	return Ok((physical_device, physical_device_properties));
}

// Create RenderPass
fn init_renderpass(
	logical_device: &ash::Device,
	physical_device: vk::PhysicalDevice,
	surfaces: &Surface
) -> Result<vk::RenderPass, vk::Result> {
	let attachments = [vk::AttachmentDescription::builder()
		.format(
			surfaces
				.get_formats(physical_device)?
				.first()
				.unwrap()
				.format,
		)
		.load_op(vk::AttachmentLoadOp::CLEAR)
		.store_op(vk::AttachmentStoreOp::STORE)
		.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
		.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
		.initial_layout(vk::ImageLayout::UNDEFINED)
		.final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
		.samples(vk::SampleCountFlags::TYPE_1)
		.build()
	];
	
	let color_attachment_references = [vk::AttachmentReference {
		attachment: 0,
		layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
	}];

	let subpasses = [vk::SubpassDescription::builder()
		.color_attachments(&color_attachment_references)
		.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).build()
	];
	
	let subpass_dependencies = [vk::SubpassDependency::builder()
		.src_subpass(vk::SUBPASS_EXTERNAL)
		.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
		.dst_subpass(0)
		.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
		.dst_access_mask(
			vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
		)
		.build()
	];
	
	let renderpass_info = vk::RenderPassCreateInfo::builder()
		.attachments(&attachments)
		.subpasses(&subpasses)
		.dependencies(&subpass_dependencies);
	let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None)? };
	Ok(renderpass)
}

// Create CommandBuffers
fn create_commandbuffers(
	logical_device: &ash::Device,
	pools: &CommandBufferPools,
	amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
	let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
		.command_pool(pools.commandpool_graphics)
		.command_buffer_count(amount as u32);
	unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}

// Fill CommandBuffers
fn fill_commandbuffers(
	commandbuffers: &[vk::CommandBuffer],
	logical_device: &ash::Device,
	renderpass: &vk::RenderPass,
	swapchain: &Swapchain,
	pipeline: &GraphicsPipeline,
	vb1: &vk::Buffer,
	vb2: &vk::Buffer,
) -> Result<(), vk::Result> {
	for (i, &commandbuffer) in commandbuffers.iter().enumerate() {
		// Beginning of CommandBuffer
		let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
		unsafe {
			logical_device.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
		}
		// Color of clearing window
		let clearvalues = [vk::ClearValue {
			color: vk::ClearColorValue {
				float32: [0.08, 0.08, 0.08, 1.0],
			},
		}];
		
		// Beginning of RenderPass
		let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
			.render_pass(*renderpass)
			.framebuffer(swapchain.framebuffers[i])
			.render_area(vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent: swapchain.extent,
			})
			.clear_values(&clearvalues);
			
		unsafe {
			// Apply RenderPass beginning
			logical_device.cmd_begin_render_pass(
				commandbuffer,
				&renderpass_begininfo,
				vk::SubpassContents::INLINE,
			);
			// Apply GraphicsPipeline
			logical_device.cmd_bind_pipeline(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				pipeline.pipeline,
			);
			// Bind vertex buffers
			logical_device.cmd_bind_vertex_buffers(commandbuffer, 0, &[*vb1], &[0]);
			logical_device.cmd_bind_vertex_buffers(commandbuffer, 1, &[*vb2], &[0]);
			// Apply `draw` command
			logical_device.cmd_draw(commandbuffer, 6, 1, 0, 0);
			// Finish RenderPass
			logical_device.cmd_end_render_pass(commandbuffer);
			// Finish CommandBuffer
			logical_device.end_command_buffer(commandbuffer)?;
		}
	}
	Ok(())
}

pub unsafe extern "system" fn vulkan_debug_utils_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
	let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
	let severity = format!("{:?}", message_severity).to_lowercase();
	let ty = format!("{:?}", message_type).to_lowercase();
	
	let severity = match severity.as_str() {
		"info" 		=> format!("{}", severity).green(),
		"warning" 	=> format!("{}", severity).yellow(),
		"error" 	=> format!("{}", severity).red(),
		"verbose" 	=> format!("{}", severity).blue(),
		&_ => format!("{}", severity).normal(),
	};
	
	println!("[DesperØ][{}][{}] {:?}", severity, ty, message);
	vk::FALSE
}

fn select_device_of_type<'a>(
	instance:	&'a ash::Instance,
	phys_devs: 	&'a Vec<vk::PhysicalDevice>,
	d_type:		vk::PhysicalDeviceType,
) -> Option<(&'a vk::PhysicalDevice, vk::PhysicalDeviceProperties)> {
	for p in phys_devs {
		let props = unsafe { instance.get_physical_device_properties(*p) };
		if props.device_type == d_type {
			return Some((p, props));
		}
	}
	None
}
