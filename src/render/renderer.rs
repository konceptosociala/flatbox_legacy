use std::sync::Arc;
use std::any::TypeId;
use std::collections::HashMap;
use ash::vk;
use ash::Device;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use nalgebra as na;
use winit::{
	window::WindowBuilder,
};
use hecs::World;
use hecs_schedule::*;

use crate::render::{
	backend::{
		instance::Instance,
		window::Window,
		queues::QueueFamilies,
		swapchain::Swapchain,
		pipeline::Pipeline,
		commandbuffers::CommandBufferPools,
		buffer::Buffer,		
		descriptor_pool::DescriptorPool,
	},
	pbr::{
		model::*,
		texture::*,
		material::*,
	},
	transform::Transform,
};

/// Maximum number of textures, which can be pushed to descriptor sets
pub const MAX_NUMBER_OF_TEXTURES: u32 = 2;

/// Main rendering collection, including Vulkan components
pub struct Renderer {
	pub(crate) instance: Instance,
	pub(crate) window: Window,
	pub(crate) queue_families: QueueFamilies,
	pub(crate) device: Device,
	pub(crate) swapchain: Swapchain,
	pub(crate) renderpass: vk::RenderPass,
	pub(crate) pipelines: HashMap<TypeId, Pipeline>,
	pub(crate) commandbuffer_pools: CommandBufferPools,
	pub(crate) allocator: Allocator,
	pub(crate) camera_buffer: Buffer,
	pub(crate) light_buffer: Buffer,
	pub(crate) descriptor_pool: DescriptorPool,
	pub(crate) texture_storage: TextureStorage,
	pub(crate) materials: Vec<Arc<(dyn Material + Send + Sync)>>,
}

impl Renderer {	
	pub(crate) fn init(window_builder: WindowBuilder) -> Result<Renderer, Box<dyn std::error::Error>> {
		let instance = Instance::init(get_window_title(&window_builder))?;
		let window	= Window::init(&instance, window_builder)?;
		let (device, queue_families) = QueueFamilies::init(&instance, &window)?;
			
		let mut allocator = Allocator::new(&AllocatorCreateDesc {
			instance: instance.instance.clone(),
			device: device.clone(),
			physical_device: *instance.physical_device.clone(),
			debug_settings: Default::default(),
			buffer_device_address: true,
		}).expect("Cannot create allocator");
		
		let mut swapchain = Swapchain::init(&instance, &device, &window.surface, &queue_families, &mut allocator)?;
		
		let renderpass = Pipeline::init_renderpass(&device, *instance.physical_device.clone(), &window.surface)?;
		swapchain.create_framebuffers(&device, renderpass)?;
		
		let commandbuffer_pools = CommandBufferPools::init(&device, &queue_families, &swapchain)?;
		
		let mut camera_buffer = Buffer::new(
			&device,
			&mut allocator,
			128,
			vk::BufferUsageFlags::UNIFORM_BUFFER,
			MemoryLocation::CpuToGpu,
			"Uniform buffer"
		)?;
		
		let camera_transform: [[[f32; 4]; 4]; 2] = [
			na::Matrix4::identity().into(),
			na::Matrix4::identity().into(),
		];
		camera_buffer.fill(&device, &mut allocator, &camera_transform)?;
		
		let mut light_buffer = Buffer::new(
			&device,
			&mut allocator,
			8,
			vk::BufferUsageFlags::STORAGE_BUFFER,
			MemoryLocation::CpuToGpu,
			"Light buffer",
		)?;
		light_buffer.fill(&device, &mut allocator, &[0.,0.])?;
		
		let descriptor_pool = unsafe { DescriptorPool::init(&device, &swapchain)? };
		unsafe { descriptor_pool.bind_buffers(&device, &camera_buffer, &light_buffer) };
		 
		Ok(Renderer {
			instance,
			window,
			queue_families,
			device,
			swapchain,
			renderpass,
			pipelines: HashMap::new(),
			commandbuffer_pools,
			allocator,
			camera_buffer,
			light_buffer,
			descriptor_pool,
			texture_storage: TextureStorage::new(),
			materials: vec![],
		})
	}
	
	pub fn bind_material<M: Material + Sync + Send>(&mut self){	
		self.pipelines.insert(TypeId::of::<M>(), M::pipeline(&self));
	}
	
	pub fn create_texture<P: AsRef<std::path::Path>>(
		&mut self,
		path: P,
		filter: Filter,
	) -> usize {
		self.texture_storage.new_texture_from_file(
			path,
			filter,
			&self.device,
			&mut self.allocator,
			&self.commandbuffer_pools.commandpool_graphics,
			&self.queue_families.graphics_queue,
		).expect("Cannot create texture")
	}
	
	pub fn create_material(
		&mut self,
		material: Arc<(dyn Material + Send + Sync)>,
	) -> MaterialHandle
	{
		let index = self.materials.len();
		self.materials.push(material);
		return MaterialHandle::new(index);
	}
	
	pub(crate) unsafe fn update_commandbuffer<W: borrow::ComponentBorrow>(
		&mut self,
		world: &mut SubWorld<W>,
		index: usize
	) -> Result<(), vk::Result> {
		let imageinfos = self.texture_storage.get_descriptor_image_info();
		let descriptorwrite_image = vk::WriteDescriptorSet::builder()
			.dst_set(self.descriptor_pool.texture_sets[self.swapchain.current_image])
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.image_info(&imageinfos)
			.build();

		self.device.update_descriptor_sets(&[descriptorwrite_image], &[]);
		
		let commandbuffer = *self.commandbuffer_pools.get_commandbuffer(index).unwrap();
		let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
		
		let clear_values = Self::set_clear_values(na::Vector3::new(0.0, 0.0, 0.0));
		
		let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
			.render_pass(self.renderpass)
			.framebuffer(self.swapchain.framebuffers[index])
			.render_area(vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent: self.swapchain.extent,
			})
			.clear_values(&clear_values);
		
		self.device.begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
		
		self.device.cmd_begin_render_pass(
			commandbuffer,
			&renderpass_begininfo,
			vk::SubpassContents::INLINE,
		);
		
		self.device.cmd_bind_descriptor_sets(
			commandbuffer,
			vk::PipelineBindPoint::GRAPHICS,
			self.descriptor_pool.pipeline_layout,
			0,
			&[
				self.descriptor_pool.camera_sets[index],
				self.descriptor_pool.texture_sets[index],
				self.descriptor_pool.light_sets[index],
			],
			&[],
		);
		
		for material_type in self.pipelines.keys() {
			let pipeline = self.pipelines.get(&material_type).unwrap();
						
			self.device.cmd_bind_pipeline(
				commandbuffer,
				vk::PipelineBindPoint::GRAPHICS,
				pipeline.pipeline,
			);
			
			for (_, (mesh, handle, transform)) in &mut world.query::<(
				&Mesh, &MaterialHandle, &Transform,
			)>(){
				if let Some(vertexbuffer) = &mesh.vertexbuffer {
					if let Some(instancebuffer) = &mesh.instancebuffer {
						if let Some(indexbuffer) = &mesh.indexbuffer {
							let material = &self.materials[handle.get()];
							if (**material).type_id() == *material_type {
								// Bind position buffer						
								self.device.cmd_bind_index_buffer(
									commandbuffer,
									indexbuffer.buffer,
									0,
									vk::IndexType::UINT32,
								);
								
								self.device.cmd_bind_vertex_buffers(
									commandbuffer,
									0,
									&[vertexbuffer.buffer],
									&[0],
								);
								
								self.device.cmd_bind_vertex_buffers(
									commandbuffer,
									1,
									&[instancebuffer.buffer],
									&[0],
								);
								
								let transform_matrices = transform.to_matrices();
								let transform_ptr = &transform_matrices as *const _ as *const u8;
								let transform_slice = std::slice::from_raw_parts(transform_ptr, 128);
								self.device.cmd_push_constants(
									commandbuffer,
									self.descriptor_pool.pipeline_layout,
									vk::ShaderStageFlags::VERTEX,
									0,
									transform_slice,
								);
								
								self.device.cmd_draw_indexed(
									commandbuffer,
									mesh.indexdata.len() as u32,
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
		
		self.device.cmd_end_render_pass(commandbuffer);
		self.device.end_command_buffer(commandbuffer)?;
			
		Ok(())
	}
	
	//~ pub(crate) fn recreate_swapchain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		//~ unsafe {
			//~ self.device
				//~ .device_wait_idle()
				//~ .expect("something wrong while waiting");
			//~ self.swapchain.cleanup(&self.device, &mut self.allocator);
		//~ }
		//~ // Recreate Swapchain
		//~ self.swapchain = Swapchain::init(
			//~ &self.instance,
			//~ &self.device,
			//~ &self.window.surface,
			//~ &self.queue_families,
			//~ &mut self.allocator,
		//~ )?;
		
		//~ // Recreate FrameBuffers
		//~ self.swapchain.create_framebuffers(&self.device, self.renderpass)?;
		
		//~ // Recreate Pipeline
		//~ for material_type in self.pipelines.keys() {
			//~ let pipeline = self.pipelines.get(&material_type).unwrap();
			
			//~ pipeline.cleanup(&self.device);
			
			
		//~ }
		//~ self.pipeline = Pipeline::init(
			//~ &self.device, 
			//~ &self.swapchain, 
			//~ &self.renderpass
		//~ )?;
		
		//~ Ok(())
	//~ }
	
	pub(crate) fn fill_lightbuffer<T: Sized>(
		&mut self,
		data: &[T],
	) -> Result<(), vk::Result>{
		self.light_buffer.fill(&self.device, &mut self.allocator, data)?;
		Ok(())
	}
	
	pub fn screenshot(&mut self, full_path: &str) -> Result<(), Box<dyn std::error::Error>> {
		// Create CommandBuffer
		let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(self.commandbuffer_pools.commandpool_graphics)
			.command_buffer_count(1);
		let copybuffer = unsafe {
			self.device.allocate_command_buffers(&commandbuf_allocate_info)
		}.unwrap()[0];
		// Begin CommandBuffer
		let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		unsafe { self.device.begin_command_buffer(copybuffer, &cmd_begin_info) }?;
		
		// Create Image to store
		let ici = vk::ImageCreateInfo::builder()
			.format(vk::Format::R8G8B8A8_UNORM)
			.image_type(vk::ImageType::TYPE_2D)
			.extent(vk::Extent3D {
				width: self.swapchain.extent.width,
				height: self.swapchain.extent.height,
				depth: 1,
			})
			.array_layers(1)
			.mip_levels(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::LINEAR)
			.usage(vk::ImageUsageFlags::TRANSFER_DST)
			.initial_layout(vk::ImageLayout::UNDEFINED);
		let image = unsafe { 
			self.device.create_image(&ici, None)
		}.unwrap();
		
		// Image allocation
		//
		// Image memory requirements
		let requirements = unsafe { self.device.get_image_memory_requirements(image) };
		// Allocation info
		let allocation_info = &AllocationCreateDesc {
			name: "Screenshot allocation",
			requirements,
			location: MemoryLocation::GpuToCpu,
			linear: true,
		};
		// Create memory allocation
		let allocation = self.allocator.allocate(allocation_info).unwrap();
		// Bind memory allocation to image
		unsafe { self.device.bind_image_memory(
			image,
			allocation.memory(), 
			allocation.offset())
		}.unwrap();
		
		// ImageMemoryBarrier
		let barrier = vk::ImageMemoryBarrier::builder()
			.image(image)
			.src_access_mask(vk::AccessFlags::empty())
			.dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
			.old_layout(vk::ImageLayout::UNDEFINED)
			.new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			})
			.build();
		// Bind IMB to CommandBuffer
		unsafe {
			self.device.cmd_pipeline_barrier(
				copybuffer,
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::TRANSFER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier],
			)
		};
		
		// Layout transition
		let source_image = self.swapchain.images[self.swapchain.current_image];
		let barrier = vk::ImageMemoryBarrier::builder()
			.image(source_image)
			.src_access_mask(vk::AccessFlags::MEMORY_READ)
			.dst_access_mask(vk::AccessFlags::TRANSFER_READ)
			.old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
			.new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			})
			.build();
		unsafe {
			self.device.cmd_pipeline_barrier(
				copybuffer,
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::TRANSFER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier],
			)
		};
		
		// Copying
		//
		// Copying description
		let copy_area = vk::ImageCopy::builder()
			.src_subresource(vk::ImageSubresourceLayers {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				mip_level: 0,
				base_array_layer: 0,
				layer_count: 1,
			})
			.src_offset(vk::Offset3D::default())
			.dst_subresource(vk::ImageSubresourceLayers {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				mip_level: 0,
				base_array_layer: 0,
				layer_count: 1,
			})
			.dst_offset(vk::Offset3D::default())
			.extent(vk::Extent3D {
				width: self.swapchain.extent.width,
				height: self.swapchain.extent.height,
				depth: 1,
			})
			.build();
		// Copy Command
		unsafe {
			self.device.cmd_copy_image(
				copybuffer,
				source_image,
				vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
				image,
				vk::ImageLayout::TRANSFER_DST_OPTIMAL,
				&[copy_area],
			)
		};
		
		// Next layout (to read)
		let barrier = vk::ImageMemoryBarrier::builder()
			.image(image)
			.src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
			.dst_access_mask(vk::AccessFlags::MEMORY_READ)
			.old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
			.new_layout(vk::ImageLayout::GENERAL)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			})
			.build();
		unsafe {
			self.device.cmd_pipeline_barrier(
				copybuffer,
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::TRANSFER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier],
			)
		};
		
		// Turn back `source_image` layout
		let barrier = vk::ImageMemoryBarrier::builder()
			.image(source_image)
			.src_access_mask(vk::AccessFlags::TRANSFER_READ)
			.dst_access_mask(vk::AccessFlags::MEMORY_READ)
			.old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
			.new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			})
			.build();
		unsafe {
			self.device.cmd_pipeline_barrier(
				copybuffer,
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::TRANSFER,
				vk::DependencyFlags::empty(),
				&[],
				&[],
				&[barrier],
			)
		};
		// End CommandBuffer
		unsafe { self.device.end_command_buffer(copybuffer) }?;
		
		// Submit CommandBuffer
		//
		// Submit info
		let submit_infos = [
			vk::SubmitInfo::builder()
				.command_buffers(&[copybuffer])
				.build()
		];
		// Create fence (to wait until CommandBuffer is finished)
		let fence = unsafe {
			self.device.create_fence(&vk::FenceCreateInfo::default(), None)
		}?;
		// Submit
		unsafe { self.device.queue_submit(
			self.queue_families.graphics_queue, 
			&submit_infos, 
			fence
		)? };
		// Wait for fences
		unsafe { self.device.wait_for_fences(&[fence], true, std::u64::MAX) }?;
		
		// Remove CommandBuffer and Fence
		unsafe { self.device.destroy_fence(fence, None) };
		unsafe {
			self.device.free_command_buffers(
				self.commandbuffer_pools.commandpool_graphics, 
				&[copybuffer]
			)
		};
		
		// Save Image
		//
		// Pointer to image
		let source_ptr = allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
		// Size of the image in bytes (usize)
		let image_size = unsafe {
			self.device.get_image_subresource_layout(
				image,
				vk::ImageSubresource {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					mip_level: 0,
					array_layer: 0,
				},
			).size as usize
		};
		// Image to bytes
		let mut data = Vec::<u8>::with_capacity(image_size);
		unsafe {
			std::ptr::copy(
				source_ptr,
				data.as_mut_ptr(),
				image_size,
			);
			data.set_len(image_size);
		}
		let data = bgra_to_rgba(&data);
		// Destroy VulkanImage
		self.allocator.free(allocation)?;
		unsafe { self.device.destroy_image(image, None); }
		// Create ImageBuffer
		let screen: image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::from_raw(
			self.swapchain.extent.width,
			self.swapchain.extent.height,
			data,
		)
		.expect("Failed create ImageBuffer");
		// Save image
		let screen_image = image::DynamicImage::ImageRgba8(screen);
		screen_image.save(full_path)?;		
		
		Ok(())
	}
	
	/// Function to destroy renderer. Used in [`Despero`]'s ['Drop'] function
	pub(crate) fn cleanup(&mut self, world: &mut World){
		unsafe {
			self.device.device_wait_idle().expect("Error halting device");	
			self.texture_storage.cleanup(&self.device, &mut self.allocator);
			self.descriptor_pool.cleanup(&self.device);
			self.device.destroy_buffer(self.camera_buffer.buffer, None);
			self.device.free_memory(self.camera_buffer.allocation.as_ref().unwrap().memory(), None);
			self.device.destroy_buffer(self.light_buffer.buffer, None);
			// Models clean
			for (_, m) in &mut world.query::<&mut Mesh>(){
				if let Some(vb) = &mut m.vertexbuffer {
					// Reassign VertexBuffer allocation to remove
					let alloc = extract_option(&mut vb.allocation);
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(vb.buffer, None);
				}
				
				if let Some(xb) = &mut m.indexbuffer {
					// Reassign IndexBuffer allocation to remove
					let alloc = extract_option(&mut xb.allocation);
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(xb.buffer, None);
				}
				
				if let Some(ib) = &mut m.instancebuffer {
					// Reassign IndexBuffer allocation to remove
					let alloc = extract_option(&mut ib.allocation);
					self.allocator.free(alloc).unwrap();
					self.device.destroy_buffer(ib.buffer, None);
				}
			}
			self.commandbuffer_pools.cleanup(&self.device);
			for pipeline in self.pipelines.values() {
				pipeline.cleanup(&self.device);
			}
			self.device.destroy_render_pass(self.renderpass, None);
			self.swapchain.cleanup(&self.device, &mut self.allocator);
			self.device.destroy_device(None);
			self.window.cleanup();
			self.instance.cleanup();
		};
	}
	
	fn set_clear_values(
		color: na::Vector3<f32>
	) -> [vk::ClearValue; 2] {
		[
			vk::ClearValue {
				color: vk::ClearColorValue {
					float32: na::Vector4::from([color.x, color.y, color.z, 1.0]).into(),
				},
			},
			vk::ClearValue {
				depth_stencil: vk::ClearDepthStencilValue {
					depth: 1.0,
					stencil: 0,
				},
			},
		]
	}
}

fn bgra_to_rgba(data: &Vec<u8>) -> Vec<u8> {
	let mut rgba: Vec<u8> = data.clone();
	for mut i in 0..data.len()/4 {
		i = i*4;
		rgba[i]   = data[i+2];
		rgba[i+1] = data[i+1];
		rgba[i+2] = data[i];
		rgba[i+3] = data[i+3];
	}
	return rgba;
}

fn get_window_title(window_builder: &WindowBuilder) -> String {
	String::from(window_builder.window.title.clone())
}

pub fn extract_option<T>(option: &mut Option<T>) -> T {
	let mut empty: Option<T> = None;
	std::mem::swap(&mut empty, option);
	empty.unwrap()
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}
