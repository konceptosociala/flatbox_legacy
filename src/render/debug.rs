use std::sync::Arc;
use colored::Colorize;
use ash::vk;

pub struct Debug {
	loader: Arc<ash::extensions::ext::DebugUtils>,
	messenger: Arc<vk::DebugUtilsMessengerEXT>,
}

impl Debug {
	pub(crate) fn init(
		entry: &ash::Entry,
		instance: &ash::Instance,
	) -> Result<Debug, vk::Result> {		
		let loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
		let messenger = unsafe { loader.create_debug_utils_messenger(&Debug::init_debug_info(), None)? };
		
		Ok(Debug {
			loader: Arc::new(loader), 
			messenger: Arc::new(messenger),
		})
	}
	
	pub fn info<M>(msg: M)
	where
		String: From<M>,
	{
		println!("[DesperØ][{}][debug] {}", format!("info").green(), String::from(msg));
	}
	
	pub fn warning<M>(msg: M)
	where
		String: From<M>,
	{
		println!("[DesperØ][{}][debug] {}", format!("warning").yellow(), String::from(msg));
	}
	
	pub fn error<M>(msg: M)
	where
		String: From<M>,
	{
		println!("[DesperØ][{}][debug] {}", format!("error").red(), String::from(msg));
	}
	
	pub(crate) unsafe extern "system" fn vulkan_debug_utils_callback(
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
			let msg = message.to_str().expect("An error occurred in Vulkan debug utils callback");
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
	
	pub(crate) fn init_debug_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
		vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(
				vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
				vk::DebugUtilsMessageSeverityFlagsEXT::ERROR |
				vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
				vk::DebugUtilsMessageSeverityFlagsEXT::INFO
			)
			.message_type(
				vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
				vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
				vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
			)
			.pfn_user_callback(Some(Debug::vulkan_debug_utils_callback))
			.build()
	}
}

impl Drop for Debug {
	fn drop(&mut self) {
		unsafe { self.loader.destroy_debug_utils_messenger(*self.messenger, None) };
	}
}
