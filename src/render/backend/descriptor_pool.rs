use ash::vk;

use crate::render::{
	backend::{
		swapchain::Swapchain,
		
	},
	renderer::MAX_NUMBER_OF_TEXTURES,
};

pub(crate) struct DescriptorPool {
	pub(crate) descriptor_pool: vk::DescriptorPool,
		
	pub(crate) descriptor_sets_camera: Vec<vk::DescriptorSet>, 
	pub(crate) descriptor_sets_texture: Vec<vk::DescriptorSet>,
	pub(crate) descriptor_sets_light: Vec<vk::DescriptorSet>,
	
	pub(crate) descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl DescriptorPool {
	pub(crate) fn init(
		logical_device: &ash::Device,
		swapchain: &Swapchain,
	) -> Result<DescriptorPool, vk::Result> {
		let descriptor_pool = Self::create_descriptor_pool(&logical_device, &swapchain)?;
		
		let camera_set_layout = unsafe { Self::create_descriptor_set_layout(
			&logical_device,
			vk::DescriptorType::UNIFORM_BUFFER,
			vk::ShaderStageFlags::VERTEX,
			0,
			1,
		)};
			
		let texture_set_layout = unsafe { Self::create_descriptor_set_layout(
			&logical_device,
			vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
			vk::ShaderStageFlags::FRAGMENT,
			0,
			MAX_NUMBER_OF_TEXTURES,
		)};
		
		let light_set_layout = unsafe { Self::create_descriptor_set_layout(
			&logical_device,
			vk::DescriptorType::STORAGE_BUFFER,
			vk::ShaderStageFlags::FRAGMENT,
			0,
			1,
		)};
		
		let descriptor_sets_camera = unsafe { Self::allocate_descriptor_sets(
			&logical_device,
			&swapchain, 
			descriptor_pool,
			camera_set_layout,
		)?};
		
		let descriptor_sets_texture = unsafe { Self::allocate_descriptor_sets(
			&logical_device,
			&swapchain, 
			descriptor_pool,
			texture_set_layout,
		)?};
		
		let descriptor_sets_light = unsafe { Self::allocate_descriptor_sets(
			&logical_device,
			&swapchain, 
			descriptor_pool,
			light_set_layout,
		)?};
		
		Ok(DescriptorPool {
			descriptor_pool,	
			descriptor_sets_camera, 
			descriptor_sets_texture,
			descriptor_sets_light,
			descriptor_set_layouts: vec![camera_set_layout, texture_set_layout, light_set_layout],
		})
	}
	
	pub(crate) unsafe fn bind_buffers(
		&self,
		logical_device: &ash::Device,
		camera_buffer: &Buffer,
		light_buffer: &Buffer,
	){
		for descset in &self.descriptor_sets_camera {
			let buffer_infos = [vk::DescriptorBufferInfo {
				buffer: camera_buffer.buffer,
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
		
		for descset in &self.descriptor_sets_light {
			let buffer_infos = [vk::DescriptorBufferInfo {
				buffer: light_buffer.buffer,
				offset: 0,
				range: 8,
			}];
			let desc_sets_write = [vk::WriteDescriptorSet::builder()
				.dst_set(*descset)
				.dst_binding(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.buffer_info(&buffer_infos)
				.build()
			];
			unsafe { logical_device.update_descriptor_sets(&desc_sets_write, &[]) };
		}
	}
	
	pub(crate) unsafe fn cleanup(&self, logical_device: &ash::Device){
		for dsl in &self.descriptor_set_layouts {
			logical_device.destroy_descriptor_set_layout(*dsl, None);
		}
	}
	
	unsafe fn create_descriptor_set_layout(
		logical_device: &ash::Device,
		dtype: vk::DescriptorType,
		stage_flags: vk::ShaderStageFlags,
		binding: u32,
		dcount: u32,
	) -> vk::DescriptorSetLayout {
		let description = [
			vk::DescriptorSetLayoutBinding::builder()
				.binding(binding)
				.descriptor_type(dtype)
				.descriptor_count(dcount)
				.stage_flags(stage_flags)
				.build()
		];
		
		let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&description);
			
		logical_device.create_descriptor_set_layout(&create_info, None)
	}
	
	unsafe fn create_descriptor_pool(
		logical_device: &ash::Device,
		swapchain: &Swapchain,
	) -> Result<vk::DescriptorPool, vk::Result> {
		let pool_sizes = [
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::UNIFORM_BUFFER,
				descriptor_count: swapchain.amount_of_images,
			},
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
				descriptor_count: MAX_NUMBER_OF_TEXTURES * swapchain.amount_of_images,
			},
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::STORAGE_BUFFER,
				descriptor_count: swapchain.amount_of_images,
			},
		];
		
		let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
			.max_sets(3 * swapchain.amount_of_images)
			.pool_sizes(&pool_sizes); 
			
		logical_device.create_descriptor_pool(&descriptor_pool_info, None)
	}
	
	unsafe fn allocate_descriptor_sets(
		logical_device: &ash::Device,
		swapchain: &Swapchain,
		descriptor_pool: vk::DescriptorPool,
		descriptor_set_layout: vk::DescriptorSetLayout,
	) -> Result<Vec<vk::DescriptorSet>, vk::Result> {
		let desc_layouts = vec![descriptor_set_layout; swapchain.amount_of_images as usize];
		let descriptor_set_allocate_info_camera = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(descriptor_pool)
			.set_layouts(&desc_layouts);
		logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info_camera)
	}
}
