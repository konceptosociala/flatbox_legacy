use std::mem::size_of;
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use nalgebra as na;

use crate::graphics::{
	inits::*,
	model::*,
};

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
	pub graphics_q_index: Option<u32>,
	pub transfer_q_index: Option<u32>,
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
	// Depth buffer
	pub depth_image: vk::Image,							  
	pub depth_image_allocation: Allocation,		  
	pub depth_imageview: vk::ImageView,
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
		allocator: &mut Allocator,
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
		
		// Depth image
		let extent3d = vk::Extent3D {
			width: extent.width,
			height: extent.height,
			depth: 1,
		};
		
		let depth_image_info = vk::ImageCreateInfo::builder()
			.image_type(vk::ImageType::TYPE_2D)
			.format(vk::Format::D32_SFLOAT)
			.extent(extent3d)
			.mip_levels(1)
			.array_layers(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::OPTIMAL)
			.usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.queue_family_indices(&queuefamilies);
		
		let depth_image = unsafe { logical_device.create_image(&depth_image_info, None)? };
		
		let depth_image_allocation_info = &AllocationCreateDesc {
			name: "Depth image buffer",
			requirements: unsafe { logical_device.get_image_memory_requirements(depth_image) },
			location: MemoryLocation::GpuOnly,
			linear: true,
		};
		
		let depth_image_allocation = allocator.allocate(depth_image_allocation_info).unwrap();
		unsafe { logical_device.bind_image_memory(
			depth_image, 
			depth_image_allocation.memory(), 
			depth_image_allocation.offset()).unwrap() 
		};
		
		let subresource_range = vk::ImageSubresourceRange::builder()
			.aspect_mask(vk::ImageAspectFlags::DEPTH)
			.base_mip_level(0)
			.level_count(1)
			.base_array_layer(0)
			.layer_count(1);
		let imageview_create_info = vk::ImageViewCreateInfo::builder()
			.image(depth_image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk::Format::D32_SFLOAT)
			.subresource_range(*subresource_range);
		let depth_imageview =
			unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
		
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
			depth_image,
			depth_image_allocation,
			depth_imageview,
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
	pub fn create_framebuffers(
		&mut self,
		logical_device: &ash::Device,
		renderpass: vk::RenderPass,
	) -> Result<(), vk::Result> {
		for iv in &self.imageviews {
			let iview = [*iv, self.depth_imageview];
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
	
	pub unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
		logical_device.destroy_image_view(self.depth_imageview, None);
		logical_device.destroy_image(self.depth_image, None);
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
	pub pipeline: vk::Pipeline,
	pub layout: vk::PipelineLayout,
	pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
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
				format: vk::Format::R32G32B32_SFLOAT,
			},
			/*vk::VertexInputAttributeDescription {
				binding: 1,
				location: 1,
				offset: 0,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			vk::VertexInputAttributeDescription {
				binding: 1,
				location: 2,
				offset: 16,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			vk::VertexInputAttributeDescription {
				binding: 1,
				location: 3,
				offset: 32,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			vk::VertexInputAttributeDescription {
				binding: 1,
				location: 4,
				offset: 48,
				format: vk::Format::R32G32B32A32_SFLOAT,
			},
			vk::VertexInputAttributeDescription {
				binding: 1,
				location: 5,
				offset: 64,
				format: vk::Format::R32G32B32_SFLOAT,
			},*/
		];
		// Input Bindings' description
		//
		// stride	  - binding variables' size
		// input_rate - frequency, when the data is changed (per vertex/ per instance)
		let vertex_binding_descs = [
			vk::VertexInputBindingDescription {
				binding: 0,
				stride: 12,
				input_rate: vk::VertexInputRate::VERTEX,
			},
			/*vk::VertexInputBindingDescription {
				binding: 1,
				stride: 76,
				input_rate: vk::VertexInputRate::INSTANCE,
			}*/
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
		let colourblend_info = vk::PipelineColorBlendStateCreateInfo::builder()
			.attachments(&colorblend_attachments);
		let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
		
		// Bind resource descriptor
		let descriptorset_layout_binding_descs = [
			vk::DescriptorSetLayoutBinding::builder()
				// Binding = 0
				.binding(0)
				// Resource type = uniform buffer
				.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
				// Descriptor count = 1
				.descriptor_count(1)
				// Use the buffer in vertex shaders only
				.stage_flags(vk::ShaderStageFlags::VERTEX)
				.build()
		];

		let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&descriptorset_layout_binding_descs);
		let descriptorsetlayout = unsafe {
			logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
		}?;
		let desclayouts = vec![descriptorsetlayout];
		
		// Push Constant
		let push_constant = vk::PushConstantRange::builder()
			.offset(0)
			.size(size_of::<InstanceData>() as u32)
			.stage_flags(vk::ShaderStageFlags::VERTEX)
			.build();
			
		let pushconstants = vec![push_constant];
		
		// Pipeline layout
		let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder()
			.push_constant_ranges(&pushconstants)
			.set_layouts(&desclayouts);
			
		let pipelinelayout = unsafe { logical_device.create_pipeline_layout(&pipelinelayout_info, None) }?;
		
		// Graphics Pipeline
		let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&shader_stages)
			.vertex_input_state(&vertex_input_info)
			.input_assembly_state(&input_assembly_info)
			.viewport_state(&viewport_info)
			.rasterization_state(&rasterizer_info)
			.multisample_state(&multisampler_info)
			.depth_stencil_state(&depth_stencil_info)
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
		
		// Destroy used shader modules
		unsafe {
			logical_device.destroy_shader_module(fragmentshader_module, None);
			logical_device.destroy_shader_module(vertexshader_module, None);
		}
		
		Ok(GraphicsPipeline {
			pipeline: graphicspipeline,
			layout: pipelinelayout,
			descriptor_set_layouts: desclayouts,
		})
	}
	
	pub fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
			for dsl in &self.descriptor_set_layouts {
                logical_device.destroy_descriptor_set_layout(*dsl, None);
            }
			logical_device.destroy_pipeline(self.pipeline, None);
			logical_device.destroy_pipeline_layout(self.layout, None);
		}
	}
}

// Buffer
#[derive(Debug)]
pub struct Buffer {
	pub buffer: vk::Buffer,
	pub allocation: Option<Allocation>,
	pub allocation_name: String,
	pub size_in_bytes: u64,
	pub buffer_usage: vk::BufferUsageFlags,
	pub memory_location: MemoryLocation,
}

impl Buffer {
	pub fn new(
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
	
	pub fn fill<T: Sized>(
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


pub struct CommandBufferPools {
	pub commandpool_graphics: vk::CommandPool,
	pub commandpool_transfer: vk::CommandPool,
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
	
	pub fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
			logical_device.destroy_command_pool(self.commandpool_graphics, None);
			logical_device.destroy_command_pool(self.commandpool_transfer, None);
		}
	}
}
