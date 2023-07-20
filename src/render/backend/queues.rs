#![allow(dead_code)]
use ash::vk;

use crate::render::backend::{
    window::Window,
    instance::Instance,
};

/// Conserves transfer and graphics queues for current instance
pub struct QueueFamilies {
    pub graphics_queue: vk::Queue,
    pub graphics_index: Option<u32>,
    pub transfer_queue: vk::Queue,
    pub transfer_index: Option<u32>,
}

impl QueueFamilies {
    /// Initialize physical device queues and indices
    pub fn init(
        instance: &Instance,
        window: &Window,
    ) -> Result<(ash::Device, QueueFamilies), vk::Result> {
        let (graphics_index, transfer_index) = Self::get_queue_indices(&instance, &window)?;
        let (logical_device, graphics_queue, transfer_queue) = Self::init_device_and_queues(
            &instance, 
            &graphics_index, 
            &transfer_index
        )?;
        
        Ok((
            logical_device,
            QueueFamilies {
                graphics_queue,
                graphics_index,
                transfer_queue,
                transfer_index,
            }
        ))
    }
    
    /// Get queue indices of current instance
    fn get_queue_indices(
        instance: &Instance, 
        window: &Window
    ) -> Result<(Option<u32>, Option<u32>), vk::Result> {
        let mut graphics_index = None;
        let mut transfer_index = None;
        let queue_family_properties = unsafe { instance.get_queue_family_properties() };
        for (index, qfam) in queue_family_properties.iter().enumerate() {
            if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS) && 
                window.surface.get_physical_device_surface_support(instance.physical_device, index)?
            {
                graphics_index = Some(index as u32);
            }
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && (transfer_index.is_none() || !qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            {
                transfer_index = Some(index as u32);
            }
        }
        
        Ok((graphics_index, transfer_index))
    }
    
    /// Initialize logical device and queues
    fn init_device_and_queues(
        instance: &Instance,
        graphics_index: &Option<u32>,
        transfer_index: &Option<u32>,
    ) -> Result<(ash::Device, vk::Queue, vk::Queue), vk::Result> {    
        let queues_info = Self::get_queues_info(graphics_index, transfer_index);
        let device_extensions = Self::get_device_extensions();
        let mut indexing_features = Self::get_indexing_features();    

        let mut buffer_adress_features = vk::PhysicalDeviceBufferAddressFeaturesEXT::builder()
            .buffer_device_address(true);

        let mut physical_device_features = vk::PhysicalDeviceFeatures2::builder()
            .features(instance.physical_device_features.features)
            .push_next(&mut buffer_adress_features);
        
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queues_info)
            .enabled_extension_names(&device_extensions)
            .push_next(&mut physical_device_features)
            .push_next(&mut indexing_features);
            
        let logical_device = unsafe { instance.instance.create_device(instance.physical_device, &device_create_info, None)? };
        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_index.unwrap(), 0) };
        let transfer_queue = unsafe { logical_device.get_device_queue(transfer_index.unwrap(), 0) };
        
        Ok((
            logical_device,
            graphics_queue,
            transfer_queue,
        ))
    }
    
    /// Get queues creation info
    fn get_queues_info(
        graphics_index: &Option<u32>, 
        transfer_index: &Option<u32>
    ) -> [vk::DeviceQueueCreateInfo; 2] {
        let priorities = [1.0f32];
        [
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(graphics_index.unwrap())
                .queue_priorities(&priorities)
                .build(),
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(transfer_index.unwrap())
                .queue_priorities(&priorities)
                .build(),
        ]
    }
    
    /// Get physical device extensions
    fn get_device_extensions() -> Vec<*const i8> {
        vec![
            ash::extensions::khr::Swapchain::name().as_ptr(),
            ash::extensions::khr::Maintenance3::name().as_ptr(),
            ash::extensions::khr::DeviceGroup::name().as_ptr(),
            ash::vk::KhrShaderNonSemanticInfoFn::name().as_ptr(),
            ash::vk::ExtDescriptorIndexingFn::name().as_ptr(),
            ash::extensions::khr::BufferDeviceAddress::name().as_ptr(),
        ]
    }
    
    /// Get physical device descriptor indexing features
    fn get_indexing_features() -> vk::PhysicalDeviceDescriptorIndexingFeatures  {
        vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
            .runtime_descriptor_array(true)
            .descriptor_binding_variable_descriptor_count(true)
            .build()
    }
}
