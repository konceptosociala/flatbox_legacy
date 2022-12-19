#![allow(dead_code)]
use ash::vk;

use crate::render::{
	debug::Debug,
	backend::surface::Surface,
};

// QueueFamilies
pub(crate) struct QueueFamilies {
	pub(crate) graphics_q_index: Option<u32>,
	pub(crate) transfer_q_index: Option<u32>,
}

impl QueueFamilies {
	pub(crate) fn init(
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
pub(crate) struct Queues {
	pub(crate) graphics_queue: vk::Queue,
	pub(crate) transfer_queue: vk::Queue,
}

// Create Instance
pub(crate) fn init_instance(
	entry: &ash::Entry,
	layer_names: &[&str],
	app_title: String,
) -> Result<ash::Instance, vk::Result> {
	let enginename = std::ffi::CString::new("DesperØ").unwrap();
	let appname = std::ffi::CString::new(app_title.as_str()).unwrap();
	let app_info = vk::ApplicationInfo::builder()
		.application_name(&appname)
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
		#[cfg(target_os = "linux")]
		ash::extensions::khr::XlibSurface::name().as_ptr(),
		#[cfg(target_os = "windows")]
		ash::extensions::khr::Win32Surface::name().as_ptr(),
	];
	
	let val_features_enabled = vec![vk::ValidationFeatureEnableEXT::DEBUG_PRINTF];
	let mut validation_features = vk::ValidationFeaturesEXT::default();
	validation_features.enabled_validation_feature_count = val_features_enabled.len() as u32;
	validation_features.p_enabled_validation_features = val_features_enabled.as_ptr();
			
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
		.pfn_user_callback(Some(Debug::vulkan_debug_utils_callback));

	let instance_create_info = vk::InstanceCreateInfo::builder()
		.push_next(&mut debugcreateinfo)
		.push_next(&mut validation_features)
		.enabled_layer_names(&layer_name_pointers)
		.enabled_extension_names(&extension_name_pointers)
		.application_info(&app_info);
	unsafe { entry.create_instance(&instance_create_info, None) }
}

// Create LogicalDevice and Queues
pub(crate) fn init_device_and_queues(
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

// Create PhysicalDevice and PhysicalDeviceProperties
pub(crate) fn init_physical_device_and_properties(
	instance: &ash::Instance
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures), vk::Result> {
	let phys_devs = unsafe { instance.enumerate_physical_devices()? };
	let (&physical_device, physical_device_properties, physical_device_features) = {
		if let Some((physical_device, physical_device_properties, physical_device_features)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::DISCRETE_GPU) { 
			(physical_device, physical_device_properties, physical_device_features) 
		} else if let Some((physical_device, physical_device_properties, physical_device_features)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::INTEGRATED_GPU) {
			(physical_device, physical_device_properties, physical_device_features)  
		} else if let Some((physical_device, physical_device_properties, physical_device_features)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::OTHER) {
			(physical_device, physical_device_properties, physical_device_features)  
		} else if let Some((physical_device, physical_device_properties, physical_device_features)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::CPU) {
			(physical_device, physical_device_properties, physical_device_features)  
		} else {
			panic!("No device detected!");
		}
	};
	
	return Ok((physical_device, physical_device_properties, physical_device_features));
}

// Select PhysicalDevice function
fn select_device_of_type<'a>(
	instance:	&'a ash::Instance,
	phys_devs: 	&'a Vec<vk::PhysicalDevice>,
	d_type:		vk::PhysicalDeviceType,
) -> Option<(&'a vk::PhysicalDevice, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures)> {
	for p in phys_devs {
		let props = unsafe { instance.get_physical_device_properties(*p) };
		let features = unsafe { instance.get_physical_device_features(*p) };
		if props.device_type == d_type {
			return Some((p, props, features));
		}
	}
	None
}
