use ash::vk;

use crate::render::backend::{
    queues::QueueFamilies,
    swapchain::Swapchain,
};

/// Contains commandbuffer pools and graphics commandbuffer
pub struct CommandBufferPools {
    pub commandpool_graphics: vk::CommandPool,
    pub commandpool_transfer: vk::CommandPool,
    pub commandbuffers: Vec<vk::CommandBuffer>,
    pub current_commandbuffer: Option<vk::CommandBuffer>,
}

impl CommandBufferPools {
    pub fn init(
        logical_device: &ash::Device,
        queue_families: &QueueFamilies,
        swapchain: &Swapchain,
    ) -> Result<CommandBufferPools, vk::Result> {
        let commandpool_graphics = 
            unsafe { Self::create_graphics_commandpool(&logical_device, &queue_families)? };
        let commandpool_transfer = 
            unsafe { Self::create_transfer_commandpool(&logical_device, &queue_families)? };
        let commandbuffers = 
            unsafe { Self::create_commandbuffers(&logical_device, &commandpool_graphics, swapchain.framebuffers_count())? };
        
        Ok(CommandBufferPools {
            commandpool_graphics,
            commandpool_transfer,
            commandbuffers,
            current_commandbuffer: None,
        })
    }
    
    pub fn get_commandbuffer(&self, index: usize) -> Option<&vk::CommandBuffer> {
        self.commandbuffers.get(index)
    }
    
    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.commandpool_graphics, None);
            logical_device.destroy_command_pool(self.commandpool_transfer, None);
        }
    }
    
    unsafe fn create_graphics_commandpool(
        logical_device: &ash::Device,
        queue_families: &QueueFamilies,
    ) -> Result<vk::CommandPool, vk::Result> {
        let graphics_commandpool_info = 
            vk::CommandPoolCreateInfo::builder()
                .queue_family_index(queue_families.graphics_index.unwrap())
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
                
        logical_device.create_command_pool(&graphics_commandpool_info, None)
    }
    
    unsafe fn create_transfer_commandpool(
        logical_device: &ash::Device,
        queue_families: &QueueFamilies,
    ) -> Result<vk::CommandPool, vk::Result> {
        let transfer_commandpool_info = 
            vk::CommandPoolCreateInfo::builder()
                .queue_family_index(queue_families.transfer_index.unwrap())
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
                
        logical_device.create_command_pool(&transfer_commandpool_info, None)
    }
    
    unsafe fn create_commandbuffers(
        logical_device: &ash::Device,
        commandpool_graphics: &vk::CommandPool,
        amount: usize,
    ) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
        let commandbuf_allocate_info = 
            vk::CommandBufferAllocateInfo::builder()
                .command_pool(*commandpool_graphics)
                .command_buffer_count(amount as u32);
                
        logical_device.allocate_command_buffers(&commandbuf_allocate_info)
    }
}
