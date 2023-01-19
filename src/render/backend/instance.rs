#![allow(dead_code)]
use std::sync::Arc;
use std::mem::ManuallyDrop;
use std::ffi::CString;
use ash::vk;
use crate::render::{
	debug::Debug,
};

/// Structure controlling Vulkan instance and physical device
pub struct Instance {
	pub(crate) entry: Arc<ash::Entry>,
	pub(crate) instance: ash::Instance,
	pub(crate) debugger: ManuallyDrop<Debug>,
	pub(crate) physical_device: Arc<vk::PhysicalDevice>,
	pub(crate) physical_device_properties: Arc<vk::PhysicalDeviceProperties>,
	pub(crate) physical_device_features: Arc<vk::PhysicalDeviceFeatures>,
}

impl Instance {
	/// Initialize [`Instance`]
	pub(crate) fn init(
		app_title: String,
	) -> Result<Instance, vk::Result> {
		let entry = unsafe { ash::Entry::load().expect("Cannot create entry") };		
		
		let layer_names_c: Vec<std::ffi::CString> = vec!["VK_LAYER_KHRONOS_validation"]
			.iter()
			.map(|&ln| std::ffi::CString::new(ln).unwrap())
			.collect();
		let layer_name_pointers: Vec<*const i8> = layer_names_c
			.iter()
			.map(|layer_name| layer_name.as_ptr())
			.collect();

		let extensions = Self::init_extensions();
		let app_info = Self::init_app_info(app_title);
		let mut debug_info = Debug::init_debug_info();
	
		let instance_create_info = vk::InstanceCreateInfo::builder()
			.push_next(&mut debug_info)
			.enabled_layer_names(&layer_name_pointers)
			.enabled_extension_names(&extensions)
			.application_info(&app_info);
		
		let instance = unsafe {entry.create_instance(&instance_create_info, None)?};
		let debugger = ManuallyDrop::new(Debug::init(&entry, &instance)?);
		let (device, device_properties, device_features) = Self::init_physical_device(&instance)?;
		
		Ok(Instance {
			entry: Arc::new(entry),
			instance,
			debugger,
			physical_device: Arc::new(device),
			physical_device_properties: Arc::new(device_properties),
			physical_device_features: Arc::new(device_features),
		})
	}
	
	/// Get queue family properties of current physical device
	pub unsafe fn get_queue_family_properties(&self) -> Vec<vk::QueueFamilyProperties> {
		self.instance.get_physical_device_queue_family_properties(*self.physical_device)
	}
	
	/// Destroy [`Instance`]
	pub unsafe fn cleanup(&mut self) {
		ManuallyDrop::drop(&mut self.debugger);
		self.instance.destroy_instance(None);
	}
	
	/// Create [`Instance`] application info
	fn init_app_info<T: ToString>(title: T) -> vk::ApplicationInfo {
		vk::ApplicationInfo::builder()
			.application_name(&CString::new(title.to_string().as_str()).unwrap())
			.engine_name(&CString::new("DesperØ").unwrap())
			.engine_version(vk::make_api_version(0, 0, 0, 0))
			.api_version(vk::make_api_version(0, 1, 0, 106))
			.build()
	}
	
	/// Init [`Instance`] extensions
	fn init_extensions() -> Vec<*const i8> {
		vec![
			ash::extensions::ext::DebugUtils::name().as_ptr(),
			ash::extensions::khr::Surface::name().as_ptr(),
			
			#[cfg(target_os = "linux")]
			ash::extensions::khr::XlibSurface::name().as_ptr(),
			
			#[cfg(target_os = "windows")]
			ash::extensions::khr::Win32Surface::name().as_ptr(),
		]
	}
	
	/// Init physical device
	fn init_physical_device(
		instance: &ash::Instance,
	) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures), vk::Result> {
		let physical_devices = unsafe { instance.enumerate_physical_devices()? };
		return match [
			vk::PhysicalDeviceType::DISCRETE_GPU,
			vk::PhysicalDeviceType::INTEGRATED_GPU,
			vk::PhysicalDeviceType::OTHER,
			vk::PhysicalDeviceType::CPU,
	    ]
	    .iter()
	    .copied()
	    .find_map(|device_type| Self::select_device_of_type(&instance, &physical_devices, device_type))
	    {
			Some((device, properties, features)) => Ok((*device, properties, features)),
			_ => Err(vk::Result::ERROR_UNKNOWN),
		}
	}
	
	/// Select physical device. Returns `None`, if physical device of given type doesn't exist
	fn select_device_of_type<'a>(
		instance:	&'a ash::Instance,
		physical_devices: 	&'a Vec<vk::PhysicalDevice>,
		d_type:		vk::PhysicalDeviceType,
	) -> Option<(&'a vk::PhysicalDevice, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures)> {
		for p in physical_devices {
			let properties = unsafe { instance.get_physical_device_properties(*p) };
			let features = unsafe { instance.get_physical_device_features(*p) };
			if properties.device_type == d_type {
				return Some((p, properties, features));
			}
		}
		None
	}
}
