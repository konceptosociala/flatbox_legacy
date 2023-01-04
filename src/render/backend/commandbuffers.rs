use ash::vk;

use crate::render::backend::queues::QueueFamilies;

// CommandBuffers Pools
pub(crate) struct CommandBufferPools {
	pub(crate) commandpool_graphics: vk::CommandPool,
	pub(crate) commandpool_transfer: vk::CommandPool,
}

impl CommandBufferPools {
	pub(crate) fn init(
		logical_device: &ash::Device,
		queue_families: &QueueFamilies,
	) -> Result<CommandBufferPools, vk::Result> {
		// Creating Graphics CommandPool
		let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
			// Select QueueFamily
			.queue_family_index(queue_families.graphics_index.unwrap())
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
		let commandpool_graphics = unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
		
		// Creating Transfer CommandPool
		let transfer_commandpool_info = vk::CommandPoolCreateInfo::builder()
			// Select QueueFamily
			.queue_family_index(queue_families.transfer_index.unwrap())
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
		let commandpool_transfer = unsafe { logical_device.create_command_pool(&transfer_commandpool_info, None) }?;

		Ok(CommandBufferPools {
			commandpool_graphics,
			commandpool_transfer,
		})
	}
	
	// Create CommandBuffers
	pub(crate) fn create_commandbuffers(
		logical_device: &ash::Device,
		pools: &CommandBufferPools,
		amount: usize,
	) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
		let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(pools.commandpool_graphics)
			.command_buffer_count(amount as u32);
		unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
	}
	
	pub(crate) fn cleanup(&self, logical_device: &ash::Device) {
		unsafe {
			logical_device.destroy_command_pool(self.commandpool_graphics, None);
			logical_device.destroy_command_pool(self.commandpool_transfer, None);
		}
	}
}
