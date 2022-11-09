use colored::Colorize;
use ash::vk;

use crate::graphics::vulkanish::*;

// Create Instance
pub fn init_instance(
	entry: &ash::Entry,
	layer_names: &[&str],
) -> Result<ash::Instance, vk::Result> {
	let enginename = std::ffi::CString::new("DesperØ").unwrap();
	let appname = std::ffi::CString::new("Ash Application").unwrap();
	let app_info = vk::ApplicationInfo::builder()
		.application_name(&appname)
		.application_version(vk::make_api_version(0, 0, 0, 1))
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
		#[cfg(feature = "x11")]
		ash::extensions::khr::XlibSurface::name().as_ptr(),
		#[cfg(feature = "windows")]
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
		.pfn_user_callback(Some(vulkan_debug_utils_callback));

	let instance_create_info = vk::InstanceCreateInfo::builder()
		.push_next(&mut debugcreateinfo)
		.push_next(&mut validation_features)
		.enabled_layer_names(&layer_name_pointers)
		.enabled_extension_names(&extension_name_pointers)
		.application_info(&app_info);
	unsafe { entry.create_instance(&instance_create_info, None) }
}

// Create LogicalDevice and Queues
pub fn init_device_and_queues(
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

	let device_extension_name_pointers: Vec<*const i8> =
		vec![
			ash::extensions::khr::Swapchain::name().as_ptr(),
			ash::vk::KhrShaderNonSemanticInfoFn::name().as_ptr(),
		];
	let device_create_info = vk::DeviceCreateInfo::builder()
		.queue_create_infos(&queue_infos)
		.enabled_extension_names(&device_extension_name_pointers)
		.enabled_layer_names(&layer_name_pointers)
		.enabled_features(&features);
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
pub fn init_physical_device_and_properties(
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

// Create RenderPass
pub fn init_renderpass(
	logical_device: &ash::Device,
	physical_device: vk::PhysicalDevice,
	surfaces: &Surface
) -> Result<vk::RenderPass, vk::Result> {
	let attachments = [
		vk::AttachmentDescription::builder()
			.format(
				surfaces
					.get_formats(physical_device)?
					.first()
					.unwrap()
					.format,
			)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::STORE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
			.samples(vk::SampleCountFlags::TYPE_1)
			.build(),
		vk::AttachmentDescription::builder()
			.format(vk::Format::D32_SFLOAT)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::DONT_CARE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
			.samples(vk::SampleCountFlags::TYPE_1)
			.build(),
	];
	
	// Color attachment reference
	let color_attachment_references = [vk::AttachmentReference {
		attachment: 0,
		layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
	}];
	
	// Depth attachment
	let depth_attachment_references = vk::AttachmentReference {
		attachment: 1,
		layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
	};

	let subpasses = [vk::SubpassDescription::builder()
		.color_attachments(&color_attachment_references)
		.depth_stencil_attachment(&depth_attachment_references)
		.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
		.build()
	];
	
	let subpass_dependencies = [vk::SubpassDependency::builder()
		.src_subpass(vk::SUBPASS_EXTERNAL)
		.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
		.dst_subpass(0)
		.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
		.dst_access_mask(
			vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
		)
		.build()
	];
	
	let renderpass_info = vk::RenderPassCreateInfo::builder()
		.attachments(&attachments)
		.subpasses(&subpasses)
		.dependencies(&subpass_dependencies);
	let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None)? };
	Ok(renderpass)
}

// Create CommandBuffers
pub fn create_commandbuffers(
	logical_device: &ash::Device,
	pools: &CommandBufferPools,
	amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
	let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
		.command_pool(pools.commandpool_graphics)
		.command_buffer_count(amount as u32);
	unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}

pub unsafe extern "system" fn vulkan_debug_utils_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
	let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
	let severity = format!("{:?}", message_severity).to_lowercase();
	let ty = format!("{:?}", message_type).to_lowercase();
	
	let severity = match severity.as_str() {
		"info" 		=> format!("{}", severity).green(),
		"warning" 	=> format!("{}", severity).yellow(),
		"error" 	=> format!("{}", severity).red(),
		"verbose" 	=> format!("{}", severity).blue(),
		&_ => format!("{}", severity).normal(),
	};
	
	if severity == format!("info").green() {
		let msg = message.to_str().expect("An error occurred in Vulkan debug utils callback. What kind of not-String are you handing me?");
		if msg.contains("DEBUG-PRINTF") {
			let msg = msg
				.to_string()
				.replace("Validation Information: [ UNASSIGNED-DEBUG-PRINTF ]", "");
			println!("[DesperØ][printf] {:?}", msg);
		}
	} else {
		println!("[DesperØ][{}][{}] {:?}", severity, ty, message);
	}
	
	vk::FALSE
}

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
