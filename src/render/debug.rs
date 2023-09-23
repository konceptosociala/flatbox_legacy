use std::sync::Arc;
use ash::vk;
use log::{error, warn, debug, trace};

pub struct Debug {
    loader: Arc<ash::extensions::ext::DebugUtils>,
    messenger: Arc<vk::DebugUtilsMessengerEXT>,
}

use crate::error::FlatboxResult;

impl Debug {
    pub fn init(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> FlatboxResult<Debug> {     
        let loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
        let messenger = unsafe { loader.create_debug_utils_messenger(&Debug::init_debug_info(), None)? };
        
        Ok(Debug {
            loader: Arc::new(loader), 
            messenger: Arc::new(messenger),
        })
    }
    
    pub(crate) unsafe extern "system" fn vulkan_debug_utils_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
        let severity = format!("{:?}", message_severity).to_lowercase();
        
        let msg = message.to_str().expect("An error occurred in Vulkan debug utils callback");
        
        match severity.as_str() {
            "info" => {
                if msg.contains("DEBUG-PRINTF") {
                    let msg = msg
                        .to_string()
                        .replace("Validation Information: [ UNASSIGNED-DEBUG-PRINTF ]", "");
                    debug!("{msg}");
                } else {
                    trace!("{msg}")
                }
            },
            "warning" => warn!("{msg}"),
            "error" => error!("{msg}"),
            "verbose" => trace!("{msg}"),
            &_ => {},
        };
        
        vk::FALSE
    }
    
    pub fn init_debug_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                // vk::DebugUtilsMessageSeverityFlagsEXT::ERROR |
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
