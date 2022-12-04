use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;
use crate::Despero;

// Screenshot
pub struct Screenshot;

impl Screenshot {
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
	
	fn screenshot(despero: &mut Despero, full_path: &str) -> Result<(), Box<dyn std::error::Error>> {
		// Create CommandBuffer
		let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(despero.renderer.commandbuffer_pools.commandpool_graphics)
			.command_buffer_count(1);
		let copybuffer = unsafe {
			despero.renderer.device.allocate_command_buffers(&commandbuf_allocate_info)
		}.unwrap()[0];
		// Begin CommandBuffer
		let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		unsafe { despero.renderer.device.begin_command_buffer(copybuffer, &cmd_begin_info) }?;
		
		// Create Image to store
		let ici = vk::ImageCreateInfo::builder()
			.format(vk::Format::R8G8B8A8_UNORM)
			.image_type(vk::ImageType::TYPE_2D)
			.extent(vk::Extent3D {
				width: despero.renderer.swapchain.extent.width,
				height: despero.renderer.swapchain.extent.height,
				depth: 1,
			})
			.array_layers(1)
			.mip_levels(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::LINEAR)
			.usage(vk::ImageUsageFlags::TRANSFER_DST)
			.initial_layout(vk::ImageLayout::UNDEFINED);
		let image = unsafe { 
			despero.renderer.device.create_image(&ici, None)
		}.unwrap();
		
		// Image allocation
		//
		// Image memory requirements
		let requirements = unsafe { despero.renderer.device.get_image_memory_requirements(image) };
		// Allocation info
		let allocation_info = &AllocationCreateDesc {
			name: "Screenshot allocation",
			requirements,
			location: MemoryLocation::GpuToCpu,
			linear: true,
		};
		// Create memory allocation
		let allocation = despero.renderer.allocator.allocate(allocation_info).unwrap();
		// Bind memory allocation to image
		unsafe { despero.renderer.device.bind_image_memory(
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
			despero.renderer.device.cmd_pipeline_barrier(
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
		let source_image = despero.renderer.swapchain.images[despero.renderer.swapchain.current_image];
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
			despero.renderer.device.cmd_pipeline_barrier(
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
				width: despero.renderer.swapchain.extent.width,
				height: despero.renderer.swapchain.extent.height,
				depth: 1,
			})
			.build();
		// Copy Command
		unsafe {
			despero.renderer.device.cmd_copy_image(
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
			despero.renderer.device.cmd_pipeline_barrier(
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
			despero.renderer.device.cmd_pipeline_barrier(
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
		unsafe { despero.renderer.device.end_command_buffer(copybuffer) }?;
		
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
			despero.renderer.device.create_fence(&vk::FenceCreateInfo::default(), None)
		}?;
		// Submit
		unsafe { despero.renderer.device.queue_submit(
			despero.renderer.queues.graphics_queue, 
			&submit_infos, 
			fence
		)? };
		// Wait for fences
		unsafe { despero.renderer.device.wait_for_fences(&[fence], true, std::u64::MAX) }?;
		
		// Remove CommandBuffer and Fence
		unsafe { despero.renderer.device.destroy_fence(fence, None) };
		unsafe {
			despero.renderer.device.free_command_buffers(
				despero.renderer.commandbuffer_pools.commandpool_graphics, 
				&[copybuffer]
			)
		};
		
		// Save Image
		//
		// Pointer to image
		let source_ptr = allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
		// Size of the image in bytes (usize)
		let image_size = unsafe {
			despero.renderer.device.get_image_subresource_layout(
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
		let data = Self::bgra_to_rgba(&data);
		// Destroy VulkanImage
		despero.renderer.allocator.free(allocation)?;
		unsafe { despero.renderer.device.destroy_image(image, None); }
		// Create ImageBuffer
		let screen: image::ImageBuffer<image::Rgba<u8>, _> = image::ImageBuffer::from_raw(
			despero.renderer.swapchain.extent.width,
			despero.renderer.swapchain.extent.height,
			data,
		)
		.expect("Failed create ImageBuffer");
		// Save image
		let screen_image = image::DynamicImage::ImageRgba8(screen);
		screen_image.save(full_path)?;		
		
		Ok(())
	}
	
	pub fn take_jpg(despero: &mut Despero, name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>>{
		Self::screenshot(despero, format!("{}/{}.jpg", path, name).as_str())
	}
	
	pub fn take_png(despero: &mut Despero, name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>>{
		Self::screenshot(despero, format!("{}/{}.png", path, name).as_str())
	}
}
