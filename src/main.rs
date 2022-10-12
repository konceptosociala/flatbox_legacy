//Distingiloj
#[cfg(all(feature = "x11", feature = "windows"))]
compile_error!("features \"x11\" and \"windows\" cannot be enabled at the same time");

//Moduloj
pub mod extras;

use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use ash::vk;
use extras::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop)?;
	
    let entry = unsafe { ash::Entry::load()? };
    let enginename = std::ffi::CString::new("NulMotor").unwrap();
    let appname = std::ffi::CString::new("Ash Application").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&appname)
        .application_version(vk::make_api_version(0, 0, 0, 1))
        .engine_name(&enginename)
        .engine_version(vk::make_api_version(0, 0, 0, 1))
        .api_version(vk::make_api_version(0, 1, 0, 106));
    
    // Validigaj Tavoloj
    let layer_names: Vec<std::ffi::CString> = vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let extension_name_pointers: Vec<*const i8> = vec![
		ash::extensions::ext::DebugUtils::name().as_ptr(),
        ash::extensions::khr::Surface::name().as_ptr(),
        
        #[cfg(feature = "x11")]
        ash::extensions::khr::XlibSurface::name().as_ptr(),
        #[cfg(feature = "windows")]
        ash::extensions::khr::Win32Surface::name().as_ptr(),
	];
    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
        
    let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
			vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
				//| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
				//| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
				| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
		)
        .message_type(
			vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
				| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
				| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
		)
        .pfn_user_callback(Some(vulkan_debug_utils_callback));
    
    let instance_create_info = vk::InstanceCreateInfo::builder()
		.push_next(&mut debugcreateinfo)
		.application_info(&app_info)
		.enabled_layer_names(&layer_name_pointers)
		.enabled_extension_names(&extension_name_pointers);
		    
    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };
    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);
    let utils_messenger = unsafe { debug_utils.create_debug_utils_messenger(&debugcreateinfo, None)? };
    
    let surface = unsafe { ash_window::create_surface(
		&entry, 
		&instance, 
		window.raw_display_handle(), 
		window.raw_window_handle(), 
		None
	)? };
	
	let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
    
    //Fizikaj aparatoj
    let phys_devs = unsafe { instance.enumerate_physical_devices()? };
	let (&physical_device, physical_device_properties) = {
		if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::DISCRETE_GPU) { 
			(physical_device, physical_device_properties) 
		} else if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::INTEGRATED_GPU) {
			(physical_device, physical_device_properties) 
		} else if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::OTHER) {
			(physical_device, physical_device_properties) 
		} else if let Some((physical_device, physical_device_properties)) = select_device_of_type(&instance, &phys_devs, vk::PhysicalDeviceType::CPU) {
			(physical_device, physical_device_properties) 
		} else {
			panic!("Neniu aparato ekzistas");
		}
    };

    //Vicaj familioj
    let queuefamilyproperties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
	let qfamindices = {
        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;
        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS) && 
				unsafe { surface_loader.get_physical_device_surface_support(physical_device, index as u32, surface)? }
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
        (
            found_graphics_q_index.unwrap(),
            found_transfer_q_index.unwrap(),
        )
    };
    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfamindices.0)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfamindices.1)
            .queue_priorities(&priorities)
            .build(),
    ];
    
    let device_extension_name_pointers: Vec<*const i8> = vec![
		ash::extensions::khr::Swapchain::name().as_ptr()
	];
	
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers)
        .enabled_layer_names(&layer_name_pointers);
        
    let logical_device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    let graphics_queue = unsafe { logical_device.get_device_queue(qfamindices.0, 0) };
    let transfer_queue = unsafe { logical_device.get_device_queue(qfamindices.1, 0) };
    
    //Ŝanĝoĉeno
    let surface_capabilities 	= unsafe { surface_loader.get_physical_device_surface_capabilities(physical_device, surface) };
    let surface_present_modes 	= unsafe { surface_loader.get_physical_device_surface_present_modes(physical_device, surface) };
    let surface_formats 		= unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface) };
    
    dbg!(&surface_capabilities);
    dbg!(&surface_present_modes);
    dbg!(&surface_formats);
    
    
    
    //Detruado
    unsafe { 
		logical_device.destroy_device(None);
		debug_utils.destroy_debug_utils_messenger(utils_messenger, None);
		surface_loader.destroy_surface(surface, None);
		instance.destroy_instance(None);
	};
	
    Ok(())
}
