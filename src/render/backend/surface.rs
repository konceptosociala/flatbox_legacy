use ash::vk;
use crate::render::{
	backend::{
		instance::Instance,
		window::Window,
	},
};

pub(crate) struct Surface {
	pub(crate) surface: vk::SurfaceKHR,
	pub(crate) surface_loader: ash::extensions::khr::Surface,
}

impl Surface {
	pub(crate) fn init(
		window: &winit::window::Window,
		instance: &Instance,
	) -> Result<Surface, vk::Result> {
		let surface = unsafe { ash_window::create_surface(
			&instance.entry, 
			&instance.instance, 
			&window, 
			None,
		)? };
		
		let surface_loader = ash::extensions::khr::Surface::new(&instance.entry, &instance.instance);
		Ok(Surface {
			surface,
			surface_loader,
		})
	}
	
	pub(crate) fn get_capabilities(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
		unsafe {
			self.surface_loader.get_physical_device_surface_capabilities(physical_device, self.surface)
		}
	}
	
	#[allow(dead_code)]
	pub(crate) fn get_present_modes(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
		unsafe {
			self.surface_loader
				.get_physical_device_surface_present_modes(physical_device, self.surface)
		}
	}
	
	pub(crate) fn get_formats(
		&self,
		physical_device: vk::PhysicalDevice,
	) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
		unsafe {
			self.surface_loader
				.get_physical_device_surface_formats(physical_device, self.surface)
		}
	}
	
	pub(crate) fn get_physical_device_surface_support(
		&self,
		physical_device: vk::PhysicalDevice,
		queuefamilyindex: usize,
	) -> Result<bool, vk::Result> {
		unsafe {
			self.surface_loader.get_physical_device_surface_support(
				physical_device,
				queuefamilyindex as u32,
				self.surface,
			)
		}
	}

}

impl Drop for Surface {
	fn drop(&mut self) {
		unsafe {
			self.surface_loader.destroy_surface(self.surface, None);
		}
	}
}
