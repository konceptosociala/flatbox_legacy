use ash::vk;
//use ash::version::InstanceV1_0;


pub unsafe extern "system" fn vulkan_debug_utils_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
	let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
	let severity = format!("{:?}", message_severity).to_lowercase();
	let ty = format!("{:?}", message_type).to_lowercase();
	println!("[NulMotor][{}][{}] {:?}", severity, ty, message);
	vk::FALSE
}

pub fn select_device_of_type<'a>(
	instance:	&'a ash::Instance,
	phys_devs: 	&'a Vec<vk::PhysicalDevice>,
	d_type:		vk::PhysicalDeviceType,
) -> Option<(&'a vk::PhysicalDevice, vk::PhysicalDeviceProperties)> {
	let mut device: Option<(&'a vk::PhysicalDevice, vk::PhysicalDeviceProperties)> = None;
	for p in phys_devs {
		let props = unsafe { instance.get_physical_device_properties(*p) };
		if props.device_type == d_type {
			return Some((p, props));
		}
	}
	None
}
