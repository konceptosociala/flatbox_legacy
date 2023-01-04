#![allow(dead_code)]
use ash::vk;

use crate::render::{
	debug::Debug,
	backend::{
		window::Window,
		instance::Instance,
	},
};

// QueueFamilies
pub(crate) struct QueueFamilies {
	pub(crate) graphics_q_index: Option<u32>,
	pub(crate) transfer_q_index: Option<u32>,
}

impl QueueFamilies {
	pub(crate) fn init(
		instance: &Instance,
		window: &Window,
	) -> Result<QueueFamilies, vk::Result>{
		// Get queue families
		let queue_family_properties = unsafe { instance.get_queue_family_properties() };
		let mut found_graphics_q_index = None;
		let mut found_transfer_q_index = None;
		// Get indices of queue families
		for (index, qfam) in queue_family_properties.iter().enumerate() {
			if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS) && 
				window.surface.get_physical_device_surface_support(instance.physical_device, index)?
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
pub(crate) struct Queues {
	pub(crate) graphics_queue: vk::Queue,
	pub(crate) transfer_queue: vk::Queue,
}

// Create LogicalDevice and Queues
pub(crate) fn init_device_and_queues(
	instance: &ash::Instance,
	physical_device: vk::PhysicalDevice,
	queue_families: &QueueFamilies,
) -> Result<(ash::Device, Queues), vk::Result> {
	let layer_names_c: Vec<std::ffi::CString> = vec!["VK_LAYER_KHRONOS_validation"]
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
	
	// Get PhysDev's features
	let features = vk::PhysicalDeviceFeatures::builder()
		.fill_mode_non_solid(true);
		
	let mut indexing_features =
        vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
			.runtime_descriptor_array(true)
			.descriptor_binding_variable_descriptor_count(true);

	let device_extension_name_pointers: Vec<*const i8> =
		vec![
			ash::extensions::khr::Swapchain::name().as_ptr(),
			ash::vk::KhrShaderNonSemanticInfoFn::name().as_ptr(),
		];
	let device_create_info = vk::DeviceCreateInfo::builder()
		.queue_create_infos(&queue_infos)
		.enabled_extension_names(&device_extension_name_pointers)
		.enabled_layer_names(&layer_name_pointers)
		.enabled_features(&features)
		.push_next(&mut indexing_features);
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
