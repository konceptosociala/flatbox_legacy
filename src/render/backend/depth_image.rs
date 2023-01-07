use ash::vk;
use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;

use crate::render::renderer::extract_option;

pub struct DepthImage {
	pub(crate) depth_image: vk::Image,							  
	pub(crate) depth_image_allocation: Option<Allocation>,		  
	pub(crate) depth_imageview: vk::ImageView,
}

impl DepthImage {
	pub fn new(
		logical_device: &ash::Device,
		allocator: &mut Allocator,
		extent: &vk::Extent2D,
		queue_family_indices: &[u32; 1],
	) -> Result<DepthImage, vk::Result> {
		let depth_image = unsafe { Self::new_depth_image(logical_device, Self::new_extent3d(extent), queue_family_indices) }?;
		let depth_image_allocation = unsafe { Self::new_depth_image_allocation(&depth_image, allocator, &logical_device) };
		let depth_imageview = unsafe { Self::new_depth_imageview(&depth_image, &logical_device)? };
			
		Ok(DepthImage {
			depth_image,
			depth_image_allocation: Some(depth_image_allocation),
			depth_imageview,
		})
	}
	
	pub(crate) unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &mut Allocator) {
		let alloc = extract_option(&mut self.depth_image_allocation);
		allocator.free(alloc).unwrap();
		logical_device.destroy_image_view(self.depth_imageview, None);
		logical_device.destroy_image(self.depth_image, None);
	}
	
	fn new_extent3d(extent: &vk::Extent2D) -> vk::Extent3D {
		vk::Extent3D {
			width: extent.width,
			height: extent.height,
			depth: 1,
		}
	}
	
	unsafe fn new_depth_image(
		logical_device: &ash::Device,
		extent3d: vk::Extent3D,
		queue_family_indices: &[u32],
	) -> Result<vk::Image, vk::Result> {
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
			.queue_family_indices(&queue_family_indices);
		
		logical_device.create_image(&depth_image_info, None)
	}
	
	unsafe fn new_depth_image_allocation(
		depth_image: &vk::Image,
		allocator: &mut Allocator,
		logical_device: &ash::Device,
	) -> Allocation {
		let depth_image_allocation_info = &AllocationCreateDesc {
			name: "Depth image buffer",
			requirements: logical_device.get_image_memory_requirements(*depth_image),
			location: MemoryLocation::GpuOnly,
			linear: true,
		};
		
		let depth_image_allocation = allocator.allocate(depth_image_allocation_info).unwrap();
		logical_device.bind_image_memory(
			*depth_image, 
			depth_image_allocation.memory(), 
			depth_image_allocation.offset()
		).unwrap();
		
		return depth_image_allocation;
	}
	
	unsafe fn new_depth_imageview(
		depth_image: &vk::Image,
		logical_device: &ash::Device,
	) -> Result<vk::ImageView, vk::Result> {
		let subresource_range = vk::ImageSubresourceRange::builder()
			.aspect_mask(vk::ImageAspectFlags::DEPTH)
			.base_mip_level(0)
			.level_count(1)
			.base_array_layer(0)
			.layer_count(1);
			
		let imageview_create_info = vk::ImageViewCreateInfo::builder()
			.image(*depth_image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk::Format::D32_SFLOAT)
			.subresource_range(*subresource_range);
			
		logical_device.create_image_view(&imageview_create_info, None)
	}
}
